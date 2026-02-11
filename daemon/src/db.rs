use chrono::Utc;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::str::FromStr;
use tracing::info;

use crate::errors::{DaemonError, Result};
use crate::models::{Device, DeviceStatus, LogEntry, LogLevel, Rule, TriggerType};

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("Connecting to database: {}", database_url);

        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect_with(options)
            .await?;

        let db = Self { pool };
        db.run_migrations().await?;

        info!("Database connected and migrations applied");
        Ok(db)
    }

    async fn run_migrations(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS devices (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mac_address TEXT NOT NULL UNIQUE,
                ip_address TEXT,
                hostname TEXT,
                nickname TEXT,
                vendor TEXT,
                first_seen TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'unknown'
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_devices_mac ON devices(mac_address);
            CREATE INDEX IF NOT EXISTS idx_devices_status ON devices(status);
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS rules (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                description TEXT,
                trigger_type TEXT NOT NULL,
                mac_filter TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                notification_channels TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_rules_enabled ON rules(enabled);
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                level TEXT NOT NULL,
                category TEXT NOT NULL,
                message TEXT NOT NULL,
                details TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_logs_timestamp ON logs(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_logs_level ON logs(level);
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS notification_channels (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                channel_type TEXT NOT NULL,
                config TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_notification_channels_name ON notification_channels(name);
            "#,
        )
        .execute(&self.pool)
        .await?;

        info!("Database migrations completed");
        Ok(())
    }

    pub async fn upsert_device(&self, device: &Device) -> Result<i64> {
        let status_str = device.status.to_string();
        let first_seen = device.first_seen.to_rfc3339();
        let last_seen = device.last_seen.to_rfc3339();

        let result = sqlx::query(
            r#"
            INSERT INTO devices (mac_address, ip_address, hostname, nickname, vendor, first_seen, last_seen, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(mac_address) DO UPDATE SET
                ip_address = excluded.ip_address,
                hostname = excluded.hostname,
                vendor = excluded.vendor,
                last_seen = excluded.last_seen,
                status = excluded.status
            RETURNING id
            "#,
        )
        .bind(&device.mac_address)
        .bind(&device.ip_address)
        .bind(&device.hostname)
        .bind(&device.nickname)
        .bind(&device.vendor)
        .bind(&first_seen)
        .bind(&last_seen)
        .bind(&status_str)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get(0))
    }

    pub async fn get_device_by_mac(&self, mac: &str) -> Result<Option<Device>> {
        let row = sqlx::query(
            r#"
            SELECT id, mac_address, ip_address, hostname, nickname, vendor, first_seen, last_seen, status
            FROM devices
            WHERE mac_address = ?
            "#,
        )
        .bind(mac)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_device(row)?)),
            None => Ok(None),
        }
    }

    pub async fn get_all_devices(&self) -> Result<Vec<Device>> {
        let rows = sqlx::query(
            r#"
            SELECT id, mac_address, ip_address, hostname, nickname, vendor, first_seen, last_seen, status
            FROM devices
            ORDER BY last_seen DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| self.row_to_device(row))
            .collect()
    }

    pub async fn get_devices_by_status(&self, status: DeviceStatus) -> Result<Vec<Device>> {
        let status_str = status.to_string();
        let rows = sqlx::query(
            r#"
            SELECT id, mac_address, ip_address, hostname, nickname, vendor, first_seen, last_seen, status
            FROM devices
            WHERE status = ?
            ORDER BY last_seen DESC
            "#,
        )
        .bind(&status_str)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| self.row_to_device(row))
            .collect()
    }

    pub async fn update_device_status(&self, mac: &str, status: DeviceStatus) -> Result<()> {
        let status_str = status.to_string();
        let last_seen = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE devices
            SET status = ?, last_seen = ?
            WHERE mac_address = ?
            "#,
        )
        .bind(&status_str)
        .bind(&last_seen)
        .bind(mac)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_device_nickname(&self, mac: &str, nickname: Option<String>) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE devices
            SET nickname = ?
            WHERE mac_address = ?
            "#,
        )
        .bind(&nickname)
        .bind(mac)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    fn row_to_device(&self, row: sqlx::sqlite::SqliteRow) -> Result<Device> {
        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "online" => DeviceStatus::Online,
            "offline" => DeviceStatus::Offline,
            _ => DeviceStatus::Unknown,
        };

        let first_seen_str: String = row.get("first_seen");
        let last_seen_str: String = row.get("last_seen");

        Ok(Device {
            id: Some(row.get("id")),
            mac_address: row.get("mac_address"),
            ip_address: row.get("ip_address"),
            hostname: row.get("hostname"),
            nickname: row.get("nickname"),
            vendor: row.get("vendor"),
            first_seen: chrono::DateTime::parse_from_rfc3339(&first_seen_str)
                .map_err(|e| DaemonError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
            last_seen: chrono::DateTime::parse_from_rfc3339(&last_seen_str)
                .map_err(|e| DaemonError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
            status,
        })
    }

    pub async fn create_rule(&self, rule: &Rule) -> Result<i64> {
        let trigger_type_str = rule.trigger_type.to_string();
        let channels_json = serde_json::to_string(&rule.notification_channels)?;
        let created_at = rule.created_at.to_rfc3339();
        let updated_at = rule.updated_at.to_rfc3339();

        let result = sqlx::query(
            r#"
            INSERT INTO rules (name, description, trigger_type, mac_filter, enabled, notification_channels, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(&trigger_type_str)
        .bind(&rule.mac_filter)
        .bind(rule.enabled)
        .bind(&channels_json)
        .bind(&created_at)
        .bind(&updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get(0))
    }

    pub async fn get_rule_by_id(&self, id: i64) -> Result<Option<Rule>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, trigger_type, mac_filter, enabled, notification_channels, created_at, updated_at
            FROM rules
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_rule(row)?)),
            None => Ok(None),
        }
    }

    pub async fn get_all_rules(&self) -> Result<Vec<Rule>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, trigger_type, mac_filter, enabled, notification_channels, created_at, updated_at
            FROM rules
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|row| self.row_to_rule(row)).collect()
    }

    pub async fn get_enabled_rules(&self) -> Result<Vec<Rule>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, trigger_type, mac_filter, enabled, notification_channels, created_at, updated_at
            FROM rules
            WHERE enabled = 1
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|row| self.row_to_rule(row)).collect()
    }

    pub async fn update_rule(&self, id: i64, rule: &Rule) -> Result<()> {
        let trigger_type_str = rule.trigger_type.to_string();
        let channels_json = serde_json::to_string(&rule.notification_channels)?;
        let updated_at = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE rules
            SET name = ?, description = ?, trigger_type = ?, mac_filter = ?, enabled = ?, notification_channels = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(&trigger_type_str)
        .bind(&rule.mac_filter)
        .bind(rule.enabled)
        .bind(&channels_json)
        .bind(&updated_at)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_rule(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM rules WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    fn row_to_rule(&self, row: sqlx::sqlite::SqliteRow) -> Result<Rule> {
        let trigger_type_str: String = row.get("trigger_type");
        let trigger_type = match trigger_type_str.as_str() {
            "device_connected" => TriggerType::DeviceConnected,
            "device_disconnected" => TriggerType::DeviceDisconnected,
            "new_device" => TriggerType::NewDevice,
            "device_status_change" => TriggerType::DeviceStatusChange,
            _ => {
                return Err(DaemonError::Database(sqlx::Error::Decode(Box::new(
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid trigger type"),
                ))))
            }
        };

        let channels_json: String = row.get("notification_channels");
        let notification_channels: Vec<String> = serde_json::from_str(&channels_json)?;

        let created_at_str: String = row.get("created_at");
        let updated_at_str: String = row.get("updated_at");

        Ok(Rule {
            id: Some(row.get("id")),
            name: row.get("name"),
            description: row.get("description"),
            trigger_type,
            mac_filter: row.get("mac_filter"),
            enabled: row.get("enabled"),
            notification_channels,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| DaemonError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| DaemonError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
        })
    }

    pub async fn get_device_count_by_status(&self, status: DeviceStatus) -> Result<i64> {
        let status_str = status.to_string();
        let row = sqlx::query("SELECT COUNT(*) as count FROM devices WHERE status = ?")
            .bind(&status_str)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get("count"))
    }

    pub async fn get_total_device_count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM devices")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get("count"))
    }

    pub async fn get_total_rule_count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM rules")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get("count"))
    }

    pub async fn get_enabled_rule_count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM rules WHERE enabled = 1")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get("count"))
    }

    // Log management functions
    pub async fn create_log(&self, entry: &LogEntry) -> Result<i64> {
        let timestamp = entry.timestamp.to_rfc3339();
        let level_str = entry.level.to_string();

        let result = sqlx::query(
            r#"
            INSERT INTO logs (timestamp, level, category, message, details)
            VALUES (?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(&timestamp)
        .bind(&level_str)
        .bind(&entry.category)
        .bind(&entry.message)
        .bind(&entry.details)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get(0))
    }

    pub async fn get_logs(
        &self,
        limit: Option<i64>,
        level: Option<LogLevel>,
    ) -> Result<Vec<LogEntry>> {
        let limit_value = limit.unwrap_or(100);

        let logs = if let Some(log_level) = level {
            let level_str = log_level.to_string();
            sqlx::query(
                r#"
                SELECT id, timestamp, level, category, message, details
                FROM logs
                WHERE level = ?
                ORDER BY timestamp DESC
                LIMIT ?
                "#,
            )
            .bind(&level_str)
            .bind(limit_value)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT id, timestamp, level, category, message, details
                FROM logs
                ORDER BY timestamp DESC
                LIMIT ?
                "#,
            )
            .bind(limit_value)
            .fetch_all(&self.pool)
            .await?
        };

        logs.into_iter().map(|row| self.row_to_log(row)).collect()
    }

    pub async fn clear_old_logs(&self, days: i64) -> Result<i64> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let cutoff_str = cutoff.to_rfc3339();

        let result = sqlx::query("DELETE FROM logs WHERE timestamp < ?")
            .bind(&cutoff_str)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as i64)
    }

    fn row_to_log(&self, row: sqlx::sqlite::SqliteRow) -> Result<LogEntry> {
        let level_str: String = row.get("level");
        let level = match level_str.as_str() {
            "info" => LogLevel::Info,
            "warning" => LogLevel::Warning,
            "error" => LogLevel::Error,
            "debug" => LogLevel::Debug,
            _ => LogLevel::Info,
        };

        let timestamp_str: String = row.get("timestamp");

        Ok(LogEntry {
            id: Some(row.get("id")),
            timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                .map_err(|e| DaemonError::Database(sqlx::Error::Decode(Box::new(e))))?
                .with_timezone(&Utc),
            level,
            category: row.get("category"),
            message: row.get("message"),
            details: row.get("details"),
        })
    }

    // Notification Channels
    pub async fn create_notification_channel(
        &self,
        channel: &crate::models::NotificationChannel,
    ) -> Result<i64> {
        let name = channel.name();
        let channel_type = match channel {
            crate::models::NotificationChannel::Telegram { .. } => "telegram",
            crate::models::NotificationChannel::Ntfy { .. } => "ntfy",
            crate::models::NotificationChannel::Webhook { .. } => "webhook",
        };
        let config = serde_json::to_string(channel)
            .map_err(|e| DaemonError::Internal(format!("Failed to serialize channel: {}", e)))?;
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            "INSERT INTO notification_channels (name, channel_type, config, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&name)
        .bind(channel_type)
        .bind(&config)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn get_notification_channel_by_id(
        &self,
        id: i64,
    ) -> Result<Option<crate::models::NotificationChannelWithId>> {
        let row = sqlx::query("SELECT id, config FROM notification_channels WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let config: String = row.get("config");
            let channel: crate::models::NotificationChannel = serde_json::from_str(&config)
                .map_err(|e| {
                    DaemonError::Internal(format!("Failed to deserialize channel: {}", e))
                })?;
            Ok(Some(crate::models::NotificationChannelWithId {
                id: row.get("id"),
                channel,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_all_notification_channels(
        &self,
    ) -> Result<Vec<crate::models::NotificationChannelWithId>> {
        let rows = sqlx::query("SELECT id, config FROM notification_channels ORDER BY created_at")
            .fetch_all(&self.pool)
            .await?;

        let mut channels = Vec::new();
        for row in rows {
            let config: String = row.get("config");
            let channel: crate::models::NotificationChannel = serde_json::from_str(&config)
                .map_err(|e| {
                    DaemonError::Internal(format!("Failed to deserialize channel: {}", e))
                })?;
            channels.push(crate::models::NotificationChannelWithId {
                id: row.get("id"),
                channel,
            });
        }

        Ok(channels)
    }

    pub async fn get_all_notification_channels_raw(
        &self,
    ) -> Result<Vec<crate::models::NotificationChannel>> {
        let rows = sqlx::query("SELECT config FROM notification_channels ORDER BY created_at")
            .fetch_all(&self.pool)
            .await?;

        let mut channels = Vec::new();
        for row in rows {
            let config: String = row.get("config");
            let channel: crate::models::NotificationChannel = serde_json::from_str(&config)
                .map_err(|e| {
                    DaemonError::Internal(format!("Failed to deserialize channel: {}", e))
                })?;
            channels.push(channel);
        }

        Ok(channels)
    }

    pub async fn update_notification_channel(
        &self,
        id: i64,
        channel: &crate::models::NotificationChannel,
    ) -> Result<()> {
        let name = channel.name();
        let channel_type = match channel {
            crate::models::NotificationChannel::Telegram { .. } => "telegram",
            crate::models::NotificationChannel::Ntfy { .. } => "ntfy",
            crate::models::NotificationChannel::Webhook { .. } => "webhook",
        };
        let config = serde_json::to_string(channel)
            .map_err(|e| DaemonError::Internal(format!("Failed to serialize channel: {}", e)))?;
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "UPDATE notification_channels
             SET name = ?, channel_type = ?, config = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(&name)
        .bind(channel_type)
        .bind(&config)
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_notification_channel(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM notification_channels WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
