# Notification Channel Management

## Overview

Notification channels in foxd are stored in the database and can be managed dynamically through the console UI or REST API. This allows you to add, update, or remove notification channels without restarting the daemon or editing configuration files.

## Supported Channel Types

### 1. Telegram Bot

Send notifications via Telegram bot.

**Configuration:**

- `bot_token` (required): Your Telegram bot token from @BotFather
- `chat_id` (required): The chat ID where messages will be sent

**Example:**

```json
{
  "type": "telegram",
  "bot_token": "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11",
  "chat_id": "123456789"
}
```

**Channel Name:** `telegram_{chat_id}` (e.g., `telegram_123456789`)

### 2. ntfy.sh

Send notifications to ntfy.sh or self-hosted ntfy servers.

**Configuration:**

- `server_url` (required): The ntfy server URL (e.g., `https://ntfy.sh`)
- `topic` (required): The topic to publish to
- `token` (optional): Authentication token for protected topics

**Example:**

```json
{
  "type": "ntfy",
  "server_url": "https://ntfy.sh",
  "topic": "foxd-alerts",
  "token": "tk_xxxxxxxxxxxx"
}
```

**Channel Name:** `ntfy_{topic}` (e.g., `ntfy_foxd-alerts`)

### 3. Webhook

Send notifications to custom HTTP endpoints.

**Configuration:**

- `url` (required): The webhook URL
- `headers` (optional): Custom HTTP headers as a JSON object

**Example:**

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

**Channel Name:** `webhook_{last_path_segment}` (e.g., `webhook_endpoint`)

## Managing Channels via Console UI

### Adding a Channel

1. Open the foxd console in your browser
2. Navigate to **Configuration**
3. Scroll to the **Notification Channels** section
4. Click **Add Channel**
5. Select the channel type and fill in the required fields
6. Click **Add Channel** to save

### Editing a Channel

Currently, channels must be deleted and recreated to update them. This will be improved in future versions.

### Deleting a Channel

1. Navigate to **Configuration** -> **Notification Channels**
2. Find the channel you want to delete
3. Click **Remove** next to the channel
4. Confirm the deletion

**Note:** Deleting a channel will not automatically update rules that reference it. You may need to update your rules manually.

## Managing Channels via API

### List All Channels

```bash
curl http://localhost:8080/api/notifications
```

**Response:**

```json
{
  "channels": [
    {
      "id": 1,
      "channel": {
        "type": "telegram",
        "bot_token": "...",
        "chat_id": "123456789"
      }
    }
  ],
  "count": 1
}
```

### Create a Channel

```bash
curl -X POST http://localhost:8080/api/notifications \
  -H "Content-Type: application/json" \
  -d '{
    "type": "telegram",
    "bot_token": "YOUR_BOT_TOKEN",
    "chat_id": "123456789"
  }'
```

### Get a Specific Channel

```bash
curl http://localhost:8080/api/notifications/1
```

### Update a Channel

```bash
curl -X PUT http://localhost:8080/api/notifications/1 \
  -H "Content-Type: application/json" \
  -d '{
    "type": "telegram",
    "bot_token": "NEW_BOT_TOKEN",
    "chat_id": "123456789"
  }'
```

### Delete a Channel

```bash
curl -X DELETE http://localhost:8080/api/notifications/1
```

## Using Channels in Rules

When creating or editing notification rules, you specify which channels should receive notifications by providing their channel names.

**Example Rule:**

```json
{
  "name": "Alert on new devices",
  "trigger_type": "new_device",
  "notification_channels": ["telegram_123456789", "ntfy_alerts"],
  "enabled": true
}
```

The channels will be matched by their auto-generated names. You can find the available channel names in:

- The console UI when creating/editing rules
- The API response from `GET /api/notifications`

## Notification Message Format

### Telegram

Messages are sent as HTML-formatted text with the following information:

- Event type (Device Connected, New Device, etc.)
- Device hostname
- IP address
- MAC address
- Device status
- Timestamp
- Custom message

### ntfy

Messages include:

- Title: Event type
- Body: Device information (hostname, MAC, IP, status)
- Tags: `fox,network`
- Priority: `default`

### Webhook

JSON payload with complete event information:

```json
{
  "timestamp": "2024-01-01T12:00:00Z",
  "event_type": "new_device",
  "device": {
    "mac_address": "aa:bb:cc:dd:ee:ff",
    "ip_address": "192.168.1.100",
    "hostname": "device-hostname",
    "status": "online",
    "last_seen": "2024-01-01T12:00:00Z"
  },
  "message": "New device discovered"
}
```

## Testing Channels

To test if a channel is working correctly:

1. Create a simple test rule:
   - Trigger: "Device Connected" or "Device Status Change"
   - MAC Filter: Leave empty to match all devices
   - Channels: Select your test channel
   - Enabled: ok

2. Wait for a device event to trigger (or manually trigger by disconnecting/reconnecting a device)

3. Check if you received the notification

4. Review daemon logs for any errors:
   ```bash
   journalctl -u foxd -f
   ```

## Best Practices

1. **Use descriptive names**: While channel names are auto-generated, consider your channel configuration (topic names, chat IDs) to make them recognizable

2. **Test new channels**: Always test a new channel with a simple rule before using it in production

3. **Secure your tokens**: Keep bot tokens and authentication tokens secure. Never commit them to version control

4. **Multiple channels for redundancy**: Configure multiple channels (e.g., Telegram + ntfy) for critical alerts

5. **Channel-specific rules**: Create different rules for different channels based on severity or device type

## Troubleshooting

### Notifications not being sent

1. **Check channel configuration**:
   - Verify bot token / server URL is correct
   - Ensure chat ID / topic is accessible
   - Test the endpoint manually (e.g., send a test Telegram message)

2. **Check rule configuration**:
   - Verify the rule is enabled
   - Ensure channel names match exactly
   - Check if MAC filter is too restrictive

3. **Review logs**:

   ```bash
   journalctl -u foxd -f | grep -i notification
   ```

4. **Verify connectivity**:
