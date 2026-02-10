use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::daemon::Daemon;
use crate::db::Database;
use crate::errors::Result;
use crate::models::{
    Config, ConfigUpdateRequest, Device, DeviceStatus, DevicesResponse, Metrics, Rule, RuleRequest,
    RulesResponse, SuccessResponse,
};

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
    Router::new()
        .route("/health", get(health_check))
        .route("/devices", get(get_devices))
        .route("/devices/:mac", get(get_device))
        .route("/rules", get(get_rules))
        .route("/rules", post(create_rule))
        .route("/rules/:id", get(get_rule))
        .route("/rules/:id", post(update_rule))
        .route("/rules/:id/delete", post(delete_rule))
        .route("/config", get(get_config))
        .route("/config", post(update_config))
        .route("/metrics", get(get_metrics))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "foxd-daemon"
    }))
}

/// GET /devices - Get all devices
async fn get_devices(State(state): State<AppState>) -> Result<Json<DevicesResponse>> {
    let devices = state.db.get_all_devices().await?;
    let count = devices.len();

    Ok(Json(DevicesResponse { devices, count }))
}

/// GET /devices/:mac - Get specific device
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

/// GET /rules - Get all rules
async fn get_rules(State(state): State<AppState>) -> Result<Json<RulesResponse>> {
    let rules = state.db.get_all_rules().await?;
    let count = rules.len();

    Ok(Json(RulesResponse { rules, count }))
}

/// GET /rules/:id - Get specific rule
async fn get_rule(State(state): State<AppState>, Path(id): Path<i64>) -> Result<Json<Rule>> {
    let rule =
        state.db.get_rule_by_id(id).await?.ok_or_else(|| {
            crate::errors::DaemonError::NotFound(format!("Rule {} not found", id))
        })?;

    Ok(Json(rule))
}

/// POST /rules - Create new rule
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

/// POST /rules/:id - Update existing rule
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

/// POST /rules/:id/delete - Delete rule
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

/// GET /config - Get current configuration
async fn get_config(State(state): State<AppState>) -> Result<Json<Config>> {
    let config = state.config.read().await;
    Ok(Json(config.clone()))
}

/// POST /config - Update configuration
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

/// GET /metrics - Get system metrics
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
