use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::db::{Job, JobStatus};
use crate::producer::{ScheduleManager, TimerManager};
use crate::queue::EventSender;
use crate::store::JobStore;
use shev_core::ShellType;
use shev_core::api::{
    ConfigResponse, CreateHandlerRequest, CreateScheduleRequest, CreateTimerRequest,
    HandlerResponse, HealthResponse, ReloadResponse, ScheduleResponse, StatusResponse,
    TimerResponse, UpdateConfigRequest, UpdateHandlerRequest, UpdateScheduleRequest,
    UpdateTimerRequest,
};

#[derive(Clone)]
pub struct ApiState {
    pub store: JobStore,
    pub timer_manager: TimerManager,
    pub schedule_manager: ScheduleManager,
    pub sender: EventSender,
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
        total_jobs: jobs.len(),
        pending_jobs: pending,
        running_jobs: running,
        completed_jobs: completed,
        failed_jobs: failed,
    })
}

#[derive(Deserialize)]
pub struct JobsQuery {
    pub status: Option<String>,
}

async fn get_jobs(State(state): State<ApiState>, Query(query): Query<JobsQuery>) -> Json<Vec<Job>> {
    let jobs = match &query.status {
        Some(status_str) => {
            if let Some(status) = JobStatus::from_str(status_str) {
                state.store.get_jobs_by_status(status).await
            } else {
                state.store.get_all_jobs().await
            }
        }
        None => state.store.get_all_jobs().await,
    };
    Json(jobs)
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

fn handler_to_response(h: crate::db::EventHandler) -> HandlerResponse {
    HandlerResponse {
        id: h.id.to_string(),
        event_type: h.event_type,
        shell: format!("{:?}", h.shell).to_lowercase(),
        command: h.command,
        timeout: h.timeout,
        env: h.env,
    }
}

async fn get_handlers(State(state): State<ApiState>) -> Json<Vec<HandlerResponse>> {
    let handlers = state.store.get_handlers().await;
    let responses: Vec<HandlerResponse> = handlers.into_iter().map(handler_to_response).collect();
    Json(responses)
}

async fn get_handler_by_type(
    State(state): State<ApiState>,
    Path(event_type): Path<String>,
) -> Result<Json<HandlerResponse>, StatusCode> {
    state
        .store
        .get_handler(&event_type)
        .await
        .map(|h| Json(handler_to_response(h)))
        .ok_or(StatusCode::NOT_FOUND)
}

async fn create_handler(
    State(state): State<ApiState>,
    Json(request): Json<CreateHandlerRequest>,
) -> Result<Json<HandlerResponse>, (StatusCode, String)> {
    let shell = ShellType::from_str(&request.shell).ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid shell type: {}", request.shell),
        )
    })?;

    let handler = state
        .store
        .create_handler(
            &request.event_type,
            &shell,
            &request.command,
            request.timeout,
            &request.env,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(handler_to_response(handler)))
}

async fn update_handler(
    State(state): State<ApiState>,
    Path(event_type): Path<String>,
    Json(request): Json<UpdateHandlerRequest>,
) -> Result<Json<HandlerResponse>, (StatusCode, String)> {
    let shell = match &request.shell {
        Some(s) => Some(ShellType::from_str(s).ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid shell type: {}", s),
            )
        })?),
        None => None,
    };

    let handler = state
        .store
        .update_handler(
            &event_type,
            shell.as_ref(),
            request.command.as_deref(),
            request.timeout,
            request.env.as_ref(),
        )
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    Ok(Json(handler_to_response(handler)))
}

async fn delete_handler(
    State(state): State<ApiState>,
    Path(event_type): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let deleted = state
        .store
        .delete_handler(&event_type)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if deleted {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            format!("Handler '{}' not found", event_type),
        ))
    }
}

fn timer_to_response(t: crate::db::TimerRecord) -> TimerResponse {
    TimerResponse {
        id: t.id.to_string(),
        event_type: t.event_type,
        context: t.context,
        interval_secs: t.interval_secs,
    }
}

