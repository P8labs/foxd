# API Examples

This guide provides practical examples of using the foxd REST API for common tasks.

## Prerequisites

All examples assume foxd is running at `http://localhost:8080`. Adjust the URL as needed for your deployment.

Examples use `curl` for simplicity, but any HTTP client will work.

## Basic Examples

### Check Daemon Health

```bash
curl http://localhost:8080/api/health
```

Response:

```json
{
  "status": "ok",
  "service": "foxd",
  "uptime_seconds": 3600,
  "system": {
    "cpu_usage_percent": 5,
    "memory_usage_percent": 12,
    "total_memory_mb": 4096,
    "used_memory_mb": 491
  }
}
```

### Get All Devices

```bash
curl http://localhost:8080/api/devices
```

### Get Metrics

```bash
curl http://localhost:8080/api/metrics
```

## Device Management

### Find Online Devices

Using `jq` to filter:

```bash
curl -s http://localhost:8080/api/devices | \
  jq '.devices[] | select(.status == "online")'
```

### Find a Specific Device by MAC

```bash
MAC="aa:bb:cc:dd:ee:ff"
curl http://localhost:8080/api/devices/$MAC
```

### Set a Device Nickname

```bash
MAC="aa:bb:cc:dd:ee:ff"
curl -X POST http://localhost:8080/api/devices/$MAC/nickname \
  -H "Content-Type: application/json" \
  -d '{"nickname": "Dad'\''s Laptop"}'
```

### Clear a Device Nickname

```bash
MAC="aa:bb:cc:dd:ee:ff"
curl -X POST http://localhost:8080/api/devices/$MAC/nickname \
  -H "Content-Type: application/json" \
  -d '{"nickname": null}'
```

### List Devices Not Seen in Last Hour

```bash
curl -s http://localhost:8080/api/devices | \
  jq --arg cutoff "$(date -u -d '1 hour ago' '+%Y-%m-%dT%H:%M:%SZ')" \
    '.devices[] | select(.last_seen < $cutoff)'
```

## Notification Setup

### Create a Telegram Channel

```bash
curl -X POST http://localhost:8080/api/notifications \
  -H "Content-Type: application/json" \
  -d '{
    "type": "telegram",
    "bot_token": "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11",
    "chat_id": "123456789"
  }'
```

### Create an ntfy Channel

```bash
curl -X POST http://localhost:8080/api/notifications \
  -H "Content-Type: application/json" \
  -d '{
    "type": "ntfy",
    "server_url": "https://ntfy.sh",
    "topic": "foxd-home-network",
    "token": null
  }'
```

### Create a Webhook Channel

```bash
curl -X POST http://localhost:8080/api/notifications \
  -H "Content-Type: application/json" \
  -d '{
    "type": "webhook",
    "url": "https://example.com/webhooks/foxd",
    "headers": {
      "Authorization": "Bearer secret-token-123",
      "X-Source": "foxd"
    }
  }'
```

### List All Notification Channels

```bash
curl http://localhost:8080/api/notifications
```

### Update a Channel

```bash
CHANNEL_ID=1
curl -X PUT http://localhost:8080/api/notifications/$CHANNEL_ID \
  -H "Content-Type: application/json" \
  -d '{
    "type": "telegram",
    "bot_token": "NEW_BOT_TOKEN",
    "chat_id": "123456789"
  }'
```

### Delete a Channel

```bash
CHANNEL_ID=1
curl -X DELETE http://localhost:8080/api/notifications/$CHANNEL_ID
```

## Rule Management

### Create a Rule for New Devices

```bash
curl -X POST http://localhost:8080/api/rules \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Alert on new devices",
    "description": "Send notification when an unknown device appears",
    "trigger_type": "new_device",
    "mac_filter": null,
    "enabled": true,
    "notification_channels": ["telegram_123456789"]
  }'
```

### Create a Rule for Specific Device Going Offline

```bash
curl -X POST http://localhost:8080/api/rules \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Work laptop disconnected",
    "description": "Alert when work laptop goes offline",
    "trigger_type": "device_disconnected",
    "mac_filter": "aa:bb:cc:dd:ee:ff",
    "enabled": true,
    "notification_channels": ["telegram_123456789", "ntfy_alerts"]
  }'
```

### Create a Rule for Any Device Connecting

```bash
curl -X POST http://localhost:8080/api/rules \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Device connected",
    "description": "Log all device connections",
    "trigger_type": "device_connected",
    "mac_filter": null,
    "enabled": true,
    "notification_channels": ["webhook_logger"]
  }'
```

### List All Rules

```bash
curl http://localhost:8080/api/rules
```

### Get a Specific Rule

