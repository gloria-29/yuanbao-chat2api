mod yuanbao;
use crate::yuanbao::{
    ChatCompletionRequest, ChatMessage, ChatMessages, ChatModel, Config, Yuanbao,
};
use futures::stream::StreamExt;
use tokio_util::sync::CancellationToken;
use tracing::metadata::LevelFilter;
use tracing::instrument;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

#[instrument]
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_filter(LevelFilter::DEBUG)
                .with_filter(filter_fn(|meta| {
                    meta.target().starts_with("yuanbao_chat2api")
                })),
        )
        .init();
    let config: Config = tokio::fs::read_to_string("config.yml")
        .await
        .unwrap()
        .parse()
        .unwrap();
    let yuanbao = Yuanbao::new(config);
    let receiver = yuanbao
        .create_completion(
            ChatCompletionRequest {
                messages: ChatMessages(vec![ChatMessage {
                    role: "user".to_string(),
                    content: "x+1=2, x=?".to_string(),
                }]),
                chat_model: ChatModel::DeepSeekR1,
            },
            CancellationToken::new(),
        )
        .await
        .unwrap();
    let mut receiver = Box::pin(receiver);
    while let Some(event) = receiver.next().await {
        dbg!(event);
    }
}
