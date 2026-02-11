# Notifications

foxd can send notifications when network events occur. You configure notification channels in `config.toml` and then reference them in rules.

## Supported Channels

### Telegram

Send notifications via a Telegram bot.

```toml
[[notifications]]
type = "telegram"
bot_token = "123456:ABC-DEF"
chat_id = "987654321"
```

To set up a Telegram bot:

1. Message [@BotFather](https://t.me/BotFather) on Telegram and create a new bot
2. Copy the bot token
3. Send a message to your bot, then use the Telegram API to find your chat ID

### ntfy

Send notifications to an [ntfy](https://ntfy.sh) topic.

```toml
[[notifications]]
type = "ntfy"
server_url = "https://ntfy.sh"
topic = "foxd-lan-monitor"
# Optional authentication token
# token = "tk_YOUR_TOKEN_HERE"
```

### Webhook

Send a POST request to any URL.

```toml
[[notifications]]
type = "webhook"
url = "https://example.com/webhook"
```

## Multiple Channels

You can configure as many channels as you want. Each `[[notifications]]` block defines one channel.

```toml
[[notifications]]
type = "telegram"
bot_token = "TOKEN_1"
chat_id = "CHAT_1"

[[notifications]]
type = "ntfy"
server_url = "https://ntfy.sh"
topic = "foxd-alerts"

[[notifications]]
type = "webhook"
url = "https://example.com/hook"
```

## Rules

Notifications are triggered by rules. Rules are managed through the web console or the REST API.

Each rule has:

| Field                   | Description                                                 |
| ----------------------- | ----------------------------------------------------------- |
| `name`                  | Human-readable name for the rule                            |
| `description`           | Optional description                                        |
| `trigger_type`          | Event that fires the rule                                   |
| `mac_filter`            | Optional MAC address to limit the rule to a specific device |
| `enabled`               | Whether the rule is active                                  |
| `notification_channels` | Which notification channels to use                          |

### Trigger Types

| Type                   | Fires when                               |
| ---------------------- | ---------------------------------------- |
| `new_device`           | A device is seen for the first time      |
| `device_connected`     | A device comes online                    |
| `device_disconnected`  | A device goes offline                    |
| `device_status_change` | A device changes status (online/offline) |
