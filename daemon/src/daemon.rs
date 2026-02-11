use async_channel::{Receiver, Sender};
use chrono::Utc;
use pcap::{Capture, Device as PcapDevice};
use pnet::packet::arp::ArpPacket;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::Packet;
use rtnetlink::new_connection;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::db::Database;
use crate::errors::{DaemonError, Result};
use crate::models::{Device, DeviceStatus, NetworkEvent, NotificationEvent, Rule, TriggerType};
use crate::notifier::Notifier;

pub struct Daemon {
    db: Database,
    notifier: Arc<RwLock<Notifier>>,
    interface: String,
    device_timeout: Duration,
    neighbor_check_interval: Duration,
    packets_captured: Arc<std::sync::atomic::AtomicU64>,
    log_cleanup_enabled: bool,
    log_retention_days: i64,
}

impl Daemon {
    pub fn new(
        db: Database,
        notifier: Notifier,
        interface: String,
        device_timeout_secs: u64,
        neighbor_check_interval_secs: u64,
        log_cleanup_enabled: bool,
        log_retention_days: u64,
    ) -> Self {
        Self {
            db,
            notifier: Arc::new(RwLock::new(notifier)),
            interface,
            device_timeout: Duration::from_secs(device_timeout_secs),
            neighbor_check_interval: Duration::from_secs(neighbor_check_interval_secs),
            packets_captured: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            log_cleanup_enabled,
            log_retention_days: log_retention_days as i64,
        }
    }

    pub async fn start(self: Arc<Self>) -> Result<()> {
        info!("Starting Fox daemon on interface: {}", self.interface);

        let (event_tx, event_rx) = async_channel::bounded(100);

        let capture_handle = {
            let daemon = Arc::clone(&self);
            let tx = event_tx.clone();
            tokio::spawn(async move {
                if let Err(e) = daemon.capture_packets(tx).await {
                    error!("Packet capture error: {}", e);
                }
            })
        };

        let netlink_handle = {
            let daemon = Arc::clone(&self);
            let tx = event_tx.clone();
            tokio::spawn(async move {
                if let Err(e) = daemon.monitor_netlink(tx).await {
                    error!("Netlink monitoring error: {}", e);
                }
            })
        };

        let processor_handle = {
            let daemon = Arc::clone(&self);
            tokio::spawn(async move {
                daemon.process_events(event_rx).await;
            })
        };

        let timeout_handle = {
            let daemon = Arc::clone(&self);
            tokio::spawn(async move {
                daemon.check_device_timeouts().await;
            })
        };

        let log_cleanup_handle = {
            let daemon = Arc::clone(&self);
            tokio::spawn(async move {
                daemon.cleanup_old_logs().await;
            })
        };

        tokio::select! {
            _ = capture_handle => warn!("Packet capture task ended"),
            _ = netlink_handle => warn!("Netlink monitoring task ended"),
            _ = processor_handle => warn!("Event processor task ended"),
            _ = timeout_handle => warn!("Timeout checker task ended"),
            _ = log_cleanup_handle => warn!("Log cleanup task ended"),
        }

        Ok(())
    }

