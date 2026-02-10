use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use tracing::{error, info, warn};

use crate::errors::{DaemonError, Result};
use crate::models::{NotificationChannel, NotificationEvent};

#[derive(Clone)]
pub struct Notifier {
    channels: HashMap<String, NotificationChannel>,
    client: Client,
    notifications_sent: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl Notifier {
    pub fn new(channels: Vec<NotificationChannel>) -> Self {
        let mut channel_map = HashMap::new();
        for channel in channels {
            channel_map.insert(channel.name(), channel);
        }

        Self {
            channels: channel_map,
            client: Client::new(),
            notifications_sent: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub fn update_channels(&mut self, channels: Vec<NotificationChannel>) {
        self.channels.clear();
        for channel in channels {
            self.channels.insert(channel.name(), channel);
        }
        info!("Updated notification channels: {}", self.channels.len());
    }

    pub async fn send(&self, event: &NotificationEvent, channel_names: &[String]) -> Result<()> {
        for channel_name in channel_names {
            if let Some(channel) = self.channels.get(channel_name) {
                if let Err(e) = self.send_to_channel(event, channel).await {
                    error!("Failed to send notification to {}: {}", channel_name, e);
                } else {
                    self.notifications_sent
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    info!("Notification sent to {}", channel_name);
                }
            } else {
                warn!("Notification channel not found: {}", channel_name);
            }
        }
        Ok(())
    }

    async fn send_to_channel(
        &self,
        event: &NotificationEvent,
        channel: &NotificationChannel,
    ) -> Result<()> {
        match channel {
            NotificationChannel::Telegram { bot_token, chat_id } => {
                self.send_telegram(event, bot_token, chat_id).await
            }
            NotificationChannel::Ntfy {
                server_url,
                topic,
                token,
            } => {
                self.send_ntfy(event, server_url, topic, token.as_deref())
                    .await
            }
            NotificationChannel::Webhook { url, headers } => {
                self.send_webhook(event, url, headers.as_ref()).await
            }
        }
    }

    async fn send_telegram(
        &self,
        event: &NotificationEvent,
        bot_token: &str,
        chat_id: &str,
    ) -> Result<()> {
        let url = format!("https://api.telegram.org/bot{}/sendMessage", bot_token);

        let message = format!(
            "🦊 <b>Fox Daemon Alert</b>\n\n\
             <b>Event:</b> {}\n\
             <b>Device:</b> {}\n\
             <b>IP:</b> {}\n\
             <b>MAC:</b> {}\n\
             <b>Status:</b> {}\n\
             <b>Time:</b> {}\n\n\
             {}",
            match event.event_type {
                crate::models::TriggerType::DeviceConnected => "Device Connected",
                crate::models::TriggerType::DeviceDisconnected => "Device Disconnected",
                crate::models::TriggerType::NewDevice => "New Device Discovered",
                crate::models::TriggerType::DeviceStatusChange => "Device Status Changed",
            },
            event.device.hostname.as_deref().unwrap_or("Unknown"),
            event.device.ip_address.as_deref().unwrap_or("Unknown"),
            event.device.mac_address,
            event.device.status,
            event.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            event.message
        );

        let payload = json!({
            "chat_id": chat_id,
            "text": message,
            "parse_mode": "HTML"
        });

        let response = self.client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(DaemonError::Notification(format!(
                "Telegram API error: {}",
                error_text
            )));
        }

        Ok(())
    }

    async fn send_ntfy(
        &self,
        event: &NotificationEvent,
        server_url: &str,
        topic: &str,
        token: Option<&str>,
    ) -> Result<()> {
        let url = format!("{}/{}", server_url.trim_end_matches('/'), topic);

        let title = match event.event_type {
            crate::models::TriggerType::DeviceConnected => "Device Connected",
            crate::models::TriggerType::DeviceDisconnected => "Device Disconnected",
            crate::models::TriggerType::NewDevice => "New Device",
            crate::models::TriggerType::DeviceStatusChange => "Status Changed",
        };

        let message = format!(
            "{}\nMAC: {}\nIP: {}\nStatus: {}",
            event.device.hostname.as_deref().unwrap_or("Unknown"),
            event.device.mac_address,
            event.device.ip_address.as_deref().unwrap_or("Unknown"),
            event.device.status
        );

        let mut request = self
            .client
            .post(&url)
            .header("Title", title)
            .header("Priority", "default")
            .header("Tags", "fox,network");

        if let Some(token) = token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }

        let response = request.body(message).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(DaemonError::Notification(format!(
                "Ntfy error: {}",
                error_text
            )));
        }

        Ok(())
    }

    async fn send_webhook(
        &self,
        event: &NotificationEvent,
        url: &str,
        headers: Option<&serde_json::Value>,
    ) -> Result<()> {
        let payload = json!({
            "timestamp": event.timestamp,
            "event_type": event.event_type,
            "device": {
                "mac_address": event.device.mac_address,
                "ip_address": event.device.ip_address,
                "hostname": event.device.hostname,
                "status": event.device.status,
                "last_seen": event.device.last_seen
            },
            "message": event.message
        });

        let mut request = self.client.post(url).json(&payload);

        if let Some(headers_obj) = headers {
            if let Some(headers_map) = headers_obj.as_object() {
                for (key, value) in headers_map {
                    if let Some(value_str) = value.as_str() {
                        request = request.header(key, value_str);
                    }
                }
            }
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(DaemonError::Notification(format!(
                "Webhook error: {}",
                error_text
            )));
        }

        Ok(())
    }

    pub fn get_notifications_sent(&self) -> u64 {
        self.notifications_sent
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}
