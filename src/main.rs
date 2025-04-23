mod service;
mod yuanbao;

use crate::service::{Config, Handler, Service};
use anyhow::Context;
use axum::Router;
use axum::extract::State;
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
    let config: Config = tokio::fs::read_to_string("config.yml")
        .await
        .context("cannot get config.yaml")
        .unwrap()
        .parse()
        .context("cannot parse config.yaml")
        .unwrap();
    let port = config.port;
    let service = Service::new(config);
    let app = Router::new()
        .route("/v1/models", get(Handler::models))
        .route("/v1/chat/completions", post(Handler::chat_completions))
        .with_state(service);
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();
    info!("Launched the service on :{port}");
    axum::serve(listener, app).await.unwrap();
}
