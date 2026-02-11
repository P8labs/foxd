# Notifications

foxd can send notifications when network events occur. Notification channels are stored in the database and managed from the web console or the REST API. Rules reference these channels by name.

## Supported Channels

### Telegram

Send notifications via a Telegram bot.

To set up a Telegram bot:

1. Message [@BotFather](https://t.me/BotFather) on Telegram and create a new bot
2. Copy the bot token
3. Send a message to your bot, then use the Telegram API to find your chat ID

### ntfy

Send notifications to an [ntfy](https://ntfy.sh) topic.

### Webhook

Send a POST request to any URL.

## Managing Channels

Use the console UI (Configuration -> Notification Channels) to add or remove channels. You can also manage channels via the REST API. See the [Notification Channel Management](/guide/notification-channels) guide for examples and payloads.

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
