use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::models::ErrorResponse;

#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Packet capture error: {0}")]
    PacketCapture(String),

    #[error("Netlink error: {0}")]
    Netlink(String),

    #[error("Notification error: {0}")]
    Notification(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<config::ConfigError> for DaemonError {
    fn from(err: config::ConfigError) -> Self {
        DaemonError::Config(err.to_string())
    }
}

impl From<pcap::Error> for DaemonError {
    fn from(err: pcap::Error) -> Self {
        DaemonError::PacketCapture(err.to_string())
    }
}

impl From<rtnetlink::Error> for DaemonError {
    fn from(err: rtnetlink::Error) -> Self {
        DaemonError::Netlink(err.to_string())
    }
}

impl IntoResponse for DaemonError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            DaemonError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            DaemonError::Config(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            DaemonError::Network(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            DaemonError::PacketCapture(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            DaemonError::Netlink(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            DaemonError::Notification(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            DaemonError::Http(_) => (StatusCode::BAD_GATEWAY, self.to_string()),
            DaemonError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            DaemonError::Serialization(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            DaemonError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            DaemonError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            DaemonError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(ErrorResponse {
            error: error_message,
            details: None,
        });

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, DaemonError>;