```bash
RULE_ID=1
curl http://localhost:8080/api/rules/$RULE_ID
```

### Disable a Rule

Get the rule first, then update with `enabled: false`:

```bash
RULE_ID=1

# Get current rule
RULE=$(curl -s http://localhost:8080/api/rules/$RULE_ID)

# Update with enabled: false
echo $RULE | jq '.enabled = false | del(.id, .created_at, .updated_at)' | \
  curl -X POST http://localhost:8080/api/rules/$RULE_ID \
    -H "Content-Type: application/json" \
    -d @-
```

### Delete a Rule

```bash
RULE_ID=1
curl -X POST http://localhost:8080/api/rules/$RULE_ID/delete
```

## Configuration Management

### Get Current Configuration

```bash
curl http://localhost:8080/api/config
```

### Update Configuration

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

**Note:** Most daemon configuration changes require a restart.

### Restart the Daemon

```bash
curl -X POST http://localhost:8080/api/restart
```

## Monitoring and Logs

### View Recent Logs

```bash
curl http://localhost:8080/api/logs
```

### Filter Logs by Category

```bash
# Get notification logs only
curl -s http://localhost:8080/api/logs | \
  jq '.logs[] | select(.category == "notification")'
```

### Filter Logs by Level

```bash
# Get errors only
curl -s http://localhost:8080/api/logs | \
  jq '.logs[] | select(.level == "error")'
```

### Watch Logs in Real-Time

Using `watch`:

```bash
watch -n 2 'curl -s http://localhost:8080/api/logs | jq ".logs[0:5]"'
```

Or continuously poll:

```bash
while true; do
  clear
  curl -s http://localhost:8080/api/logs | jq '.logs[0:10]'
  sleep 5
done
```

## Automation Scripts

### Monitor for Specific Device and Take Action

```bash
#!/bin/bash

TARGET_MAC="aa:bb:cc:dd:ee:ff"
API_URL="http://localhost:8080/api"

# Check device status
STATUS=$(curl -s "$API_URL/devices/$TARGET_MAC" | jq -r '.status')

if [ "$STATUS" == "online" ]; then
  echo "Device is online!"
  # Perform action (e.g., open garage)
  # ./open-garage.sh
else
  echo "Device is offline"
fi
```

### Daily Device Report

```bash
#!/bin/bash

API_URL="http://localhost:8080/api"

echo "=== Daily Device Report ==="
echo "Date: $(date)"
echo

# Get metrics
METRICS=$(curl -s "$API_URL/metrics")

echo "Total devices: $(echo $METRICS | jq -r '.total_devices')"
echo "Online devices: $(echo $METRICS | jq -r '.online_devices')"
echo "Offline devices: $(echo $METRICS | jq -r '.offline_devices')"
echo

# List online devices
echo "=== Online Devices ==="
curl -s "$API_URL/devices" | \
  jq -r '.devices[] | select(.status == "online") | "\(.nickname // .hostname // "Unknown"): \(.ip_address // "No IP") (\(.mac_address))"'

echo
echo "=== Recent Events ==="
curl -s "$API_URL/logs" | \
  jq -r '.logs[0:5] | .[] | "\(.timestamp) [\(.level)] \(.message)"'
```

### Bulk Nickname Assignment

```bash
#!/bin/bash

# Map of MAC addresses to nicknames
declare -A DEVICES=(
  ["aa:bb:cc:dd:ee:01"]="Dad's iPhone"
  ["aa:bb:cc:dd:ee:02"]="Mom's Laptop"
  ["aa:bb:cc:dd:ee:03"]="Living Room TV"
  ["aa:bb:cc:dd:ee:04"]="Smart Thermostat"
)

API_URL="http://localhost:8080/api"

for MAC in "${!DEVICES[@]}"; do
  NICKNAME="${DEVICES[$MAC]}"
  echo "Setting nickname for $MAC: $NICKNAME"

  curl -X POST "$API_URL/devices/$MAC/nickname" \
    -H "Content-Type: application/json" \
    -d "{\"nickname\": \"$NICKNAME\"}" \
    -s -o /dev/null
done

echo "Done!"
```

### Backup Database via API

```bash
#!/bin/bash

API_URL="http://localhost:8080/api"
BACKUP_DIR="./foxd-backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

# Export devices
curl -s "$API_URL/devices" > "$BACKUP_DIR/devices_$TIMESTAMP.json"

# Export rules
curl -s "$API_URL/rules" > "$BACKUP_DIR/rules_$TIMESTAMP.json"

# Export channels
curl -s "$API_URL/notifications" > "$BACKUP_DIR/channels_$TIMESTAMP.json"

# Export logs
curl -s "$API_URL/logs" > "$BACKUP_DIR/logs_$TIMESTAMP.json"

echo "Backup saved to $BACKUP_DIR/"
```