async fn get_timers(State(state): State<ApiState>) -> Json<Vec<TimerResponse>> {
    let timers = state.store.get_timers().await;
    let responses: Vec<TimerResponse> = timers.into_iter().map(timer_to_response).collect();
    Json(responses)
}

async fn get_timer_by_type(
    State(state): State<ApiState>,
    Path(event_type): Path<String>,
) -> Result<Json<TimerResponse>, StatusCode> {
    state
        .store
        .get_timer(&event_type)
        .await
        .map(|t| Json(timer_to_response(t)))
        .ok_or(StatusCode::NOT_FOUND)
}

async fn create_timer(
    State(state): State<ApiState>,
    Json(request): Json<CreateTimerRequest>,
) -> Result<Json<TimerResponse>, (StatusCode, String)> {
    let timer = state
        .store
        .create_timer(&request.event_type, request.interval_secs, &request.context)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    state
        .timer_manager
        .register_timer(timer.clone(), state.sender.clone())
        .await;

    Ok(Json(timer_to_response(timer)))
}

async fn update_timer(
    State(state): State<ApiState>,
    Path(event_type): Path<String>,
    Json(request): Json<UpdateTimerRequest>,
) -> Result<Json<TimerResponse>, (StatusCode, String)> {
    let timer = state
        .store
        .update_timer_record(
            &event_type,
            request.interval_secs,
            request.context.as_deref(),
        )
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    state
        .timer_manager
        .register_timer(timer.clone(), state.sender.clone())
        .await;

    Ok(Json(timer_to_response(timer)))
}

async fn delete_timer(
    State(state): State<ApiState>,
    Path(event_type): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let deleted = state
        .store
        .delete_timer(&event_type)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if deleted {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            format!("Timer '{}' not found", event_type),
        ))
    }
}

async fn reload(State(state): State<ApiState>) -> Json<ReloadResponse> {
    state.store.load_handlers().await;
    let handlers = state.store.get_handlers().await;

    let timers = state.store.load_timers().await;
    for timer in &timers {
        state
            .timer_manager
            .register_timer(timer.clone(), state.sender.clone())
            .await;
    }

    let schedules = state.store.load_schedules().await;
    for schedule in &schedules {
        state
            .schedule_manager
            .register_schedule(schedule.clone(), state.sender.clone())
            .await;
    }

    Json(ReloadResponse {
        success: true,
        handlers_loaded: handlers.len(),
        timers_loaded: timers.len(),
        schedules_loaded: schedules.len(),
    })
}

fn schedule_to_response(s: crate::db::ScheduleRecord) -> ScheduleResponse {
    ScheduleResponse {
        id: s.id.to_string(),
        event_type: s.event_type,
        context: s.context,
        scheduled_time: s.scheduled_time,
        periodic: s.periodic,
    }
}

async fn get_schedules(State(state): State<ApiState>) -> Json<Vec<ScheduleResponse>> {
    let schedules = state.store.get_schedules().await;
    let responses: Vec<ScheduleResponse> =
        schedules.into_iter().map(schedule_to_response).collect();
    Json(responses)
}

async fn get_schedule_by_type(
    State(state): State<ApiState>,
    Path(event_type): Path<String>,
) -> Result<Json<ScheduleResponse>, StatusCode> {
    state
        .store
        .get_schedule(&event_type)
        .await
        .map(|s| Json(schedule_to_response(s)))
        .ok_or(StatusCode::NOT_FOUND)
}

