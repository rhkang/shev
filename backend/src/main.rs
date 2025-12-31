mod api;
mod config;
mod consumer;
mod db;
mod executor;
mod middleware;
mod producer;
mod queue;
mod store;

use std::net::SocketAddr;

use axum::{Router, middleware as axum_middleware};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::middleware::{IpFilter, ip_filter_middleware};

use crate::api::create_api_router;
use clap::Parser;

use crate::config::{Args, get_db_path};
use crate::consumer::start_consumer;
use crate::db::Database;
use crate::producer::{ScheduleManager, TimerManager, create_http_producer_router};
use crate::queue::create_event_queue;
use crate::store::JobStore;

#[tokio::main]
async fn main() {
    let args = Args::parse();

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
    let schedules = store.load_schedules().await;

    let handler_count = store.get_handlers().await.len();
    info!(
        "Loaded {} handler(s), {} timer(s), and {} schedule(s) from database",
        handler_count,
        timers.len(),
        schedules.len()
    );

    let (sender, receiver) = create_event_queue(queue_size);

    let timer_manager = TimerManager::new(store.clone());
    for timer in timers {
        timer_manager.register_timer(timer, sender.clone()).await;
    }

    let schedule_manager = ScheduleManager::new(store.clone());
    for schedule in schedules {
        schedule_manager.register_schedule(schedule, sender.clone()).await;
    }

    let consumer_store = store.clone();
    tokio::spawn(async move {
        start_consumer(receiver, consumer_store).await;
    });

    let ip_filter = IpFilter::new(args.allowed_ips.clone(), args.allowed_write_ips.clone());
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(create_http_producer_router(sender.clone()))
        .merge(create_api_router(store, timer_manager, schedule_manager, sender))
        .layer(cors)
        .layer(axum_middleware::from_fn_with_state(ip_filter, ip_filter_middleware));

    let host = if args.listen { [0, 0, 0, 0] } else { [127, 0, 0, 1] };
    let addr = SocketAddr::from((host, port));
    info!("Server listening on {}", addr);
    if !args.allowed_ips.is_empty() {
        info!("Allowed read IPs: {:?}", args.allowed_ips);
    }
    if !args.allowed_write_ips.is_empty() {
        info!("Allowed write IPs: {:?}", args.allowed_write_ips);
    } else if args.listen {
        info!("Write operations restricted to localhost only");
    }

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
