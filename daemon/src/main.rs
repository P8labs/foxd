mod api;
mod daemon;
mod db;
mod errors;
mod models;
mod notifier;

use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::{AppState, create_router};
use crate::daemon::Daemon;
use crate::db::Database;
use crate::errors::{DaemonError, Result};
use crate::models::Config;
use crate::notifier::Notifier;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "foxd=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("{} v{} starting", NAME, VERSION);

    let config = load_config().map_err(|e| {
        error!("Failed to load configuration: {}", e);
        e
    })?;

    let db_url = if config.database.path.starts_with("sqlite://") {
        config.database.path.clone()
    } else {
        format!("sqlite://{}", config.database.path)
    };

    let db = Database::new(&db_url).await.map_err(|e| {
        error!(
            "Failed to initialize database at '{}': {}",
            config.database.path, e
        );
        e
    })?;
    info!("Database ready at {}", config.database.path);

    let notification_channels = db.get_all_notification_channels_raw().await.map_err(|e| {
        error!("Failed to load notification channels: {}", e);
        e
    })?;
    let notifier = Notifier::new(notification_channels.clone());
    info!(
        "Notifier ready ({} channel(s) configured)",
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

    let addr = SocketAddr::from((
        config
            .api
            .host
            .parse::<std::net::IpAddr>()
            .unwrap_or([127, 0, 0, 1].into()),
        config.api.port,
    ));
    let web_url = format_web_url(&addr);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    let mut daemon_handle = {
        let daemon = Arc::clone(&daemon);
        tokio::spawn(async move { daemon.start().await })
    };

    let mut api_handle = {
        let web_url = web_url.clone();
        tokio::spawn(async move {
            let app = create_router(api_state);
            let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
                error!("Failed to bind API server to {}: {}", addr, e);
                DaemonError::Io(e)
            })?;
            info!("API server listening on {}", web_url);
            let mut rx = shutdown_rx;
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    rx.changed().await.ok();
                })
                .await
                .map_err(|e| {
                    error!("API server error: {}", e);
                    DaemonError::Io(e)
                })?;
            info!("API server stopped");
            Ok::<(), DaemonError>(())
        })
    };

    info!(
        "{} v{} ready — interface: {}, web: {}",
        NAME, VERSION, config.daemon.interface, web_url
    );

    tokio::select! {
        result = &mut daemon_handle => {
            match result {
                Ok(Ok(())) => warn!("Daemon task ended unexpectedly"),
                Ok(Err(e)) => error!("Daemon error: {}", e),
                Err(e) => error!("Daemon task panicked: {}", e),
            }
        }
        result = &mut api_handle => {
            match result {
                Ok(Ok(())) => warn!("API server ended unexpectedly"),
                Ok(Err(e)) => error!("API server error: {}", e),
                Err(e) => error!("API server task panicked: {}", e),
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received, stopping...");
            // Reset SIGINT to default so the next Ctrl-C force-kills the process.
            unsafe { libc::signal(libc::SIGINT, libc::SIG_DFL); }
        }
    }

    let _ = shutdown_tx.send(true);
    daemon_handle.abort();
    let _ = api_handle.await;

    info!("{} stopped", NAME);
    Ok(())
}

fn format_web_url(addr: &SocketAddr) -> String {
    match addr {
        SocketAddr::V4(a) if a.ip().is_unspecified() => {
            format!("http://localhost:{}", a.port())
        }
        SocketAddr::V6(a) if a.ip().is_unspecified() => {
            format!("http://localhost:{}", a.port())
        }
        SocketAddr::V4(a) => format!("http://{}:{}", a.ip(), a.port()),
        SocketAddr::V6(a) => format!("http://[{}]:{}", a.ip(), a.port()),
    }
}

fn load_config() -> Result<Config> {
    let config_path = std::env::var("FOXD_CONFIG").unwrap_or_else(|_| "config.toml".to_string());

    if std::path::Path::new(&config_path).exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| errors::DaemonError::Config(format!("Failed to parse config: {}", e)))?;
        Ok(config)
    } else {
        info!("Config file not found at '{}', using defaults", config_path);
        Ok(default_config())
    }
}

fn default_config() -> Config {
    Config {
        daemon: models::DaemonConfig {
            interface: std::env::var("INTERFACE").unwrap_or_else(|_| "wlan0".to_string()),
            capture_filter: None,
            neighbor_check_interval_secs: 30,
            device_timeout_secs: 60,
            log_cleanup_enabled: true,
            log_retention_days: 30,
        },
        database: models::DatabaseConfig {
            path: std::env::var("DB_PATH").unwrap_or_else(|_| "./foxd.db".to_string()),
        },
        api: models::ApiConfig {
            host: std::env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
        },
    }
}
