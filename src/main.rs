mod service;
mod yuanbao;

use crate::service::Handler;
use axum::Router;
use axum::routing::{get, post};
use futures::stream::StreamExt;
use tokio::net::TcpListener;
use tracing::metadata::LevelFilter;
use tracing::{info, instrument};
use tracing_subscriber::Layer;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[instrument]
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_filter(LevelFilter::INFO)
                .with_filter(filter_fn(|meta| {
                    meta.target().starts_with("yuanbao_chat2api")
                })),
        )
        .init();
    let app = Router::new()
        .route("/v1/models", get(Handler::models))
        .route("/v1/chat/completions", post(Handler::chat_completions));
    let listener = TcpListener::bind("0.0.0.0:7555").await.unwrap();
    info!("Launched the service on :7555");
    axum::serve(listener, app).await.unwrap();
}
