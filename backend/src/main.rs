mod api;
mod config;
mod consumer;
mod db;
mod executor;
mod producer;
mod queue;
mod store;

use std::net::SocketAddr;

use axum::Router;
use tokio::net::TcpListener;
use tracing::info;

use crate::api::create_api_router;
use crate::config::get_db_path;
use crate::consumer::{ConsumerControl, start_consumer};
use crate::db::Database;
use crate::producer::{TimerManager, create_http_producer_router};
use crate::queue::create_event_queue;
use crate::store::JobStore;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting shev - Shell Event System");

    let db_path = get_db_path();
    info!("Using database: {}", db_path);

    let db = Database::open(&db_path).expect("Failed to open database");
    db.init_schema().await.expect("Failed to init schema");

    let cancelled = db.cancel_stale_jobs().await;
    if cancelled > 0 {
        info!("Cancelled {} stale job(s) from previous run", cancelled);
    }

    let port = db.get_port().await;
    let queue_size = db.get_queue_size().await;

    let store = JobStore::new(db);
    store.load_handlers().await;
    let timers = store.load_timers().await;

    let handler_count = store.get_handlers().await.len();
    info!(
        "Loaded {} handler(s) and {} timer(s) from database",
        handler_count,
        timers.len()
    );

    let control = ConsumerControl::new();

    let (sender, receiver) = create_event_queue(queue_size);

    let timer_manager = TimerManager::new(sender.clone(), store.clone());
    for timer in timers {
        timer_manager.register_timer(timer).await;
    }

    let consumer_store = store.clone();
    let consumer_control = control.clone();
    tokio::spawn(async move {
        start_consumer(receiver, consumer_store, consumer_control).await;
    });

    let app = Router::new()
        .merge(create_http_producer_router(sender, timer_manager.clone()))
        .merge(create_api_router(store, control, timer_manager));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Server listening on {}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
