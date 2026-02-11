# REST API Reference

foxd exposes a REST API under the `/api` prefix. All responses are JSON.

## Base URL

The API is available at:

```
http://<host>:<port>/api
```

Default: `http://localhost:8080/api`

## Authentication

The current version of foxd does not require authentication. API access is controlled by network-level security (bind address configuration). It is recommended to bind to `127.0.0.1` or use a firewall if exposing the API.

## Response Format

All successful responses return JSON with appropriate HTTP status codes:

- `200 OK` - Successful GET, POST, or PUT request
- `201 Created` - Resource created successfully (not currently used)
- `204 No Content` - Successful DELETE (not currently used)
- `400 Bad Request` - Invalid request data
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error

### Error Response

All errors return a JSON object with an `error` field:

```json
{
  "error": "Description of the error",
  "details": "Optional additional details"
}
```

## Health

### `GET /api/health`

Returns daemon status and system metrics.

**Response:**

```json
{
  "status": "ok",
  "service": "foxd",
  "uptime_seconds": 3600,
  "system": {
    "cpu_usage_percent": 12,
    "memory_usage_percent": 45,
    "total_memory_mb": 1024,
    "used_memory_mb": 460
  }
}
```

**Status Codes:**

- `200 OK` - Always returns 200 if daemon is running

## Devices

### `GET /api/devices`

List all discovered devices on the network.

**Response:**

```json
{
  "devices": [
    {
      "id": 1,
      "mac_address": "aa:bb:cc:dd:ee:ff",
      "ip_address": "192.168.1.42",
      "hostname": "my-laptop",
      "nickname": null,
      "vendor": "Apple Inc.",
      "status": "online",
      "first_seen": "2025-01-15T10:00:00Z",
      "last_seen": "2025-01-15T12:30:00Z"
    }
  ],
  "count": 1
}
```

**Device Status Values:**

- `online` - Device has been seen recently
- `offline` - Device has not been seen within timeout period
- `unknown` - Initial state before first status update

**Status Codes:**

- `200 OK` - Success
- `500 Internal Server Error` - Database error

### `GET /api/devices/{mac}`

Get a single device by MAC address.

**Parameters:**

- `mac` (path) - MAC address in format `aa:bb:cc:dd:ee:ff` (colons required)

**Response:**

```json
{
  "id": 1,
  "mac_address": "aa:bb:cc:dd:ee:ff",
  "ip_address": "192.168.1.42",
  "hostname": "my-laptop",
  "nickname": "Dad's Laptop",
  "vendor": "Apple Inc.",
  "status": "online",
  "first_seen": "2025-01-15T10:00:00Z",
  "last_seen": "2025-01-15T12:30:00Z"
}
```

**Status Codes:**

- `200 OK` - Device found
- `404 Not Found` - Device does not exist
- `500 Internal Server Error` - Database error

**Example:**

```bash
curl http://localhost:8080/api/devices/aa:bb:cc:dd:ee:ff
```

### `POST /api/devices/{mac}/nickname`

Set or clear a device nickname.

**Parameters:**

- `mac` (path) - MAC address in format `aa:bb:cc:dd:ee:ff`

**Request Body:**

```json
{
  "nickname": "Dad's Laptop"
}
```

To clear a nickname, pass `null`:

```json
{
  "nickname": null
}
```

**Response:**

Returns the updated device object:

```json
{
  "id": 1,
  "mac_address": "aa:bb:cc:dd:ee:ff",
  "ip_address": "192.168.1.42",
  "hostname": "my-laptop",
  "nickname": "Dad's Laptop",
  "vendor": "Apple Inc.",
  "status": "online",
  "first_seen": "2025-01-15T10:00:00Z",
  "last_seen": "2025-01-15T12:30:00Z"
}
```

**Status Codes:**

- `200 OK` - Nickname updated
- `404 Not Found` - Device does not exist
- `500 Internal Server Error` - Database error

**Example:**

```bash
curl -X POST http://localhost:8080/api/devices/aa:bb:cc:dd:ee:ff/nickname \
  -H "Content-Type: application/json" \
  -d '{"nickname": "Dad'\''s Laptop"}'
```

## Rules

Rules define when and how notifications are triggered based on device events.

### `GET /api/rules`

List all notification rules.

**Response:**

