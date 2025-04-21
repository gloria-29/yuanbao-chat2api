use crate::yuanbao::{
    ChatCompletionEvent, ChatCompletionMessageType, ChatCompletionRequest, ChatMessage,
    ChatMessages, ChatModel, Config, Yuanbao,
};
use anyhow::Context;
use async_channel::Receiver;
use axum::http::StatusCode;
use axum::response::Sse;
use axum::response::sse::Event;
use axum::{Json, debug_handler};
use futures::Stream;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::pin::Pin;
use std::sync::LazyLock;
use std::task::Poll;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Serialize)]
pub struct ModelList {
    pub object: String,
    pub data: Vec<Model>,
}
#[derive(Serialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub owned_by: String,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct AxumChatCompletionRequest {
    messages: ChatMessages,
    model: String,
    #[serde(default)]
    stream: bool,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AxumChatCompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub created: i64,
    pub model: String,
    #[serde(rename = "object")]
    pub object_type: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub finish_reason: Option<String>,
    pub index: u32,
    pub delta: ChatMessage,
}
struct MyStream {
    receiver: Pin<Box<Receiver<ChatCompletionEvent>>>,
    cancel_token: CancellationToken,
}
impl Stream for MyStream {
    type Item = ChatCompletionEvent;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.receiver.poll_next_unpin(cx) {
            Poll::Ready(res) => Poll::Ready(res),
            Poll::Pending => Poll::Pending,
        }
    }
}
impl Drop for MyStream {
    fn drop(&mut self) {
        self.cancel_token.cancel();
    }
}

pub struct Service {}
impl Service {
    pub fn new() -> Service {
        Service {}
    }
    pub async fn models(&self) -> Json<ModelList> {
        Json::from(ModelList {
            object: "list".to_string(),
            data: vec![
                Model {
                    id: "deepseek-v3".to_string(),
                    object: "model".to_string(),
                    owned_by: "deepseek".to_string(),
                },
                Model {
                    id: "deepseek-r1".to_string(),
                    object: "model".to_string(),
                    owned_by: "deepseek".to_string(),
                },
            ],
        })
    }
    pub async fn chat_completions(
        &self,
        req: AxumChatCompletionRequest,
    ) -> anyhow::Result<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
        let config: Config = tokio::fs::read_to_string("config.yml")
            .await
            .context("cannot get config.yaml")?
            .parse()
            .context("cannot parse config.yaml")?;
        let yuanbao = Yuanbao::new(config);
        let model = req.model.parse()?;
        let cancel_token = CancellationToken::new();
        let receiver = yuanbao
            .create_completion(
                ChatCompletionRequest {
                    messages: req.messages,
                    chat_model: model,
                },
                cancel_token.clone(),
                // https://users.rust-lang.org/t/disconnected-state-on-warps-sse/112716
                // https://github.com/tokio-rs/axum/discussions/1914
            )
            .await
            .context("cannot create completion")?;
        let receiver = Box::pin(receiver);
        let uuid = Uuid::new_v4().to_string();
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        Ok(Sse::new(
            MyStream {
                receiver,
                cancel_token: cancel_token.clone(),
            }
            .map(move |event| {
                let mut message = ChatMessage {
                    content: None,
                    role: "assistant".to_string(),
                    reasoning_content: None,
                };
                let mut finish_reason = None;
                match event {
                    ChatCompletionEvent::Message(msg) => match msg.r#type {
                        ChatCompletionMessageType::Think => {
                            message = ChatMessage {
                                role: "assistant".to_string(),
                                content: None,
                                reasoning_content: Some(msg.text),
                            }
                        }
                        ChatCompletionMessageType::Msg => {
                            message = ChatMessage {
                                role: "assistant".to_string(),
                                content: Some(msg.text),
                                reasoning_content: None,
                            }
                        }
                    },
                    ChatCompletionEvent::Error(err) => {
                        finish_reason = Some(format!("{:#}", err));
                    }
                    ChatCompletionEvent::Finish(f) => {
                        finish_reason = Some(f);
                    }
                }
                Ok(Event::default().data(
                    serde_json::to_string(&AxumChatCompletionResponse {
                        id: uuid.to_string(),
                        choices: vec![Choice {
                            finish_reason,
                            index: 0,
                            delta: message,
                        }],
                        created: time,
                        model: model.to_common_string(),
                        object_type: "chat.completion.chunk".to_string(),
                    })
                    .unwrap(),
                ))
            }),
        ))
    }
}
static SERVICE: LazyLock<Service> = LazyLock::new(|| Service::new());
pub struct Handler {}
impl Handler {
    #[debug_handler]
    pub async fn models() -> Json<ModelList> {
        SERVICE.models().await
    }
    // #[debug_handler]
    pub async fn chat_completions(
        Json(req): Json<AxumChatCompletionRequest>,
    ) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
        match SERVICE.chat_completions(req).await {
            Ok(sse) => Ok(sse),
            Err(err) => Err((StatusCode::BAD_REQUEST, format!("{:#}", err))),
        }
    }
}
