use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::db::{Event, Job, JobStatus};
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct EventRequest {
    pub event_type: String,
    #[serde(default)]
    pub context: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventResponse {
    #[serde(flatten)]
    pub event: Event,
    pub message: String,
}

#[utoipa::path(
    get,
    path = "/status",
    responses(
        (status = 200, description = "Job status summary", body = StatusResponse)
    ),
    tag = "Status"
)]
pub async fn get_status(State(state): State<ApiState>) -> Json<StatusResponse> {
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

#[derive(Deserialize, ToSchema)]
pub struct JobsQuery {
    pub status: Option<String>,
}

#[utoipa::path(
    get,
    path = "/jobs",
    params(
        ("status" = Option<String>, Query, description = "Filter by job status (pending, running, completed, failed, cancelled)")
    ),
    responses(
        (status = 200, description = "List of jobs", body = Vec<Job>)
    ),
    tag = "Jobs"
)]
pub async fn get_jobs(
    State(state): State<ApiState>,
    Query(query): Query<JobsQuery>,
) -> Json<Vec<Job>> {
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

#[utoipa::path(
    get,
    path = "/jobs/{job_id}",
    params(
        ("job_id" = Uuid, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Job details", body = Job),
        (status = 404, description = "Job not found")
    ),
    tag = "Jobs"
)]
pub async fn get_job(
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

#[utoipa::path(
    post,
    path = "/jobs/{job_id}/cancel",
    params(
        ("job_id" = Uuid, Path, description = "Job ID")
    ),
    responses(
        (status = 200, description = "Job cancelled", body = Job),
        (status = 400, description = "Job cannot be cancelled"),
        (status = 404, description = "Job not found")
    ),
    tag = "Jobs"
)]
pub async fn cancel_job(
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

#[utoipa::path(
    get,
    path = "/handlers",
    responses(
        (status = 200, description = "List of handlers", body = Vec<HandlerResponse>)
    ),
    tag = "Handlers"
)]
pub async fn get_handlers(State(state): State<ApiState>) -> Json<Vec<HandlerResponse>> {
    let handlers = state.store.get_handlers().await;
    let responses: Vec<HandlerResponse> = handlers.into_iter().map(handler_to_response).collect();
    Json(responses)
}

#[utoipa::path(
    get,
    path = "/handlers/{event_type}",
    params(
        ("event_type" = String, Path, description = "Event type")
    ),
    responses(
        (status = 200, description = "Handler details", body = HandlerResponse),
        (status = 404, description = "Handler not found")
    ),
    tag = "Handlers"
)]
pub async fn get_handler_by_type(
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

#[utoipa::path(
    post,
    path = "/handlers",
    request_body = CreateHandlerRequest,
    responses(
        (status = 200, description = "Handler created", body = HandlerResponse),
        (status = 400, description = "Invalid shell type"),
        (status = 500, description = "Internal error")
    ),
    tag = "Handlers"
)]
pub async fn create_handler(
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

#[utoipa::path(
    put,
    path = "/handlers/{event_type}",
    params(
        ("event_type" = String, Path, description = "Event type")
    ),
    request_body = UpdateHandlerRequest,
    responses(
        (status = 200, description = "Handler updated", body = HandlerResponse),
        (status = 400, description = "Invalid shell type"),
        (status = 404, description = "Handler not found")
    ),
    tag = "Handlers"
)]
pub async fn update_handler(
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

#[utoipa::path(
    delete,
    path = "/handlers/{event_type}",
    params(
        ("event_type" = String, Path, description = "Event type")
    ),
    responses(
        (status = 200, description = "Handler deleted"),
        (status = 404, description = "Handler not found"),
        (status = 500, description = "Internal error")
    ),
    tag = "Handlers"
)]
pub async fn delete_handler(
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

#[utoipa::path(
    get,
    path = "/timers",
    responses(
        (status = 200, description = "List of timers", body = Vec<TimerResponse>)
    ),
    tag = "Timers"
)]
pub async fn get_timers(State(state): State<ApiState>) -> Json<Vec<TimerResponse>> {
    let timers = state.store.get_timers().await;
    let responses: Vec<TimerResponse> = timers.into_iter().map(timer_to_response).collect();
    Json(responses)
}

