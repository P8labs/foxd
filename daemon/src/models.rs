use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "lowercase")]
pub enum DeviceStatus {
    Online,
    Offline,
    Unknown,
}

impl std::fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceStatus::Online => write!(f, "online"),
            DeviceStatus::Offline => write!(f, "offline"),
            DeviceStatus::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: Option<i64>,
    pub mac_address: String,
    pub ip_address: Option<String>,
    pub hostname: Option<String>,
    pub nickname: Option<String>,
    pub vendor: Option<String>,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub status: DeviceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "snake_case")]
pub enum TriggerType {
    DeviceConnected,
    DeviceDisconnected,
    NewDevice,
    DeviceStatusChange,
}

impl std::fmt::Display for TriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TriggerType::DeviceConnected => write!(f, "device_connected"),
            TriggerType::DeviceDisconnected => write!(f, "device_disconnected"),
            TriggerType::NewDevice => write!(f, "new_device"),
            TriggerType::DeviceStatusChange => write!(f, "device_status_change"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: TriggerType,
    pub mac_filter: Option<String>,
    pub enabled: bool,
    pub notification_channels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: TriggerType,
    pub mac_filter: Option<String>,
    pub enabled: bool,
    pub notification_channels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum NotificationChannel {
    Telegram {
        bot_token: String,
        chat_id: String,
    },
    Ntfy {
        server_url: String,
        topic: String,
        token: Option<String>,
    },
    Webhook {
        url: String,
        headers: Option<serde_json::Value>,
    },
}

impl NotificationChannel {
    pub fn name(&self) -> String {
        match self {
            NotificationChannel::Telegram { chat_id, .. } => format!("telegram_{}", chat_id),
            NotificationChannel::Ntfy { topic, .. } => format!("ntfy_{}", topic),
            NotificationChannel::Webhook { url, .. } => {
                format!("webhook_{}", url.split('/').last().unwrap_or("unknown"))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationChannelWithId {
    pub id: i64,
    pub channel: NotificationChannel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub daemon: DaemonConfig,
    pub database: DatabaseConfig,
    pub api: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub interface: String,
    pub capture_filter: Option<String>,
    pub neighbor_check_interval_secs: u64,
    pub device_timeout_secs: u64,
    pub log_cleanup_enabled: bool,
    pub log_retention_days: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigUpdateRequest {
    pub daemon: Option<DaemonConfig>,
}

#[derive(Debug, Serialize)]
pub struct NotificationChannelsResponse {
    pub channels: Vec<NotificationChannelWithId>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceNicknameRequest {
    pub nickname: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NotificationEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: TriggerType,
    pub device: Device,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum NetworkEvent {
    ArpRequest {
        source_mac: String,
        source_ip: IpAddr,
    },
    ArpReply {
        source_mac: String,
        source_ip: IpAddr,
    },
    DhcpRequest {
        client_mac: String,
        requested_ip: Option<IpAddr>,
    },
    NeighborAdded {
        mac: String,
        ip: IpAddr,
        interface_index: u32,
    },
    NeighborRemoved {
        mac: String,
        ip: IpAddr,
        interface_index: u32,
    },
    NeighborUpdated {
        mac: String,
        ip: IpAddr,
        interface_index: u32,
    },
}

#[derive(Debug, Serialize)]
pub struct DevicesResponse {
    pub devices: Vec<Device>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct RulesResponse {
    pub rules: Vec<Rule>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Metrics {
    pub total_devices: i64,
    pub online_devices: i64,
    pub offline_devices: i64,
    pub total_rules: i64,
    pub enabled_rules: i64,
    pub packets_captured: u64,
    pub notifications_sent: u64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warning => write!(f, "warning"),
            LogLevel::Error => write!(f, "error"),
            LogLevel::Debug => write!(f, "debug"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: Option<i64>,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub category: String,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LogsResponse {
    pub logs: Vec<LogEntry>,
    pub count: usize,
}