## Integration Examples

### Home Assistant Automation

Using Home Assistant's RESTful integration:

```yaml
# configuration.yaml
sensor:
  - platform: rest
    name: Foxd Online Devices
    resource: http://foxd-host:8080/api/metrics
    value_template: "{{ value_json.online_devices }}"

binary_sensor:
  - platform: rest
    name: Work Laptop Present
    resource: http://foxd-host:8080/api/devices/aa:bb:cc:dd:ee:ff
    value_template: "{{ value_json.status == 'online' }}"
    device_class: presence

automation:
  - alias: "Garage - Open when car arrives"
    trigger:
      - platform: state
        entity_id: binary_sensor.car_device_present
        to: "on"
    action:
      - service: cover.open_cover
        target:
          entity_id: cover.garage_door
```

### Python Script Example

```python
import requests
import time

API_BASE = "http://localhost:8080/api"

def get_online_devices():
    """Get list of online devices"""
    response = requests.get(f"{API_BASE}/devices")
    devices = response.json()["devices"]
    return [d for d in devices if d["status"] == "online"]

def create_notification_rule(name, trigger_type, channels):
    """Create a new notification rule"""
    rule = {
        "name": name,
        "description": f"Auto-created rule: {name}",
        "trigger_type": trigger_type,
        "mac_filter": None,
        "enabled": True,
        "notification_channels": channels
    }
    response = requests.post(f"{API_BASE}/rules", json=rule)
    return response.json()

def monitor_device(mac_address, check_interval=30):
    """Monitor a specific device and print status changes"""
    last_status = None

    while True:
        try:
            response = requests.get(f"{API_BASE}/devices/{mac_address}")
            device = response.json()
            current_status = device["status"]

            if current_status != last_status:
                print(f"Device {mac_address} status changed: {last_status} -> {current_status}")
                last_status = current_status

            time.sleep(check_interval)
        except Exception as e:
            print(f"Error: {e}")
            time.sleep(check_interval)

# Example usage
if __name__ == "__main__":
    # List online devices
    online = get_online_devices()
    print(f"Online devices: {len(online)}")
    for device in online:
        name = device.get("nickname") or device.get("hostname") or "Unknown"
        print(f"  - {name} ({device['mac_address']})")
```

### Node.js/JavaScript Example

```javascript
const axios = require("axios");

const API_BASE = "http://localhost:8080/api";

async function getDevices() {
  const response = await axios.get(`${API_BASE}/devices`);
  return response.data.devices;
}

async function createTelegramChannel(botToken, chatId) {
  const channel = {
    type: "telegram",
    bot_token: botToken,
    chat_id: chatId,
  };

  const response = await axios.post(`${API_BASE}/notifications`, channel);
  return response.data;
}

async function createRule(name, triggerType, channels) {
  const rule = {
    name,
    description: `Auto-created: ${name}`,
    trigger_type: triggerType,
    mac_filter: null,
    enabled: true,
    notification_channels: channels,
  };

  const response = await axios.post(`${API_BASE}/rules`, rule);
  return response.data;
}

// Example: Setup notifications for new devices
async function setupNewDeviceAlert() {
  // Create Telegram channel
  const channel = await createTelegramChannel(
    process.env.TELEGRAM_BOT_TOKEN,
    process.env.TELEGRAM_CHAT_ID,
  );

  console.log(`Created channel: ${channel.id}`);

  // Create rule
  const channelName = `telegram_${process.env.TELEGRAM_CHAT_ID}`;
  const rule = await createRule("Alert on new devices", "new_device", [
    channelName,
  ]);

  console.log(`Created rule: ${rule.name} (ID: ${rule.id})`);
}

setupNewDeviceAlert().catch(console.error);
```

## Tips and Best Practices

1. **Use jq for JSON processing:** Install `jq` for easier JSON manipulation in scripts
2. **Store API URL in variable:** Makes scripts portable across environments
3. **Check HTTP status codes:** Always verify response status in production scripts
4. **Handle errors gracefully:** Add error handling for network issues and API errors
5. **Rate limiting:** While not enforced, avoid hammering the API unnecessarily
6. **Use webhooks for real-time:** For event-driven automation, use webhook notifications instead of polling
7. **Backup regularly:** Export configuration and data periodically
8. **Security:** Never expose the API publicly without authentication/firewall

## See Also

- [REST API Reference](../api/) - Complete API documentation
- [Notifications Guide](../guide/notifications) - Setting up notification rules
- [Configuration Guide](../guide/configuration) - Daemon configuration options
