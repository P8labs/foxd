mod api;
mod daemon;
mod db;
mod errors;
mod models;
mod notifier;

use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::{create_router, AppState};
use crate::daemon::Daemon;
use crate::db::Database;
use crate::errors::Result;
use crate::models::Config;
use crate::notifier::Notifier;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "foxd_daemon=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("foxd starting...");

    let config = load_config()?;
    info!("Configuration loaded from config.toml");

    let db_url = if config.database.path.starts_with("sqlite://") {
        config.database.path.clone()
    } else {
        format!("sqlite://{}", config.database.path)
    };
    let db = Database::new(&db_url).await?;
    info!("Database initialized");

    let notification_channels = db.get_all_notification_channels_raw().await?;
    let notifier = Notifier::new(notification_channels.clone());
    info!(
        "Notification service initialized with {} channels",
        notification_channels.len()
    );

    let daemon = Arc::new(Daemon::new(
        db.clone(),
        notifier,
        config.daemon.interface.clone(),
        config.daemon.device_timeout_secs,
        config.daemon.neighbor_check_interval_secs,
        config.daemon.log_cleanup_enabled,
        config.daemon.log_retention_days,
    ));

    let api_state = AppState::new(db, config.clone(), Some(Arc::clone(&daemon)));

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    let mut daemon_handle = {
        let daemon = Arc::clone(&daemon);
        tokio::spawn(async move {
            if let Err(e) = daemon.start().await {
                error!("Daemon error: {}", e);
            }
        })
    };

    let mut api_handle = tokio::spawn(async move {
        let addr = SocketAddr::from((
            config
                .api
                .host
                .parse::<std::net::IpAddr>()
                .unwrap_or([127, 0, 0, 1].into()),
            config.api.port,
        ));

        info!("Starting API server on {}", addr);

        let app = create_router(api_state);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("Failed to bind to address");

        info!("API server listening on {}", addr);

        let mut rx = shutdown_rx;
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                rx.changed().await.ok();
            })
            .await
            .expect("Server error");

        info!("API server stopped");
    });

    tokio::select! {
        _ = &mut daemon_handle => {
            error!("Daemon task ended unexpectedly");
        }
        _ = &mut api_handle => {
            error!("API server ended unexpectedly");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received, stopping...");
            // Reset SIGINT to default so next Ctrl+C force-kills via OS
            unsafe { libc::signal(libc::SIGINT, libc::SIG_DFL); }
        }
    }

    let _ = shutdown_tx.send(true);
    daemon_handle.abort();
    let _ = api_handle.await;

    info!("foxd stopped");
    Ok(())
}

fn load_config() -> Result<Config> {
    let config_path = std::env::var("FOXD_CONFIG").unwrap_or_else(|_| "config.toml".to_string());

    if std::path::Path::new(&config_path).exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| errors::DaemonError::Config(format!("Failed to parse config: {}", e)))?;
        Ok(config)
    } else {
        info!("Config file not found, using defaults");
        Ok(default_config())
    }
}

fn default_config() -> Config {
    Config {
        daemon: models::DaemonConfig {
            interface: std::env::var("FOXD_INTERFACE").unwrap_or_else(|_| "wlan0".to_string()),
            capture_filter: None,
            neighbor_check_interval_secs: 30,
            device_timeout_secs: 60,
            log_cleanup_enabled: true,
            log_retention_days: 30,
        },
        database: models::DatabaseConfig {
            path: std::env::var("FOXD_DB_PATH").unwrap_or_else(|_| "./foxd.db".to_string()),
        },
        api: models::ApiConfig {
            host: std::env::var("FOXD_API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("FOXD_API_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
        },
    }
}
