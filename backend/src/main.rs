mod api;
mod config;
mod consumer;
mod executor;
mod models;
mod producer;
mod queue;
mod store;

use std::net::SocketAddr;

use axum::Router;
use tokio::net::TcpListener;
use tracing::info;

use crate::api::create_api_router;
use crate::config::Config;
use crate::consumer::{ConsumerControl, start_consumer};
use crate::producer::{TimerManager, create_http_producer_router};
use crate::queue::create_event_queue;
use crate::store::JobStore;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting shev - Shell Event System");

    let config = Config::load_or_default("shev.json");
    info!(
        "Loaded {} handler(s) and {} timer(s) from config",
        config.handlers.len(),
        config.timers.len()
    );

    let store = JobStore::new();
    let control = ConsumerControl::new();

    for handler_config in config.handlers {
        let handler = handler_config.into();
        store.register_handler(handler).await;
    }

    let (sender, receiver) = create_event_queue(config.queue_size);

    let timer_manager = TimerManager::new(sender.clone(), store.clone());
    for timer_config in config.timers {
        timer_manager.register_timer(timer_config.into()).await;
    }

    let consumer_store = store.clone();
    let consumer_control = control.clone();
    tokio::spawn(async move {
        start_consumer(receiver, consumer_store, consumer_control).await;
    });

    let app = Router::new()
        .merge(create_http_producer_router(sender, timer_manager))
        .merge(create_api_router(store, control));

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    info!("Server listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
