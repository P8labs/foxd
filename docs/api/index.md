# REST API

foxd exposes a REST API under the `/api` prefix. All responses are JSON.

## Health

### `GET /api/health`

Returns daemon status and system metrics.

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

## Devices

### `GET /api/devices`

List all discovered devices.

```json
{
  "devices": [
    {
      "id": 1,
      "mac_address": "aa:bb:cc:dd:ee:ff",
      "ip_address": "192.168.1.42",
      "hostname": "my-laptop",
      "nickname": null,
      "status": "online",
      "first_seen": "2025-01-15T10:00:00Z",
      "last_seen": "2025-01-15T12:30:00Z"
    }
  ],
  "count": 1
}
```

### `GET /api/devices/{mac}`

Get a single device by MAC address.

### `POST /api/devices/{mac}/nickname`

Set or clear a device nickname.

```json
{ "nickname": "Dad's Laptop" }
```

Pass `null` to clear.

## Rules

### `GET /api/rules`

List all notification rules.

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
      "notification_channels": ["telegram"],
      "created_at": "2025-01-15T10:00:00Z",
      "updated_at": "2025-01-15T10:00:00Z"
    }
  ],
  "count": 1
}
```

### `GET /api/rules/{id}`

Get a single rule by ID.

### `POST /api/rules`

Create a new rule.

```json
{
  "name": "New device alert",
  "description": "Fires when a new device appears",
  "trigger_type": "new_device",
  "mac_filter": null,
  "enabled": true,
  "notification_channels": ["telegram"]
}
```

### `POST /api/rules/{id}`

Update an existing rule. Same body as create.

### `POST /api/rules/{id}/delete`

Delete a rule.

## Configuration

### `GET /api/config`

Get the current running configuration.

### `POST /api/config`

Update configuration at runtime. Changes to notification channels take effect immediately. Changes to daemon settings require a restart.

```json
{
  "notifications": [
    {
      "type": "telegram",
      "bot_token": "123456:ABC",
      "chat_id": "789"
    }
  ]
}
```

## Metrics

### `GET /api/metrics`

Get daemon metrics.

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

## Logs

### `GET /api/logs`

Get the most recent 200 log entries.

```json
{
  "logs": [
    {
      "id": 1,
      "timestamp": "2025-01-15T12:00:00Z",
      "level": "info",
      "category": "device",
      "message": "New device discovered: aa:bb:cc:dd:ee:ff",
      "details": null
    }
  ],
  "count": 1
}
```

## Restart

### `POST /api/restart`

Trigger a daemon restart. The daemon exits with code 42, which can be used by systemd or a wrapper script to restart it.
