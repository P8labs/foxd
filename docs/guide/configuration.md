# Configuration

foxd is configured via a `config.toml` file. If no config file is found, foxd uses sensible defaults.

Notification channels are managed from the web console (Configuration -> Notification Channels) or the REST API, not from `config.toml`.

The config file path defaults to `config.toml` in the current directory. Override it with the `FOXD_CONFIG` environment variable.

## Full Example

```toml
[daemon]
# Network interface to monitor (e.g., eth0, wlan0, enp0s3)
interface = "eth0"

# Optional pcap capture filter (BPF syntax)
# capture_filter = "arp or (udp port 67 or udp port 68)"

# How often to check the neighbor table (seconds)
neighbor_check_interval_secs = 60

# How long before marking a device as offline (seconds)
device_timeout_secs = 300

# Automatic log cleanup
log_cleanup_enabled = true
log_retention_days = 30

[database]
# SQLite database file path
path = "./foxd.db"

[api]
# API server listen address
host = "0.0.0.0"

# API server port
port = 8080
```

## Daemon Section

### `interface`

The network interface to capture packets on. This should be the interface connected to the LAN you want to monitor.

```toml
interface = "eth0"
```

Find your interfaces with `ip link show`.

### `capture_filter`

Optional BPF filter for pcap. If omitted, foxd captures all traffic. Useful for reducing noise.

```toml
capture_filter = "arp or (udp port 67 or udp port 68)"
```

### `neighbor_check_interval_secs`

How often (in seconds) foxd polls the Linux neighbor table via netlink. Default: `30`.

```toml
neighbor_check_interval_secs = 60
```

### `device_timeout_secs`

How many seconds of inactivity before a device is marked offline. Default: `60`.

```toml
device_timeout_secs = 300
```

### `log_cleanup_enabled`

Enable or disable automatic deletion of old log entries. Default: `true`.

```toml
log_cleanup_enabled = true
```

### `log_retention_days`

How many days to keep log entries before they are automatically deleted. The cleanup runs once every 24 hours. Default: `30`.

```toml
log_retention_days = 30
```

## Database Section

### `path`

Path to the SQLite database file. Created automatically if it does not exist.

```toml
path = "./foxd.db"
```

## API Section

### `host`

IP address for the HTTP server to bind to. Use `0.0.0.0` to listen on all interfaces, or `127.0.0.1` for localhost only.

```toml
host = "0.0.0.0"
```

### `port`

Port for the HTTP server. Default: `8080`.

```toml
port = 8080
```