```json
{
  "rules": [
    {
      "id": 1,
      "name": "Alert on new devices",
      "description": "Notify when an unknown device joins the network",
      "trigger_type": "new_device",
      "mac_filter": null,
      "enabled": true,
      "notification_channels": ["telegram_123456789"],
      "created_at": "2025-01-15T10:00:00Z",
      "updated_at": "2025-01-15T10:00:00Z"
    }
  ],
  "count": 1
}
```

**Trigger Types:**

- `new_device` - Device seen for the first time
- `device_connected` - Device comes online
- `device_disconnected` - Device goes offline
- `device_status_change` - Any status change (online ↔ offline)

**Status Codes:**

- `200 OK` - Success
- `500 Internal Server Error` - Database error

### `GET /api/rules/{id}`

Get a single rule by ID.

**Parameters:**

- `id` (path) - Rule ID (integer)

**Response:**

```json
{
  "id": 1,
  "name": "Alert on new devices",
  "description": "Notify when an unknown device joins the network",
  "trigger_type": "new_device",
  "mac_filter": null,
  "enabled": true,
  "notification_channels": ["telegram_123456789"],
  "created_at": "2025-01-15T10:00:00Z",
  "updated_at": "2025-01-15T10:00:00Z"
}
```

**Status Codes:**

- `200 OK` - Rule found
- `404 Not Found` - Rule does not exist
- `500 Internal Server Error` - Database error

### `POST /api/rules`

Create a new notification rule.

**Request Body:**

```json
{
  "name": "New device alert",
  "description": "Fires when a new device appears",
  "trigger_type": "new_device",
  "mac_filter": null,
  "enabled": true,
  "notification_channels": ["telegram_123456789", "ntfy_alerts"]
}
```

**Fields:**

- `name` (required) - Rule name
- `description` (optional) - Rule description
- `trigger_type` (required) - One of: `new_device`, `device_connected`, `device_disconnected`, `device_status_change`
- `mac_filter` (optional) - MAC address to filter (e.g., `aa:bb:cc:dd:ee:ff`). If `null`, rule applies to all devices
- `enabled` (required) - Boolean, whether rule is active
- `notification_channels` (required) - Array of channel names (use channel names from notification channels list)

**Response:**

Returns the created rule with assigned ID:

```json
{
  "id": 2,
  "name": "New device alert",
  "description": "Fires when a new device appears",
  "trigger_type": "new_device",
  "mac_filter": null,
  "enabled": true,
  "notification_channels": ["telegram_123456789", "ntfy_alerts"],
  "created_at": "2025-01-15T14:00:00Z",
  "updated_at": "2025-01-15T14:00:00Z"
}
```

**Status Codes:**

- `200 OK` - Rule created
- `400 Bad Request` - Invalid request data
- `500 Internal Server Error` - Database error

**Example:**

```bash
curl -X POST http://localhost:8080/api/rules \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Alert when laptop disconnects",
    "description": "Notify if work laptop goes offline",
    "trigger_type": "device_disconnected",
    "mac_filter": "aa:bb:cc:dd:ee:ff",
    "enabled": true,
    "notification_channels": ["telegram_123456789"]
  }'
```

### `POST /api/rules/{id}`

Update an existing rule.

**Parameters:**

- `id` (path) - Rule ID (integer)

**Request Body:**

Same format as create rule (all fields required):

```json
{
  "name": "Updated rule name",
  "description": "Updated description",
  "trigger_type": "device_connected",
  "mac_filter": null,
  "enabled": false,
  "notification_channels": ["ntfy_alerts"]
}
```

**Response:**

Returns the updated rule:

```json
{
  "id": 1,
  "name": "Updated rule name",
  "description": "Updated description",
  "trigger_type": "device_connected",
  "mac_filter": null,
  "enabled": false,
  "notification_channels": ["ntfy_alerts"],
  "created_at": "2025-01-15T10:00:00Z",
  "updated_at": "2025-01-15T14:30:00Z"
}
```

**Status Codes:**

- `200 OK` - Rule updated
- `404 Not Found` - Rule does not exist
- `400 Bad Request` - Invalid request data
- `500 Internal Server Error` - Database error

### `POST /api/rules/{id}/delete`

Delete a rule.

**Parameters:**

- `id` (path) - Rule ID (integer)

**Response:**

```json
{
  "message": "Rule 1 deleted successfully"
}
```

**Status Codes:**

