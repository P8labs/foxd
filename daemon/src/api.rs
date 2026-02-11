use axum::{
    extract::{Path, State},
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use rust_embed::Embed;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::daemon::Daemon;
use crate::db::Database;
use crate::errors::Result;
use crate::models::{
    Config, ConfigUpdateRequest, Device, DeviceNicknameRequest, DeviceStatus, DevicesResponse,
    ErrorResponse, LogsResponse, Metrics, Rule, RuleRequest, RulesResponse, SuccessResponse,
};

#[derive(Embed)]
#[folder = "../console/build/"]
struct ConsoleAssets;

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub config: Arc<RwLock<Config>>,
    pub daemon: Option<Arc<Daemon>>,
    pub start_time: Instant,
}

impl AppState {
    pub fn new(db: Database, config: Config, daemon: Option<Arc<Daemon>>) -> Self {
        Self {
            db,
            config: Arc::new(RwLock::new(config)),
            daemon,
            start_time: Instant::now(),
        }
    }
}

pub fn create_router(state: AppState) -> Router {
    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/devices", get(get_devices))
        .route("/devices/{mac}", get(get_device))
        .route("/devices/{mac}/nickname", post(update_device_nickname))
        .route("/rules", get(get_rules).post(create_rule))
        .route("/rules/{id}", get(get_rule).post(update_rule))
        .route("/rules/{id}/delete", post(delete_rule))
        .route("/config", get(get_config).post(update_config))
        .route("/metrics", get(get_metrics))
        .route("/logs", get(get_logs))
        .route("/restart", post(restart_daemon))
        .fallback(api_fallback)
        .with_state(state);

    Router::new()
        .nest("/api", api_routes)
        .fallback(serve_console)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}

async fn api_fallback(uri: Uri) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: format!("Not found: {}", uri.path()),
            details: None,
        }),
    )
        .into_response()
}

async fn serve_console(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if !path.is_empty() {
        if let Some(content) = ConsoleAssets::get(path) {
            let mime = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime)],
                content.data.to_vec(),
            )
                .into_response();
        }
    }

    let index_path = if path.is_empty() {
        "index.html".to_string()
    } else {
        format!("{}/index.html", path)
    };

    if let Some(content) = ConsoleAssets::get(&index_path) {
        return (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html".to_string())],
            content.data.to_vec(),
        )
            .into_response();
    }

    match ConsoleAssets::get("index.html") {
        Some(content) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html".to_string())],
            content.data.to_vec(),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            "Console not available. Build the console first: cd console && pnpm build",
        )
            .into_response(),
    }
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_all();

    let uptime_seconds = state.start_time.elapsed().as_secs();
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_usage_percent = if total_memory > 0 {
        (used_memory as f64 / total_memory as f64 * 100.0) as u32
    } else {
        0
    };

    let cpu_usage = sys.global_cpu_info().cpu_usage() as u32;

    Json(serde_json::json!({
        "status": "ok",
        "service": "foxd",
        "uptime_seconds": uptime_seconds,
        "system": {
            "cpu_usage_percent": cpu_usage,
            "memory_usage_percent": memory_usage_percent,
            "total_memory_mb": total_memory / 1024 / 1024,
            "used_memory_mb": used_memory / 1024 / 1024,
        }
    }))
}

async fn get_devices(State(state): State<AppState>) -> Result<Json<DevicesResponse>> {
    let devices = state.db.get_all_devices().await?;
    let count = devices.len();

    Ok(Json(DevicesResponse { devices, count }))
}

async fn get_device(
    State(state): State<AppState>,
    Path(mac): Path<String>,
) -> Result<Json<Device>> {
    let device =
        state.db.get_device_by_mac(&mac).await?.ok_or_else(|| {
            crate::errors::DaemonError::NotFound(format!("Device {} not found", mac))
        })?;

    Ok(Json(device))
}

async fn update_device_nickname(
    State(state): State<AppState>,
    Path(mac): Path<String>,
    Json(request): Json<DeviceNicknameRequest>,
) -> Result<Json<Device>> {
    let _device =
        state.db.get_device_by_mac(&mac).await?.ok_or_else(|| {
            crate::errors::DaemonError::NotFound(format!("Device {} not found", mac))
        })?;

    state
        .db
        .update_device_nickname(&mac, request.nickname)
        .await?;

    let updated_device = state.db.get_device_by_mac(&mac).await?.ok_or_else(|| {
        crate::errors::DaemonError::Internal("Failed to retrieve updated device".to_string())
    })?;

    info!(
        "Updated nickname for device {}: {:?}",
        mac, updated_device.nickname
    );

    Ok(Json(updated_device))
}

async fn get_rules(State(state): State<AppState>) -> Result<Json<RulesResponse>> {
    let rules = state.db.get_all_rules().await?;
    let count = rules.len();

    Ok(Json(RulesResponse { rules, count }))
}

