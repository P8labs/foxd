# Getting Started

## Requirements

- Linux (foxd uses netlink and libpcap, which are Linux-specific)
- `libpcap-dev` installed on the system
- Root or `CAP_NET_RAW` / `CAP_NET_ADMIN` capabilities for packet capture

## Install from Binary

Download the latest release for your platform from the [releases page](https://github.com/p8labs/foxd/releases).

```bash
# Example for Linux x86_64
curl -L -o foxd https://github.com/p8labs/foxd/releases/latest/download/foxd-linux-amd64
chmod +x foxd
sudo mv foxd /usr/local/bin/
```

Available binaries:

| Platform                   | File               |
| -------------------------- | ------------------ |
| Linux x86_64               | `foxd-linux-amd64` |
| Linux ARM64                | `foxd-linux-arm64` |
| Linux ARMv7 (Raspberry Pi) | `foxd-linux-armv7` |

## Build from Source

You need Rust, Node.js, and pnpm installed.

```bash
git clone https://github.com/p8labs/foxd.git
cd foxd
./build.sh
```

The final binary will be at `daemon/target/release/foxd`.

## Running

### Quick start

```bash
# Run with defaults (monitors wlan0, API on 127.0.0.1:8080)
sudo ./foxd
```

### With a config file

```bash
# Copy the example config
cp daemon/config.toml.example config.toml

# Edit to match your environment
vim config.toml

# Run
sudo ./foxd
```

### Environment variables

You can also configure foxd with environment variables instead of a config file:

| Variable         | Default       | Description                  |
| ---------------- | ------------- | ---------------------------- |
| `FOXD_CONFIG`    | `config.toml` | Path to config file          |
| `FOXD_INTERFACE` | `wlan0`       | Network interface to monitor |
| `FOXD_DB_PATH`   | `./foxd.db`   | SQLite database path         |
| `FOXD_API_HOST`  | `127.0.0.1`   | API listen address           |
| `FOXD_API_PORT`  | `8080`        | API listen port              |

### Web console

Once foxd is running, open your browser to `http://<host>:8080` to access the built-in web console. The console lets you view devices, manage rules, configure notifications, and check logs.

### Run as a systemd service

Create `/etc/systemd/system/foxd.service`:

```ini
[Unit]
Description=foxd LAN monitor
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/foxd
WorkingDirectory=/etc/foxd
Restart=on-failure
RestartSec=5
AmbientCapabilities=CAP_NET_RAW CAP_NET_ADMIN

[Install]
WantedBy=multi-user.target
```

```bash
sudo mkdir -p /etc/foxd
sudo cp config.toml /etc/foxd/
sudo systemctl daemon-reload
sudo systemctl enable --now foxd
```