- `200 OK` - Rule deleted
- `404 Not Found` - Rule does not exist
- `500 Internal Server Error` - Database error

**Example:**

```bash
curl -X POST http://localhost:8080/api/rules/1/delete
```

## Configuration

### `GET /api/config`

Get the current running configuration.

**Response:**

```json
{
  "daemon": {
    "interface": "eth0",
    "capture_filter": null,
    "neighbor_check_interval_secs": 60,
    "device_timeout_secs": 300,
    "log_cleanup_enabled": true,
    "log_retention_days": 30
  },
  "database": {
    "path": "./foxd.db"
  },
  "api": {
    "host": "0.0.0.0",
    "port": 8080
  }
}
```

**Status Codes:**

- `200 OK` - Success

### `POST /api/config`

Update configuration at runtime.

**Important:** Changes to notification channels take effect immediately. Changes to daemon settings (interface, timeouts, etc.) require a daemon restart to take effect.

**Request Body:**

```json
{
  "daemon": {
    "interface": "wlan0",
    "capture_filter": "arp or (udp port 67 or udp port 68)",
    "neighbor_check_interval_secs": 30,
    "device_timeout_secs": 180,
    "log_cleanup_enabled": true,
    "log_retention_days": 7
  }
}
```

All daemon fields are required if the `daemon` object is provided.

**Response:**

```json
{
  "message": "Configuration updated successfully"
}
```

**Status Codes:**

- `200 OK` - Configuration updated
- `400 Bad Request` - Invalid configuration
- `500 Internal Server Error` - Failed to update

**Example:**

```bash
curl -X POST http://localhost:8080/api/config \
  -H "Content-Type: application/json" \
  -d '{
    "daemon": {
      "interface": "eth0",
      "capture_filter": null,
      "neighbor_check_interval_secs": 60,
      "device_timeout_secs": 300,
      "log_cleanup_enabled": true,
      "log_retention_days": 30
    }
  }'
```

## Notification Channels

Notification channels define where alerts are sent. Channels are stored in the database and can be managed dynamically.

### `GET /api/notifications`

List all configured notification channels.

**Response:**

```json
{
  "channels": [
    {
      "id": 1,
      "channel": {
        "type": "telegram",
        "bot_token": "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11",
        "chat_id": "123456789"
      }
    },
    {
      "id": 2,
      "channel": {
        "type": "ntfy",
        "server_url": "https://ntfy.sh",
        "topic": "foxd-alerts",
        "token": null
      }
    },
    {
      "id": 3,
      "channel": {
        "type": "webhook",
        "url": "https://example.com/webhook",
        "headers": {
          "Authorization": "Bearer token123"
        }
      }
    }
  ],
  "count": 3
}
```

**Channel Types:**

- `telegram` - Send via Telegram bot
- `ntfy` - Send to ntfy.sh or self-hosted ntfy server
- `webhook` - POST to custom HTTP endpoint

**Status Codes:**

- `200 OK` - Success
- `500 Internal Server Error` - Database error

### `GET /api/notifications/{id}`

Get a specific notification channel by ID.

**Parameters:**

- `id` (path) - Channel ID (integer)

**Response:**

```json
{
  "id": 1,
  "channel": {
    "type": "telegram",
    "bot_token": "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11",
    "chat_id": "123456789"
  }
}
```

**Status Codes:**

- `200 OK` - Channel found
- `404 Not Found` - Channel does not exist
- `500 Internal Server Error` - Database error

### `POST /api/notifications`

Create a new notification channel.

**Request Body - Telegram:**

```json
{
  "type": "telegram",
  "bot_token": "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11",
  "chat_id": "123456789"
}
```

**Request Body - ntfy:**

```json
{
  "type": "ntfy",
  "server_url": "https://ntfy.sh",
  "topic": "foxd-alerts",
  "token": "tk_xxxxxxxxxxxx"
}
```

The `token` field is optional and can be `null` for public topics.

**Request Body - Webhook:**

```json
{
  "type": "webhook",
  "url": "https://example.com/webhook",
  "headers": {
    "Authorization": "Bearer token123",
    "X-Custom-Header": "value"
  }
}
```

The `headers` field is optional and can be `null`.

**Response:**

Returns the created channel with assigned ID:

```json
{
  "id": 4,
  "channel": {
    "type": "telegram",
    "bot_token": "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11",
    "chat_id": "123456789"
  }
}
```

**Status Codes:**

