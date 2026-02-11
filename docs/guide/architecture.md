# Architecture

## Overview

foxd is a self-contained Rust daemon designed to passively monitor local area networks. It combines packet capture, kernel netlink monitoring, and event-driven notification delivery into a single binary with an embedded web console.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         foxd Daemon                          │
│                                                              │
│  ┌────────────────────┐      ┌───────────────────────────┐ │
│  │   Packet Capture   │      │   Netlink Monitor         │ │
│  │   (libpcap)        │      │   (Linux Neighbor Table)  │ │
│  │                    │      │                           │ │
│  │  • ARP packets     │      │  • Neighbor added events  │ │
│  │  • DHCP packets    │      │  • Neighbor removed       │ │
│  │  • BPF filtering   │      │  • Neighbor updates       │ │
│  └─────────┬──────────┘      └──────────┬────────────────┘ │
│            │                            │                   │
│            └────────┬───────────────────┘                   │
│                     │                                       │
│            ┌────────▼────────────┐                          │
│            │   Event Processor   │                          │
│            │                     │                          │
│            │  • Device discovery │                          │
│            │  • State management │                          │
│            │  • Event correlation│                          │
│            └────────┬────────────┘                          │
│                     │                                       │
│         ┌───────────┴───────────┐                           │
│         │                       │                           │
│   ┌─────▼──────┐        ┌──────▼──────┐                    │
│   │  Database  │        │  Rule Engine │                    │
│   │  (SQLite)  │        │              │                    │
│   │            │        │  • Match     │                    │
│   │  • Devices │        │  • Evaluate  │                    │
│   │  • Rules   │        │  • Trigger   │                    │
│   │  • Logs    │        └──────┬───────┘                    │
│   │  • Channels│               │                            │
│   └─────┬──────┘               │                            │
│         │               ┌──────▼────────┐                   │
│         │               │   Notifier    │                   │
│         │               │               │                   │
│         │               │  • Telegram   │                   │
│         │               │  • ntfy       │                   │
│         │               │  • Webhook    │                   │
│         │               └───────────────┘                   │
│         │                                                   │
│   ┌─────▼──────────────────────────────────┐               │
│   │          REST API & Web Console         │               │
│   │                                         │               │
│   │  • /api/* - JSON endpoints              │               │
│   │  • /* - Embedded SvelteKit console      │               │
│   └─────────────────────────────────────────┘               │
└─────────────────────────────────────────────────────────────┘
```

## Components

### 1. Packet Capture

**Purpose:** Passively observe network traffic to detect device activity.

**Technology:** libpcap (via Rust bindings)

**Operation:**

- Captures packets on specified network interface
- Applies BPF filter (typically ARP and DHCP)
- Extracts MAC addresses, IP addresses, and hostnames
- Forwards events to the event processor

### 2. Netlink Monitor

**Purpose:** Track Linux kernel's neighbor table for device state changes.

**Technology:** rtnetlink (Rust netlink library)

**Operation:**

- Subscribes to kernel neighbor table events
- Receives notifications when devices are added, removed, or updated
- Provides definitive state information for device online/offline status
- Runs in parallel with packet capture

**Why Both?**

- Packet capture detects activity and discovers new devices quickly
- Netlink provides authoritative state from the kernel's perspective
- Combined approach reduces false positives and missed events

### 3. Event Processor

**Purpose:** Correlate events and manage device state.

**Operation:**

- Receives events from both capture and netlink monitors
- Maintains in-memory device state
- Detects first-time device discovery (new devices)
- Tracks online/offline transitions
- Updates database with current state
- Emits high-level events (connected, disconnected, new device)

**State Machine per Device:**

```
     ┌─────────┐
     │ Unknown │
     └────┬────┘
          │ First packet/neighbor entry
          ▼
     ┌─────────┐
     │  Online │◄──┐
     └────┬────┘   │
          │        │ Activity seen
          │ Timeout│
          ▼        │
    ┌──────────┐  │
    │ Offline  │──┘
    └──────────┘
```

### 4. Database (SQLite)

**Purpose:** Persistent storage for all daemon data.

**Schema:**

- `devices` - Discovered devices with MAC, IP, hostname, status
- `rules` - Notification rules and triggers
- `notification_channels` - Channel configurations
- `logs` - Structured log entries

**Why SQLite?**

- Zero configuration
- Single file storage
- Embedded in-process
- ACID guarantees
- No separate server needed

### 5. Rule Engine

**Purpose:** Evaluate events against user-defined rules and trigger notifications.

**Operation:**

- Receives high-level events (new_device, device_connected, etc.)
- Loads active rules from database
- Evaluates each rule against the event
- Filters by MAC address if specified
- Triggers matching notification channels

**Trigger Types:**

- `new_device` - First time ever seeing a device
- `device_connected` - Device came online (from offline)
- `device_disconnected` - Device went offline (from online)
- `device_status_change` - Any status transition

### 6. Notifier

**Purpose:** Send notifications to external services.

**Supported Channels:**

**Telegram:**

- Uses Telegram Bot API
- Sends HTML-formatted messages
- Requires bot token and chat ID

**ntfy:**

- Posts to ntfy.sh or self-hosted servers
- Supports authentication tokens
- Uses tags and priority levels

**Webhook:**

- HTTP POST with JSON payload
- Custom headers supported
- Flexible integration with any HTTP endpoint

### 7. REST API

**Purpose:** Provide programmatic access to daemon functionality.

**Framework:** Axum (async Rust web framework)

**Endpoints:**

- Device management (`/api/devices/*`)
- Rule management (`/api/rules/*`)
- Notification channel management (`/api/notifications/*`)
- Configuration (`/api/config`)
- Metrics and logs (`/api/metrics`, `/api/logs`)

### 8. Web Console

**Purpose:** User interface for managing foxd.

**Technology:** SvelteKit + TypeScript

**Features:**

- Dashboard with metrics and device list
- Device management (nicknames, status)
- Rule creation and editing
- Notification channel configuration
- Log viewer

**Embedded:** Compiled to static assets and embedded into the Rust binary using `rust-embed`. No separate web server needed.

## Data Flow

### Device Discovery Flow

```
1. Packet captured (ARP/DHCP) or neighbor event received
   ↓
2. Event processor extracts MAC, IP, hostname
   ↓
3. Check if device exists in database
   ↓
4. If new: Insert device, mark as "new_device" event
   If existing: Update last_seen, check status change
   ↓
5. Update device status (online/offline) based on activity
   ↓
6. Emit event to rule engine
   ↓
7. Rule engine evaluates all active rules
   ↓
8. Matching rules trigger notifications via notifier
   ↓
9. Notifier sends to configured channels
   ↓
10. Log event and notification in database
```

### API Request Flow

```
1. HTTP request received by Axum
   ↓
2. Route matched and handler called
   ↓
3. Handler extracts parameters and body
   ↓
4. Database query or configuration read
   ↓
5. Optional: Update notifier state if channels changed
   ↓
6. Format response as JSON
   ↓
7. Return HTTP response to client
```

## Concurrency Model

foxd uses Tokio for async runtime:

- **Packet Capture Thread:** Blocking libpcap operations run in dedicated thread
- **Netlink Monitor:** Async task subscribing to kernel events
- **Event Processor:** Async task processing queued events
- **Database:** Wrapped with `Arc<RwLock>` for safe concurrent access
- **API Server:** Tokio async HTTP server with per-request tasks
- **Notifier:** Async HTTP clients for sending notifications

## Configuration

Configuration is loaded at startup from `config.toml`:

```toml
[daemon]
interface = "eth0"
device_timeout_secs = 300

[database]
path = "./foxd.db"

[api]
host = "0.0.0.0"
port = 8080
```

Notification channels are stored in the database, not in config file. This allows dynamic management without restarts.

## Performance Characteristics

**Memory Usage:**

- Base: ~10-20 MB (daemon + web console embedded)
- Per device: ~1-2 KB in-memory state
- Database size grows with devices and logs

**CPU Usage:**

- Idle: <1% (waiting for events)
- Active: 2-5% during packet capture and processing
- Spikes during rule evaluation and notification sending

**Network:**

- Passive listening (no active probing)
- Outbound only for notifications
- Minimal bandwidth impact

**Disk I/O:**

- SQLite writes on every device update
- Log entries written continuously
- Configurable log cleanup reduces growth

## Security Considerations

**Required Privileges:**

- `CAP_NET_RAW` and `CAP_NET_ADMIN` for packet capture and netlink
- Or run as root (not recommended)

**Network Exposure:**

- API has no authentication
- Bind to `127.0.0.1` for localhost-only access
- Use firewall rules or reverse proxy for external access

**Stored Credentials:**

- Telegram bot tokens and ntfy auth tokens stored in SQLite
- Database file should be protected with file permissions
- No encryption at rest (consider full-disk encryption)

**Dependencies:**

- All Rust crates vetted through cargo audit
- Minimal external dependencies
- No npm packages in runtime (only build time for console)

## Deployment Patterns

### Single Host Monitoring

Most common deployment. foxd runs on the network it monitors:

```
┌─────────────────────────┐
│  Raspberry Pi / Server  │
│                         │
│  ┌─────────────────┐    │
│  │      foxd       │    │
│  └─────────────────┘    │
│                         │
│  Connected to LAN       │
└──────────┬──────────────┘
           │
    ┌──────▼──────────────────┐
    │      Local Network      │
    │  (Devices to monitor)   │
    └─────────────────────────┘
```

### Multiple Network Segments

Run multiple foxd instances for different network segments:

```
┌────────────┐         ┌────────────┐
│  foxd #1   │         │  foxd #2   │
│ (Network A)│         │ (Network B)│
└─────┬──────┘         └─────┬──────┘
      │                      │
      │   ┌──────────────┐   │
      └───►   Webhook    ◄───┘
          │  Aggregator  │
          └──────────────┘
```

Each instance monitors its segment, sends notifications to central webhook.

### Container Deployment

foxd can run in Docker with host network mode:

```bash
docker run --network=host \
  -v /path/to/config.toml:/etc/foxd/config.toml \
  -v /path/to/data:/var/lib/foxd \
  --cap-add=NET_RAW \
  --cap-add=NET_ADMIN \
  foxd:latest
```

## Future Architecture Considerations

**Potential Enhancements:**

- Event streaming (MQTT, Redis pub/sub)
- Multi-host coordination (shared database)
- Authentication and RBAC for API
- Encrypted database with key management
- Plugin system for custom notifiers
- Prometheus metrics export
- Distributed tracing

**Design Constraints:**

- Maintain single-binary deployment
- Keep core functionality simple and focused
- Avoid cloud dependencies
- Minimize external service requirements