    async fn capture_packets(&self, tx: Sender<NetworkEvent>) -> Result<()> {
        info!("Starting packet capture on {}", self.interface);

        let device = PcapDevice::list()?
            .into_iter()
            .find(|d| d.name == self.interface)
            .ok_or_else(|| {
                DaemonError::PacketCapture(format!("Interface {} not found", self.interface))
            })?;

        let mut cap = Capture::from_device(device)?
            .promisc(true)
            .timeout(1000)
            .open()?;

        cap.filter("arp or (udp port 67 or udp port 68)", true)?;

        info!("Packet capture started, filter applied");

        let packets_captured = Arc::clone(&self.packets_captured);
        let tx_clone = tx.clone();

        tokio::task::spawn_blocking(move || {
            while let Ok(packet) = cap.next_packet() {
                packets_captured.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                if let Some(ethernet) = EthernetPacket::new(packet.data) {
                    match ethernet.get_ethertype() {
                        EtherTypes::Arp => {
                            if let Some(arp) = ArpPacket::new(ethernet.payload()) {
                                let event = Self::parse_arp_packet(&arp);
                                if let Some(event) = event {
                                    let _ = tx_clone.try_send(event);
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        })
        .await
        .map_err(|e| DaemonError::Internal(format!("Capture task error: {}", e)))?;

        Ok(())
    }

    fn parse_arp_packet(arp: &ArpPacket) -> Option<NetworkEvent> {
        let source_mac = format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            arp.get_sender_hw_addr().0,
            arp.get_sender_hw_addr().1,
            arp.get_sender_hw_addr().2,
            arp.get_sender_hw_addr().3,
            arp.get_sender_hw_addr().4,
            arp.get_sender_hw_addr().5
        );

        let source_ip = IpAddr::from(arp.get_sender_proto_addr());

        let operation = arp.get_operation();

        match operation.0 {
            1 => Some(NetworkEvent::ArpRequest {
                source_mac,
                source_ip,
            }),
            2 => Some(NetworkEvent::ArpReply {
                source_mac,
                source_ip,
            }),
            _ => None,
        }
    }

    async fn monitor_netlink(&self, _tx: Sender<NetworkEvent>) -> Result<()> {
        info!("Starting netlink neighbor monitoring");

        let (connection, handle, _messages) = new_connection()?;
        tokio::spawn(connection);

        let mut check_interval = interval(self.neighbor_check_interval);

        loop {
            check_interval.tick().await;

            let mut neighbor_stream = handle.neighbours().get().execute();
            use futures::stream::TryStreamExt;

            while let Some(msg) = neighbor_stream.try_next().await.ok().flatten() {
                debug!("Neighbor table entry: ifindex={}", msg.header.ifindex);
                // Further parsing can be added when needed
            }
        }
    }

    async fn process_events(&self, rx: Receiver<NetworkEvent>) {
        info!("Event processor started");

        while let Ok(event) = rx.recv().await {
            if let Err(e) = self.handle_event(event).await {
                error!("Error handling event: {}", e);
            }
        }
    }

    async fn handle_event(&self, event: NetworkEvent) -> Result<()> {
        match event {
            NetworkEvent::ArpRequest {
                source_mac,
                source_ip,
                ..
            }
            | NetworkEvent::ArpReply {
                source_mac,
                source_ip,
            } => {
                self.handle_device_activity(&source_mac, Some(source_ip))
                    .await?;
            }
            NetworkEvent::DhcpRequest {
                client_mac,
                requested_ip,
            } => {
                self.handle_device_activity(&client_mac, requested_ip)
                    .await?;
            }
            NetworkEvent::NeighborAdded { mac, ip, .. }
            | NetworkEvent::NeighborUpdated { mac, ip, .. } => {
                self.handle_device_activity(&mac, Some(ip)).await?;
            }
            NetworkEvent::NeighborRemoved { mac, .. } => {
                self.handle_device_disconnection(&mac).await?;
            }
        }

        Ok(())
    }

    async fn handle_device_activity(&self, mac: &str, ip: Option<IpAddr>) -> Result<()> {
        let now = Utc::now();

        let existing_device = self.db.get_device_by_mac(mac).await?;

        let is_new = existing_device.is_none();
        let old_status = existing_device
            .as_ref()
            .map(|d| d.status.clone())
            .unwrap_or(DeviceStatus::Unknown);

        let device = Device {
            id: existing_device.as_ref().and_then(|d| d.id),
            mac_address: mac.to_string(),
            ip_address: ip.map(|i| i.to_string()),
            hostname: None,
            nickname: existing_device.as_ref().and_then(|d| d.nickname.clone()),
            vendor: None,
            first_seen: existing_device
                .as_ref()
                .map(|d| d.first_seen)
                .unwrap_or(now),
            last_seen: now,
            status: DeviceStatus::Online,
        };

        self.db.upsert_device(&device).await?;
        debug!(
            "Device activity: {} ({})",
            mac,
            ip.map(|i| i.to_string())
                .unwrap_or_else(|| "no IP".to_string())
        );

        // Log device activity
        if is_new {
            let log_entry = crate::models::LogEntry {
                id: None,
                timestamp: now,
                level: crate::models::LogLevel::Info,
                category: "device".to_string(),
                message: format!("New device discovered: {}", mac),
                details: ip.map(|i| i.to_string()),
            };
            let _ = self.db.create_log(&log_entry).await;
        } else if old_status != DeviceStatus::Online && device.status == DeviceStatus::Online {
            let log_entry = crate::models::LogEntry {
                id: None,
                timestamp: now,
                level: crate::models::LogLevel::Info,
                category: "device".to_string(),
                message: format!("Device connected: {}", mac),
                details: ip.map(|i| i.to_string()),
            };
            let _ = self.db.create_log(&log_entry).await;
        }

        let rules = self.db.get_enabled_rules().await?;

        for rule in rules {
            if let Some(filter) = &rule.mac_filter {
                if !mac.eq_ignore_ascii_case(filter) {
                    continue;
                }
            }

            let should_notify = match rule.trigger_type {
                TriggerType::NewDevice => is_new,
                TriggerType::DeviceConnected => {
                    old_status != DeviceStatus::Online && device.status == DeviceStatus::Online
                }
                TriggerType::DeviceStatusChange => old_status != device.status,
                _ => false,
            };

            if should_notify {
                self.send_notification(&rule, &device).await?;
            }
        }

        Ok(())
    }

    async fn handle_device_disconnection(&self, mac: &str) -> Result<()> {
        if let Some(device) = self.db.get_device_by_mac(mac).await? {
            if device.status != DeviceStatus::Offline {
                self.db
                    .update_device_status(mac, DeviceStatus::Offline)
                    .await?;

                debug!("Device disconnected: {}", mac);

                // Log disconnection
                let log_entry = crate::models::LogEntry {
                    id: None,
                    timestamp: Utc::now(),
                    level: crate::models::LogLevel::Warning,
                    category: "device".to_string(),
                    message: format!("Device disconnected: {}", mac),
                    details: device.ip_address.clone(),
                };
                let _ = self.db.create_log(&log_entry).await;

                let rules = self.db.get_enabled_rules().await?;

                for rule in rules {
                    if let Some(filter) = &rule.mac_filter {
                        if !mac.eq_ignore_ascii_case(filter) {
                            continue;
                        }
                    }

                    if rule.trigger_type == TriggerType::DeviceDisconnected
                        || rule.trigger_type == TriggerType::DeviceStatusChange
                    {
                        let mut updated_device = device.clone();
                        updated_device.status = DeviceStatus::Offline;
                        self.send_notification(&rule, &updated_device).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn check_device_timeouts(&self) {
        let mut check_interval = interval(self.neighbor_check_interval);

        loop {
            check_interval.tick().await;

            if let Err(e) = self.check_timeouts().await {
                error!("Error checking device timeouts: {}", e);
            }
        }
    }

    async fn check_timeouts(&self) -> Result<()> {
        let devices = self.db.get_devices_by_status(DeviceStatus::Online).await?;
        let now = Utc::now();

        for device in devices {
            let time_since_seen = now.signed_duration_since(device.last_seen);

            if time_since_seen.num_seconds() as u64 > self.device_timeout.as_secs() {
                info!(
                    "Device {} timed out (last seen {} seconds ago)",
                    device.mac_address,
                    time_since_seen.num_seconds()
                );
                self.handle_device_disconnection(&device.mac_address)
                    .await?;
            }
        }

        Ok(())
    }

    async fn send_notification(&self, rule: &Rule, device: &Device) -> Result<()> {
        let event = NotificationEvent {
            timestamp: Utc::now(),
            event_type: rule.trigger_type.clone(),
            device: device.clone(),
            message: format!(
                "Rule '{}' triggered for device {}",
                rule.name, device.mac_address
            ),
        };

        let notifier = self.notifier.read().await;
        notifier.send(&event, &rule.notification_channels).await?;

        Ok(())
    }

    async fn cleanup_old_logs(&self) {
        let mut cleanup_interval = interval(Duration::from_secs(86400));

        loop {
            cleanup_interval.tick().await;

            if !self.log_cleanup_enabled {
                continue;
            }

            match self.db.clear_old_logs(self.log_retention_days).await {
                Ok(deleted_count) => {
                    if deleted_count > 0 {
                        info!(
                            "Cleaned up {} old log entries (retention: {} days)",
                            deleted_count, self.log_retention_days
                        );
                    }
                }
                Err(e) => {
                    error!("Error cleaning up old logs: {}", e);
                }
            }
        }
    }

    pub fn get_notifier(&self) -> Arc<RwLock<Notifier>> {
        Arc::clone(&self.notifier)
    }

    pub fn get_packets_captured(&self) -> u64 {
        self.packets_captured
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}