#[utoipa::path(
    get,
    path = "/timers/{event_type}",
    params(
        ("event_type" = String, Path, description = "Event type")
    ),
    responses(
        (status = 200, description = "Timer details", body = TimerResponse),
        (status = 404, description = "Timer not found")
    ),
    tag = "Timers"
)]
pub async fn get_timer_by_type(
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

#[utoipa::path(
    post,
    path = "/timers",
    request_body = CreateTimerRequest,
    responses(
        (status = 200, description = "Timer created", body = TimerResponse),
        (status = 500, description = "Internal error")
    ),
    tag = "Timers"
)]
pub async fn create_timer(
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

#[utoipa::path(
    put,
    path = "/timers/{event_type}",
    params(
        ("event_type" = String, Path, description = "Event type")
    ),
    request_body = UpdateTimerRequest,
    responses(
        (status = 200, description = "Timer updated", body = TimerResponse),
        (status = 404, description = "Timer not found")
    ),
    tag = "Timers"
)]
pub async fn update_timer(
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

#[utoipa::path(
    delete,
    path = "/timers/{event_type}",
    params(
        ("event_type" = String, Path, description = "Event type")
    ),
    responses(
        (status = 200, description = "Timer deleted"),
        (status = 404, description = "Timer not found"),
        (status = 500, description = "Internal error")
    ),
    tag = "Timers"
)]
pub async fn delete_timer(
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

#[utoipa::path(
    post,
    path = "/reload",
    responses(
        (status = 200, description = "Configuration reloaded", body = ReloadResponse)
    ),
    tag = "Config"
)]
pub async fn reload(State(state): State<ApiState>) -> Json<ReloadResponse> {
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

#[utoipa::path(
    get,
    path = "/schedules",
    responses(
        (status = 200, description = "List of schedules", body = Vec<ScheduleResponse>)
    ),
    tag = "Schedules"
)]
pub async fn get_schedules(State(state): State<ApiState>) -> Json<Vec<ScheduleResponse>> {
    let schedules = state.store.get_schedules().await;
    let responses: Vec<ScheduleResponse> =
        schedules.into_iter().map(schedule_to_response).collect();
    Json(responses)
}

#[utoipa::path(
    get,
    path = "/schedules/{event_type}",
    params(
        ("event_type" = String, Path, description = "Event type")
    ),
    responses(
        (status = 200, description = "Schedule details", body = ScheduleResponse),
        (status = 404, description = "Schedule not found")
    ),
    tag = "Schedules"
)]
pub async fn get_schedule_by_type(
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

#[utoipa::path(
    post,
    path = "/schedules",
    request_body = CreateScheduleRequest,
    responses(
        (status = 200, description = "Schedule created", body = ScheduleResponse),
        (status = 500, description = "Internal error")
    ),
    tag = "Schedules"
)]
pub async fn create_schedule(
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

#[utoipa::path(
    put,
    path = "/schedules/{event_type}",
    params(
        ("event_type" = String, Path, description = "Event type")
    ),
    request_body = UpdateScheduleRequest,
    responses(
        (status = 200, description = "Schedule updated", body = ScheduleResponse),
        (status = 404, description = "Schedule not found")
    ),
    tag = "Schedules"
)]
pub async fn update_schedule(
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

#[utoipa::path(
    delete,
    path = "/schedules/{event_type}",
    params(
        ("event_type" = String, Path, description = "Event type")
    ),
    responses(
        (status = 200, description = "Schedule deleted"),
        (status = 404, description = "Schedule not found"),
        (status = 500, description = "Internal error")
    ),
    tag = "Schedules"
)]
pub async fn delete_schedule(
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

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Health check result", body = HealthResponse)
    ),
    tag = "Status"
)]
pub async fn healthcheck(State(state): State<ApiState>) -> Json<HealthResponse> {
    let warnings = state.store.get_warnings().await;
    Json(HealthResponse {
        healthy: warnings.is_empty(),
        warnings,
    })
}

#[utoipa::path(
    get,
    path = "/config",
    responses(
        (status = 200, description = "Current configuration", body = ConfigResponse)
    ),
    tag = "Config"
)]
pub async fn get_config(State(state): State<ApiState>) -> Json<ConfigResponse> {
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

#[utoipa::path(
    put,
    path = "/config",
    request_body = UpdateConfigRequest,
    responses(
        (status = 200, description = "Configuration updated", body = ConfigResponse),
        (status = 400, description = "Invalid configuration value"),
        (status = 500, description = "Internal error")
    ),
    tag = "Config"
)]
pub async fn update_config(
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

#[utoipa::path(
    post,
    path = "/events",
    request_body = EventRequest,
    responses(
        (status = 200, description = "Event queued", body = EventResponse),
        (status = 500, description = "Failed to queue event")
    ),
    tag = "Events"
)]
pub async fn trigger_event(
    State(state): State<ApiState>,
    Json(request): Json<EventRequest>,
) -> Result<Json<EventResponse>, StatusCode> {
    let event = Event::new(request.event_type, request.context);
    info!("HTTP producing event: {:?}", event.id);

    state
        .sender
        .send(event.clone())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(EventResponse {
        event,
        message: "Event queued".to_string(),
    }))
}

pub fn create_api_router(
    store: JobStore,
    timer_manager: TimerManager,
    schedule_manager: ScheduleManager,
    sender: EventSender,
) -> OpenApiRouter {
    let state = ApiState {
        store,
        timer_manager,
        schedule_manager,
        sender,
    };

    OpenApiRouter::new()
        .routes(routes!(get_status))
        .routes(routes!(healthcheck))
        .routes(routes!(get_jobs, get_job, cancel_job))
        .routes(routes!(
            get_handlers,
            create_handler,
            get_handler_by_type,
            update_handler,
            delete_handler
        ))
        .routes(routes!(
            get_timers,
            create_timer,
            get_timer_by_type,
            update_timer,
            delete_timer
        ))
        .routes(routes!(
            get_schedules,
            create_schedule,
            get_schedule_by_type,
            update_schedule,
            delete_schedule
        ))
        .routes(routes!(get_config, update_config))
        .routes(routes!(trigger_event))
        .routes(routes!(reload))
        .with_state(state)
}