async fn get_rule(State(state): State<AppState>, Path(id): Path<i64>) -> Result<Json<Rule>> {
    let rule =
        state.db.get_rule_by_id(id).await?.ok_or_else(|| {
            crate::errors::DaemonError::NotFound(format!("Rule {} not found", id))
        })?;

    Ok(Json(rule))
}

async fn create_rule(
    State(state): State<AppState>,
    Json(request): Json<RuleRequest>,
) -> Result<Json<Rule>> {
    let now = Utc::now();

    let rule = Rule {
        id: None,
        name: request.name,
        description: request.description,
        trigger_type: request.trigger_type,
        mac_filter: request.mac_filter,
        enabled: request.enabled,
        notification_channels: request.notification_channels,
        created_at: now,
        updated_at: now,
    };

    let id = state.db.create_rule(&rule).await?;

    let created_rule = state.db.get_rule_by_id(id).await?.ok_or_else(|| {
        crate::errors::DaemonError::Internal("Failed to retrieve created rule".to_string())
    })?;

    info!("Created rule: {} (id: {})", created_rule.name, id);

    Ok(Json(created_rule))
}

async fn update_rule(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<RuleRequest>,
) -> Result<Json<Rule>> {
    let existing =
        state.db.get_rule_by_id(id).await?.ok_or_else(|| {
            crate::errors::DaemonError::NotFound(format!("Rule {} not found", id))
        })?;

    let rule = Rule {
        id: Some(id),
        name: request.name,
        description: request.description,
        trigger_type: request.trigger_type,
        mac_filter: request.mac_filter,
        enabled: request.enabled,
        notification_channels: request.notification_channels,
        created_at: existing.created_at,
        updated_at: Utc::now(),
    };

    state.db.update_rule(id, &rule).await?;

    let updated_rule = state.db.get_rule_by_id(id).await?.ok_or_else(|| {
        crate::errors::DaemonError::Internal("Failed to retrieve updated rule".to_string())
    })?;

    info!("Updated rule: {} (id: {})", updated_rule.name, id);

    Ok(Json(updated_rule))
}

async fn delete_rule(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<SuccessResponse>> {
    let _ =
        state.db.get_rule_by_id(id).await?.ok_or_else(|| {
            crate::errors::DaemonError::NotFound(format!("Rule {} not found", id))
        })?;

    state.db.delete_rule(id).await?;

    info!("Deleted rule with id: {}", id);

    Ok(Json(SuccessResponse {
        message: format!("Rule {} deleted successfully", id),
    }))
}

async fn get_config(State(state): State<AppState>) -> Result<Json<Config>> {
    let config = state.config.read().await;
    Ok(Json(config.clone()))
}

async fn update_config(
    State(state): State<AppState>,
    Json(request): Json<ConfigUpdateRequest>,
) -> Result<Json<SuccessResponse>> {
    let mut config = state.config.write().await;

    if let Some(notifications) = request.notifications {
        config.notifications = notifications.clone();

        if let Some(daemon) = &state.daemon {
            let notifier = daemon.get_notifier();
            let mut notifier_guard = notifier.write().await;
            notifier_guard.update_channels(notifications);
        }

        info!("Updated notification channels");
    }

    if let Some(daemon_config) = request.daemon {
        config.daemon = daemon_config;
        info!("Updated daemon configuration (requires restart)");
    }

    Ok(Json(SuccessResponse {
        message: "Configuration updated successfully".to_string(),
    }))
}

async fn get_metrics(State(state): State<AppState>) -> Result<Json<Metrics>> {
    let total_devices = state.db.get_total_device_count().await?;
    let online_devices = state
        .db
        .get_device_count_by_status(DeviceStatus::Online)
        .await?;
    let offline_devices = state
        .db
        .get_device_count_by_status(DeviceStatus::Offline)
        .await?;
    let total_rules = state.db.get_total_rule_count().await?;
    let enabled_rules = state.db.get_enabled_rule_count().await?;

    let packets_captured = state
        .daemon
        .as_ref()
        .map(|d| d.get_packets_captured())
        .unwrap_or(0);

    let notifications_sent = if let Some(daemon) = &state.daemon {
        let notifier = daemon.get_notifier();
        let notifier_guard = notifier.read().await;
        notifier_guard.get_notifications_sent()
    } else {
        0
    };

    let uptime_seconds = state.start_time.elapsed().as_secs();

    Ok(Json(Metrics {
        total_devices,
        online_devices,
        offline_devices,
        total_rules,
        enabled_rules,
        packets_captured,
        notifications_sent,
        uptime_seconds,
    }))
}

async fn restart_daemon() -> Result<Json<SuccessResponse>> {
    info!("Restart requested via API");

    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_secs(1));
        std::process::exit(42);
    });

    Ok(Json(SuccessResponse {
        message: "Daemon restart initiated".to_string(),
    }))
}

async fn get_logs(State(state): State<AppState>) -> Result<Json<LogsResponse>> {
    let logs = state.db.get_logs(Some(200), None).await?;
    let count = logs.len();

    Ok(Json(LogsResponse { logs, count }))
}