- `200 OK` - Channel created
- `400 Bad Request` - Invalid channel configuration
- `500 Internal Server Error` - Database error

**Example:**

```bash
curl -X POST http://localhost:8080/api/notifications \
  -H "Content-Type: application/json" \
  -d '{
    "type": "telegram",
    "bot_token": "YOUR_BOT_TOKEN",
    "chat_id": "123456789"
  }'
```

### `PUT /api/notifications/{id}`

Update an existing notification channel.

**Parameters:**

- `id` (path) - Channel ID (integer)

**Request Body:**

Same format as create (provide complete channel configuration):

```json
{
  "type": "telegram",
  "bot_token": "NEW_BOT_TOKEN",
  "chat_id": "123456789"
}
```

**Response:**

Returns the updated channel:

```json
{
  "id": 1,
  "channel": {
    "type": "telegram",
    "bot_token": "NEW_BOT_TOKEN",
    "chat_id": "123456789"
  }
}
```

**Status Codes:**

- `200 OK` - Channel updated
- `404 Not Found` - Channel does not exist
- `400 Bad Request` - Invalid channel configuration
- `500 Internal Server Error` - Database error

**Example:**

```bash
curl -X PUT http://localhost:8080/api/notifications/1 \
  -H "Content-Type: application/json" \
  -d '{
    "type": "telegram",
    "bot_token": "NEW_BOT_TOKEN",
    "chat_id": "123456789"
  }'
```

### `DELETE /api/notifications/{id}`

Delete a notification channel.

**Parameters:**

- `id` (path) - Channel ID (integer)

**Response:**

```json
{
  "message": "Notification channel 1 deleted successfully"
}
```

**Status Codes:**

- `200 OK` - Channel deleted
- `404 Not Found` - Channel does not exist
- `500 Internal Server Error` - Database error

**Example:**

```bash
curl -X DELETE http://localhost:8080/api/notifications/1
```

**Note:** Deleting a channel does not automatically update rules that reference it. You should update or delete affected rules separately.

## Metrics

### `GET /api/metrics`

Get comprehensive daemon metrics and statistics.

**Response:**

```json
{
  "total_devices": 15,
  "online_devices": 8,
  "offline_devices": 7,
  "total_rules": 3,
  "enabled_rules": 2,
  "packets_captured": 45230,
  "notifications_sent": 12,
  "uptime_seconds": 86400
}
```

**Fields:**

- `total_devices` - Total number of devices ever discovered
- `online_devices` - Currently online devices
- `offline_devices` - Currently offline devices
- `total_rules` - Total number of rules configured
- `enabled_rules` - Number of enabled rules
- `packets_captured` - Total packets processed since daemon start
- `notifications_sent` - Total notifications sent since daemon start
- `uptime_seconds` - Daemon uptime in seconds

**Status Codes:**

- `200 OK` - Success
- `500 Internal Server Error` - Database error

**Example:**

```bash
curl http://localhost:8080/api/metrics
```

## Logs

### `GET /api/logs`

Get the most recent log entries (up to 200).

**Response:**

```json
{
  "logs": [
    {
      "id": 42,
      "timestamp": "2025-01-15T12:00:00Z",
      "level": "info",
      "category": "device",
      "message": "New device discovered: aa:bb:cc:dd:ee:ff",
      "details": null
    },
    {
      "id": 41,
      "timestamp": "2025-01-15T11:58:30Z",
      "level": "info",
      "category": "notification",
      "message": "Notification sent via telegram_123456789",
      "details": "{\"device\": \"aa:bb:cc:dd:ee:ff\", \"event\": \"new_device\"}"
    },
    {
      "id": 40,
      "timestamp": "2025-01-15T11:55:00Z",
      "level": "warning",
      "category": "system",
      "message": "High packet capture rate detected",
      "details": null
    }
  ],
  "count": 3
}
```

**Log Levels:**

- `info` - Informational messages
- `warning` - Warning messages
- `error` - Error messages
- `debug` - Debug messages (if enabled)

**Log Categories:**

- `device` - Device discovery and status changes
- `notification` - Notification events
- `system` - System-level events
- `api` - API requests and responses
- `config` - Configuration changes

**Status Codes:**

- `200 OK` - Success
- `500 Internal Server Error` - Database error

**Example:**

```bash
curl http://localhost:8080/api/logs
```

