use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::Serialize;
use uuid::Uuid;

use crate::consumer::ConsumerControl;
use crate::db::{Job, JobStatus};
use crate::producer::TimerManager;
use crate::store::JobStore;

#[derive(Clone)]
pub struct ApiState {
    pub store: JobStore,
    pub control: ConsumerControl,
    pub timer_manager: TimerManager,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub consumer_running: bool,
    pub total_jobs: usize,
    pub pending_jobs: usize,
    pub running_jobs: usize,
    pub completed_jobs: usize,
    pub failed_jobs: usize,
}

async fn get_status(State(state): State<ApiState>) -> Json<StatusResponse> {
    let jobs = state.store.get_all_jobs().await;

    let pending = jobs
        .iter()
        .filter(|j| j.status == JobStatus::Pending)
        .count();
    let running = jobs
        .iter()
        .filter(|j| j.status == JobStatus::Running)
        .count();
    let completed = jobs
        .iter()
        .filter(|j| j.status == JobStatus::Completed)
        .count();
    let failed = jobs
        .iter()
        .filter(|j| j.status == JobStatus::Failed)
        .count();

    Json(StatusResponse {
        consumer_running: state.control.is_running(),
        total_jobs: jobs.len(),
        pending_jobs: pending,
        running_jobs: running,
        completed_jobs: completed,
        failed_jobs: failed,
    })
}

async fn get_jobs(State(state): State<ApiState>) -> Json<Vec<Job>> {
    Json(state.store.get_all_jobs().await)
}

async fn get_completed_jobs(State(state): State<ApiState>) -> Json<Vec<Job>> {
    Json(state.store.get_completed_jobs().await)
}

async fn get_job(
    State(state): State<ApiState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<Job>, StatusCode> {
    state
        .store
        .get_job(job_id)
        .await
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn cancel_job(
    State(state): State<ApiState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<Job>, StatusCode> {
    if state.store.cancel_job(job_id).await {
        state
            .store
            .get_job(job_id)
            .await
            .map(Json)
            .ok_or(StatusCode::NOT_FOUND)
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

#[derive(Serialize)]
pub struct ControlResponse {
    pub success: bool,
    pub consumer_running: bool,
}

async fn start_consumer(State(state): State<ApiState>) -> Json<ControlResponse> {
    state.control.start();
    Json(ControlResponse {
        success: true,
        consumer_running: true,
    })
}

async fn stop_consumer(State(state): State<ApiState>) -> Json<ControlResponse> {
    state.control.stop();
    Json(ControlResponse {
        success: true,
        consumer_running: false,
    })
}

#[derive(Serialize)]
pub struct HandlerResponse {
    pub id: Uuid,
    pub event_type: String,
    pub shell: String,
    pub timeout: Option<u64>,
}

async fn get_handlers(State(state): State<ApiState>) -> Json<Vec<HandlerResponse>> {
    let handlers = state.store.get_handlers().await;
    let responses: Vec<HandlerResponse> = handlers
        .into_iter()
        .map(|h| HandlerResponse {
            id: h.id,
            event_type: h.event_type,
            shell: format!("{:?}", h.shell).to_lowercase(),
            timeout: h.timeout,
        })
        .collect();
    Json(responses)
}

#[derive(Serialize)]
pub struct TimerResponse {
    pub id: Uuid,
    pub event_type: String,
    pub context: String,
    pub interval_secs: u64,
}

async fn get_timers(State(state): State<ApiState>) -> Json<Vec<TimerResponse>> {
    let timers = state.store.get_timers().await;
    let responses: Vec<TimerResponse> = timers
        .into_iter()
        .map(|t| TimerResponse {
            id: t.id,
            event_type: t.event_type,
            context: t.context,
            interval_secs: t.interval_secs,
        })
        .collect();
    Json(responses)
}

#[derive(Serialize)]
pub struct ReloadResponse {
    pub success: bool,
    pub handlers_loaded: usize,
    pub timers_loaded: usize,
}

async fn reload(State(state): State<ApiState>) -> Json<ReloadResponse> {
    state.store.load_handlers().await;
    let handlers = state.store.get_handlers().await;

    let timers = state.store.load_timers().await;
    for timer in &timers {
        state.timer_manager.register_timer(timer.clone()).await;
    }

    Json(ReloadResponse {
        success: true,
        handlers_loaded: handlers.len(),
        timers_loaded: timers.len(),
    })
}

pub fn create_api_router(
    store: JobStore,
    control: ConsumerControl,
    timer_manager: TimerManager,
) -> Router {
    let state = ApiState {
        store,
        control,
        timer_manager,
    };

    Router::new()
        .route("/status", get(get_status))
        .route("/jobs", get(get_jobs))
        .route("/jobs/completed", get(get_completed_jobs))
        .route("/jobs/{job_id}", get(get_job))
        .route("/jobs/{job_id}/cancel", post(cancel_job))
        .route("/consumer/start", post(start_consumer))
        .route("/consumer/stop", post(stop_consumer))
        .route("/handlers", get(get_handlers))
        .route("/timers", get(get_timers))
        .route("/reload", post(reload))
        .with_state(state)
}
