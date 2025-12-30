use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::Serialize;
use uuid::Uuid;

use crate::consumer::ConsumerControl;
use crate::models::{EventHandler, Job, JobStatus};
use crate::store::JobStore;

#[derive(Clone)]
pub struct ApiState {
    pub store: JobStore,
    pub control: ConsumerControl,
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

async fn get_handlers(State(state): State<ApiState>) -> Json<Vec<EventHandler>> {
    Json(state.store.get_handlers().await)
}

pub fn create_api_router(store: JobStore, control: ConsumerControl) -> Router {
    let state = ApiState { store, control };

    Router::new()
        .route("/status", get(get_status))
        .route("/jobs", get(get_jobs))
        .route("/jobs/completed", get(get_completed_jobs))
        .route("/jobs/:job_id", get(get_job))
        .route("/jobs/:job_id/cancel", post(cancel_job))
        .route("/consumer/start", post(start_consumer))
        .route("/consumer/stop", post(stop_consumer))
        .route("/handlers", get(get_handlers))
        .with_state(state)
}