**Note:** Logs are automatically cleaned up based on the `log_retention_days` configuration setting. By default, logs older than 30 days are deleted.

## Restart

### `POST /api/restart`

Trigger a controlled daemon restart.

The daemon will exit with exit code 42, which can be detected by systemd or a wrapper script to perform a restart. This is useful for applying configuration changes that require a restart.

**Response:**

```json
{
  "message": "Daemon restart initiated"
}
```

**Status Codes:**

- `200 OK` - Restart initiated

**Example:**

```bash
curl -X POST http://localhost:8080/api/restart
```

**Systemd Integration:**

If running under systemd with `Restart=on-failure`, the daemon will automatically restart after exit code 42. To configure this properly, use:

```ini
[Service]
Type=simple
ExecStart=/usr/local/bin/foxd
Restart=always
RestartSec=5
```

**Behavior:**

- The API will respond immediately with a 200 status
- After 1 second, the daemon process will exit
- All active connections will be closed
- Database changes will be committed
- If running under systemd, the service will restart automatically

## Channel Names

When referencing notification channels in rules, use the auto-generated channel name format:

| Channel Type | Name Format                  | Example              |
| ------------ | ---------------------------- | -------------------- |
| Telegram     | `telegram_{chat_id}`         | `telegram_123456789` |
| ntfy         | `ntfy_{topic}`               | `ntfy_alerts`        |
| Webhook      | `webhook_{last_url_segment}` | `webhook_endpoint`   |

These names are automatically generated and returned in the API responses from `/api/notifications`.

## Data Types

### Device Object

```json
{
  "id": 1,
  "mac_address": "aa:bb:cc:dd:ee:ff",
  "ip_address": "192.168.1.42",
  "hostname": "device-name",
  "nickname": "My Device",
  "vendor": "Apple Inc.",
  "status": "online",
  "first_seen": "2025-01-15T10:00:00Z",
  "last_seen": "2025-01-15T12:30:00Z"
}
```

### Rule Object

```json
{
  "id": 1,
  "name": "Alert on new devices",
  "description": "Notify when an unknown device joins",
  "trigger_type": "new_device",
  "mac_filter": null,
  "enabled": true,
  "notification_channels": ["telegram_123456789"],
  "created_at": "2025-01-15T10:00:00Z",
  "updated_at": "2025-01-15T10:00:00Z"
}
```

### Notification Channel Object

```json
{
  "id": 1,
  "channel": {
    "type": "telegram",
    "bot_token": "...",
    "chat_id": "123456789"
  }
}
```

## Webhook Payload Format

When using webhook notification channels, foxd sends a POST request with the following JSON payload:

```json
{
  "timestamp": "2025-01-15T12:00:00Z",
  "event_type": "new_device",
  "device": {
    "id": 1,
    "mac_address": "aa:bb:cc:dd:ee:ff",
    "ip_address": "192.168.1.42",
    "hostname": "device-name",
    "nickname": null,
    "vendor": "Apple Inc.",
    "status": "online",
    "first_seen": "2025-01-15T12:00:00Z",
    "last_seen": "2025-01-15T12:00:00Z"
  },
  "message": "New device discovered: device-name (aa:bb:cc:dd:ee:ff)"
}
```

**Event Types:**

- `new_device` - First time device is seen
- `device_connected` - Device came online
- `device_disconnected` - Device went offline
- `device_status_change` - Device status changed

## Rate Limiting

The current version does not implement rate limiting. It is recommended to use network-level controls or a reverse proxy if rate limiting is needed.

## CORS

CORS is configured permissively to allow the embedded web console to function. If exposing the API to external clients, consider implementing stricter CORS policies.

## Best Practices

1. **Polling:** Avoid polling endpoints unnecessarily. Most data changes infrequently (devices, rules, config). The metrics endpoint can be polled more frequently if needed for monitoring.

2. **Error Handling:** Always check HTTP status codes and parse error responses from the API.

3. **MAC Address Format:** Always use lowercase MAC addresses with colon separators (`aa:bb:cc:dd:ee:ff`).

4. **Channel Configuration:** Test notification channels after creation to ensure they work correctly.

5. **Configuration Changes:** Remember that most daemon configuration changes require a restart. Use the `/api/restart` endpoint after updating daemon settings.

6. **Database Backups:** The SQLite database file should be backed up regularly. Stop the daemon before backing up, or use SQLite backup tools for online backups.
