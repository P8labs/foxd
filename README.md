# Foxd

This is a LAN listener that can be used for many things for example:

- We can get notification if someone connects to network and identify them that mean if my car arrives home automatically it will connect to home network and I will get notified that it arrived open the garage.
- Or we can looks if any unidentified device is connected to network.
- We can also monitor IOT devices and chech thier health by looking if they are connected to wifi, etc.

# How It Works

This project for now has two parts a console and a daemon that can be hosted on home server through a docker container or directly.

### Daemon

The work of the daemon is to listen for LAN events and notify to channels. Also serves a API server that console can utilize to show it in UI. I am writing it in rust because it can than be compiled for bare metal.

### Console

This is a simple Svelte web app where admin can see all activites and manage them. we can add rules and other configs through website.

## Configuration

1. Copy the example configuration:

```bash
cp config.toml.example config.toml
```

2. Edit `config.toml` with your settings:

```toml
[daemon]
interface = "eth0"  # Your network interface
neighbor_check_interval_secs = 60
device_timeout_secs = 300

[database]
path = "./foxd.db"

[api]
host = "0.0.0.0"
port = 8080
```

Notification channels are managed from the web console (Configuration -> Notification Channels) or the REST API. You no longer add them to `config.toml`.

### Setting up Telegram Notifications

1. Create a bot with [@BotFather](https://t.me/botfather) on Telegram
2. Get your bot token
3. Start a chat with your bot
4. Get your chat ID by visiting: `https://api.telegram.org/bot<YOUR_BOT_TOKEN>/getUpdates`
5. Add the channel in the console under Configuration -> Notification Channels

### Setting up ntfy.sh Notifications

1. Choose a unique topic name (e.g., `foxd-alerts-12345`)
2. Subscribe to it on your phone using the [ntfy app](https://ntfy.sh/)
3. Add the channel in the console under Configuration -> Notification Channels

## Building

### Development Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

The binary will be in `target/release/foxd-daemon`

## Running

### Development Mode

```bash
# Requires root for packet capture
sudo RUST_LOG=foxd_daemon=debug cargo run
```

### Production Mode

```bash
sudo ./target/release/foxd-daemon
```

### Environment Variables

- `FOXD_CONFIG` - Path to config file (default: `config.toml`)
- `FOXD_INTERFACE` - Network interface to monitor
- `FOXD_DB_PATH` - Database file path
- `FOXD_API_HOST` - API server host
- `FOXD_API_PORT` - API server port
- `RUST_LOG` - Logging level (e.g., `foxd_daemon=info`)

More [docs](https://p8labs.github.io/foxd).

### AI Disclosure

> AI is used to generate docs for this project also used autocompletion in console (web) part. For Daemon no editor AI is used. Browser ChatGPT, Google is used to research and fix some bugs.
