# Fox Daemon (foxd-daemon)

A comprehensive LAN monitoring daemon built in Rust that captures network events, tracks device presence, and sends notifications through multiple channels.

## Features

- **Real-time Network Monitoring**
  - ARP packet capture for device discovery
  - DHCP request monitoring
  - Netlink integration for kernel neighbor table events
  - Automatic device online/offline detection

- **HTTP REST API**
  - Device management and status tracking
  - Rule-based notification configuration
  - Real-time metrics and health checks
  - Configuration hot-reload

- **Persistent Storage**
  - SQLite database for device history
  - Rule definitions
  - Channel configurations

- **Multi-Channel Notifications**
  - Telegram Bot API
  - ntfy.sh push notifications
  - Custom webhooks

- **Production Ready**
  - Systemd service integration
  - Docker containerization
  - Comprehensive logging with tracing
  - Integration tests

## Architecture

```
foxd-daemon/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library exports
│   ├── models.rs        # Data structures
│   ├── errors.rs        # Error handling
│   ├── db.rs            # Database operations
│   ├── daemon.rs        # Packet capture & netlink monitoring
│   ├── notifier.rs      # Notification adapters
│   └── api.rs           # HTTP API server
├── tests/
│   └── integration_tests.rs
├── Cargo.toml
├── Dockerfile
├── foxd-daemon.service  # Systemd service
└── config.toml.example  # Configuration template
```

## Prerequisites

### System Requirements

- Linux (for netlink and packet capture)
- libpcap development headers
- SQLite (included in Rust build)

### Installing Dependencies

**Debian/Ubuntu:**

```bash
sudo apt-get update
sudo apt-get install libpcap-dev pkg-config libssl-dev
```

**Fedora/RHEL:**

```bash
sudo dnf install libpcap-devel openssl-devel
```

**Arch Linux:**

```bash
sudo pacman -S libpcap openssl
```

### Rust

Install Rust using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

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

# Add notification channels
[[notifications]]
type = "telegram"
bot_token = "YOUR_BOT_TOKEN"
chat_id = "YOUR_CHAT_ID"

[[notifications]]
type = "ntfy"
server_url = "https://ntfy.sh"
topic = "your-topic"
```

### Setting up Telegram Notifications

1. Create a bot with [@BotFather](https://t.me/botfather) on Telegram
2. Get your bot token
3. Start a chat with your bot
4. Get your chat ID by visiting: `https://api.telegram.org/bot<YOUR_BOT_TOKEN>/getUpdates`
5. Add credentials to `config.toml`

### Setting up ntfy.sh Notifications

