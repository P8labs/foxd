# Getting Started

## Requirements

**Linux:**

- Linux OS (uses netlink for neighbor table monitoring)
- `libpcap` installed on the system
- Root or `CAP_NET_RAW` / `CAP_NET_ADMIN` capabilities for packet capture

## Install Foxd

Install using the official script:

```bash
curl -fsSL https://foxd.p8labs.in/install.sh | sudo bash
```

The installer will:

- Detect your system architecture
- Download the latest release from GitHub
- Install the binary to `/usr/local/bin/foxd`
- Create `/etc/foxd/`
- Generate a default configuration
- Create and enable a systemd service
- Start foxd automatically

## Run with Docker (Quick Setup)

Foxd is available as a prebuilt multi-arch Docker image on **GHCR**.

### Quick start

```bash
mkdir -p ~/foxd-data

docker run -d \
  --name foxd \
  --network host \
  --cap-add NET_RAW \
  --cap-add NET_ADMIN \
  -v ~/foxd-data:/data \
  -e DB_PATH=/data/foxd.db \
  ghcr.io/p8labs/foxd:latest
```

### Why these flags are required

- `--network host` → allows foxd to see real LAN traffic (not container-only network)
- `NET_RAW` → required for packet capture (libpcap)
- `NET_ADMIN` → required for netlink + interface monitoring

Without these, foxd will not function correctly.

### Use a specific version

```bash
mkdir -p ~/foxd-data

docker run -d \
  --name foxd \
  --network host \
  --cap-add NET_RAW \
  --cap-add NET_ADMIN \
  -v ~/foxd-data:/data \
  -e DB_PATH=/data/foxd.db \
  ghcr.io/p8labs/foxd:v1.2.1
```

### Notes

- The container runs as a non-root user but still requires Linux capabilities.
- API will be available on the host at `http://<host>:3090`.

## Install from Binary

Download the latest release binaries from the github [releases](https://github.com/P8labs/foxd/releases).

```bash
# Example for Linux
curl -L -o foxd https://github.com/P8labs/foxd/releases/download/latest/foxd-v1.1.2-linux-amd64
chmod +x
sudo setcap cap_net_raw,cap_net_admin=eip ./foxd
sudo mv foxd /usr/local/bin/
```

## Build from Source

You need Rust, Node.js, and pnpm installed.

```bash
git clone https://github.com/P8labs/foxd.git
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

Once foxd is running, open your browser to `http://<host>:8080` to access the built-in web console. The console lets you view devices, manage rules, configure notification channels, and check logs.

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