async fn create_schedule(
    State(state): State<ApiState>,
    Json(request): Json<CreateScheduleRequest>,
) -> Result<Json<ScheduleResponse>, (StatusCode, String)> {
    let schedule = state
        .store
        .create_schedule(
            &request.event_type,
            request.scheduled_time,
            &request.context,
            request.periodic,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    state
        .schedule_manager
        .register_schedule(schedule.clone(), state.sender.clone())
        .await;

    Ok(Json(schedule_to_response(schedule)))
}

async fn update_schedule(
    State(state): State<ApiState>,
    Path(event_type): Path<String>,
    Json(request): Json<UpdateScheduleRequest>,
) -> Result<Json<ScheduleResponse>, (StatusCode, String)> {
    let schedule = state
        .store
        .update_schedule_record(
            &event_type,
            request.scheduled_time,
            request.context.as_deref(),
            request.periodic,
        )
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;

    state
        .schedule_manager
        .register_schedule(schedule.clone(), state.sender.clone())
        .await;

    Ok(Json(schedule_to_response(schedule)))
}

async fn delete_schedule(
    State(state): State<ApiState>,
    Path(event_type): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let deleted = state
        .store
        .delete_schedule(&event_type)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    if deleted {
        Ok(Json(serde_json::json!({"deleted": true})))
    } else {
        Err((
            StatusCode::NOT_FOUND,
            format!("Schedule '{}' not found", event_type),
        ))
    }
}

async fn healthcheck(State(state): State<ApiState>) -> Json<HealthResponse> {
    let warnings = state.store.get_warnings().await;
    Json(HealthResponse {
        healthy: warnings.is_empty(),
        warnings,
    })
}

async fn get_config(State(state): State<ApiState>) -> Json<ConfigResponse> {
    let port = state
        .store
        .get_config("port")
        .await
        .unwrap_or_else(|| "3000".to_string());
    let queue_size = state
        .store
        .get_config("queue_size")
        .await
        .unwrap_or_else(|| "100".to_string());

    Json(ConfigResponse { port, queue_size })
}

async fn update_config(
    State(state): State<ApiState>,
    Json(request): Json<UpdateConfigRequest>,
) -> Result<Json<ConfigResponse>, (StatusCode, String)> {
    if let Some(port) = &request.port {
        let port_num: u16 = port
            .parse()
            .map_err(|_| (StatusCode::BAD_REQUEST, format!("Invalid port: {}", port)))?;
        if port_num == 0 {
            return Err((StatusCode::BAD_REQUEST, "Port cannot be 0".to_string()));
        }
        state
            .store
            .set_config("port", port)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    if let Some(queue_size) = &request.queue_size {
        let size: usize = queue_size.parse().map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                format!("Invalid queue_size: {}", queue_size),
            )
        })?;
        if size == 0 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Queue size cannot be 0".to_string(),
            ));
        }
        state
            .store
            .set_config("queue_size", queue_size)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    }

    let port = state
        .store
        .get_config("port")
        .await
        .unwrap_or_else(|| "3000".to_string());
    let queue_size = state
        .store
        .get_config("queue_size")
        .await
        .unwrap_or_else(|| "100".to_string());

    Ok(Json(ConfigResponse { port, queue_size }))
}

pub fn create_api_router(
    store: JobStore,
    timer_manager: TimerManager,
    schedule_manager: ScheduleManager,
    sender: EventSender,
) -> Router {
    let state = ApiState {
        store,
        timer_manager,
        schedule_manager,
        sender,
    };

    Router::new()
        .route("/status", get(get_status))
        .route("/health", get(healthcheck))
        .route("/jobs", get(get_jobs))
        .route("/jobs/{job_id}", get(get_job))
        .route("/jobs/{job_id}/cancel", post(cancel_job))
        // Handlers CRUD
        .route("/handlers", get(get_handlers).post(create_handler))
        .route(
            "/handlers/{event_type}",
            get(get_handler_by_type)
                .put(update_handler)
                .delete(delete_handler),
        )
        // Timers CRUD
        .route("/timers", get(get_timers).post(create_timer))
        .route(
            "/timers/{event_type}",
            get(get_timer_by_type)
                .put(update_timer)
                .delete(delete_timer),
        )
        // Schedules CRUD
        .route("/schedules", get(get_schedules).post(create_schedule))
        .route(
            "/schedules/{event_type}",
            get(get_schedule_by_type)
                .put(update_schedule)
                .delete(delete_schedule),
        )
        // Config
        .route("/config", get(get_config).put(update_config))
        .route("/reload", post(reload))
        .with_state(state)
}