1. Choose a unique topic name (e.g., `foxd-alerts-12345`)
2. Subscribe to it on your phone using the [ntfy app](https://ntfy.sh/)
3. Add to `config.toml`:

```toml
[[notifications]]
type = "ntfy"
server_url = "https://ntfy.sh"
topic = "your-unique-topic"
```

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

## API Endpoints

### Health Check

```bash
GET /health
```

### Devices

List all devices:

```bash
curl http://localhost:8080/devices
```

Get specific device:

```bash
curl http://localhost:8080/devices/aa:bb:cc:dd:ee:ff
```

### Rules

List all rules:

```bash
curl http://localhost:8080/rules
```

Create a rule:

```bash
curl -X POST http://localhost:8080/rules \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Notify on new device",
    "description": "Alert when any new device joins",
    "trigger_type": "new_device",
    "mac_filter": null,
    "enabled": true,
    "notification_channels": ["telegram_123456"]
  }'
```

Update a rule:

```bash
curl -X POST http://localhost:8080/rules/1 \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Updated rule",
    "trigger_type": "device_connected",
    "enabled": true,
    "notification_channels": ["ntfy_mytopic"]
  }'
```

Delete a rule:

```bash
curl -X POST http://localhost:8080/rules/1/delete
```

### Configuration

Get current config:

```bash
curl http://localhost:8080/config
```

Update config:

```bash
curl -X POST http://localhost:8080/config \
  -H "Content-Type: application/json" \
  -d '{
    "notifications": [
      {
        "type": "telegram",
        "bot_token": "NEW_TOKEN",
        "chat_id": "CHAT_ID"
      }
    ]
  }'
```

### Metrics

```bash
curl http://localhost:8080/metrics
```

Returns:

```json
{
  "total_devices": 15,
  "online_devices": 8,
  "offline_devices": 7,
  "total_rules": 3,
  "enabled_rules": 2,
  "packets_captured": 45231,
  "notifications_sent": 12,
  "uptime_seconds": 3600
}
```

## Systemd Service

### Installation

1. Build the release binary:

```bash
cargo build --release
```

2. Copy binary to system location:

```bash
sudo cp target/release/foxd-daemon /usr/local/bin/
```

3. Create config directory:

```bash
sudo mkdir -p /etc/foxd
sudo cp config.toml /etc/foxd/
```

4. Install systemd service:

```bash
sudo cp foxd-daemon.service /etc/systemd/system/
sudo systemctl daemon-reload
```

5. Enable and start service:

```bash
sudo systemctl enable foxd-daemon
sudo systemctl start foxd-daemon
```

### Service Management

Check status:

```bash
sudo systemctl status foxd-daemon
```

View logs:

```bash
sudo journalctl -u foxd-daemon -f
```

Restart:

```bash
sudo systemctl restart foxd-daemon
```

## Docker Deployment

### Build Image

```bash
docker build -t foxd-daemon .
```

### Run Container

```bash
docker run -d \
  --name foxd \
  --net=host \
  --cap-add=NET_ADMIN \
  --cap-add=NET_RAW \
  -v $(pwd)/config.toml:/app/config.toml \
  -v foxd-data:/data \
  foxd-daemon
```

**Important:**

- `--net=host` is required for packet capture
- `--cap-add=NET_ADMIN` and `--cap-add=NET_RAW` are required for network access
- Mount your config file and a volume for the database

### Docker Compose

```yaml
version: "3.8"
services:
  foxd:
    build: .
    container_name: foxd-daemon
    network_mode: host
    cap_add:
      - NET_ADMIN
      - NET_RAW
    volumes:
      - ./config.toml:/app/config.toml
      - foxd-data:/data
    environment:
      - RUST_LOG=foxd_daemon=info
    restart: unless-stopped

volumes:
  foxd-data:
```

## Testing

Run tests:

```bash
cargo test
```

Run integration tests:

```bash
cargo test --test integration_tests
```

## Database Schema

### Devices Table

```sql
CREATE TABLE devices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    mac_address TEXT NOT NULL UNIQUE,
    ip_address TEXT,
    hostname TEXT,
    vendor TEXT,
    first_seen TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'unknown'
);
```

### Rules Table

```sql
CREATE TABLE rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    trigger_type TEXT NOT NULL,
    mac_filter TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    notification_channels TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

## Troubleshooting

### Permission Denied

Packet capture requires root privileges:

```bash
sudo ./foxd-daemon
```

Or set capabilities:

```bash
sudo setcap cap_net_raw,cap_net_admin=eip ./foxd-daemon
```

### Interface Not Found

List available interfaces:

```bash
ip link show
```

Update `config.toml` with the correct interface name.

### Database Locked

Only one instance can run at a time. Stop other instances:

```bash
sudo systemctl stop foxd-daemon
```

### No Packets Captured

Check interface is up and has traffic:

```bash
sudo tcpdump -i eth0 arp
```

## Performance Considerations

- **CPU Usage**: Minimal in steady state, spikes during heavy ARP traffic
- **Memory**: ~20-50 MB depending on device count
- **Disk**: Database grows with device history
- **Network**: Captures only ARP and DHCP packets (minimal overhead)

## Security Considerations

- Daemon requires root/CAP_NET_RAW for packet capture
- API server has no authentication by default (add reverse proxy with auth)
- Telegram bot tokens and webhook URLs should be kept secure
- Consider running behind a firewall and only exposing API locally

## Roadmap

- [ ] Web UI dashboard
- [ ] MAC address vendor lookup
- [ ] Device naming/aliasing
- [ ] Historical presence tracking
- [ ] Export metrics to Prometheus
- [ ] Email notification support
- [ ] IPv6 support
- [ ] Custom packet filters

## License

MIT License - see LICENSE file for details

## Contributing

Contributions welcome! Please open issues for bugs or feature requests.

## Support

For issues, questions, or contributions, please use the GitHub issue tracker.
