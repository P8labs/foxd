# Foxd

This is a LAN listener that can be used for many things for example:

- We can get notification if someone connects to network and identify them that mean if my car arrives home automatically it will connect to home network and I will get notified that it arrived open the garage.
- Or we can looks if any unidentified device is connected to network.
- We can also monitor IOT devices and chech thier health by looking if they are connected to wifi, etc.

# Installation

Foxd can be installed in two ways:

- **Recommended:** Install script (native binary)
- **Alternative:** Docker container

## Option 1 — Install via Script (Recommended)

```bash
curl -fsSL https://foxd.p8labs.in/install.sh | sudo bash
```

### What this does

- Detects your system architecture
- Downloads latest release from GitHub
- Installs binary to `/usr/local/bin/foxd`
- Creates `/etc/foxd/config.toml`
- Sets up and starts a systemd service

## Run manually

```bash
sudo foxd
```

## Default config location

```bash
/etc/foxd/config.toml
```

Example:

```toml
[daemon]
interface = "auto"

[database]
path = "/etc/foxd/foxd.db"

[api]
host = "0.0.0.0"
port = 3090
```

## Option 2 — Run with Docker

Foxd is available as a multi-arch image on GHCR.

### Quick start

```bash
docker run -d \
  --name foxd \
  --network host \
  --cap-add NET_RAW \
  --cap-add NET_ADMIN \
  ghcr.io/p8labs/foxd:latest
```

### Use a specific version

```bash
docker run -d \
  --network host \
  --cap-add NET_RAW \
  --cap-add NET_ADMIN \
  ghcr.io/p8labs/foxd:v1.2.1
```

### Persist config (recommended)

```bash
docker run -d \
  --name foxd \
  --network host \
  --cap-add NET_RAW \
  --cap-add NET_ADMIN \
  -v /etc/foxd:/etc/foxd \
  ghcr.io/p8labs/foxd:latest
```

## Docker Configuration

Foxd uses:

```bash
/etc/foxd/config.toml
```

### Create config on host

```bash
sudo mkdir -p /etc/foxd
sudo nano /etc/foxd/config.toml
```

Example:

```toml
[daemon]
interface = "auto"

[database]
path = "/etc/foxd/foxd.db"

[api]
host = "0.0.0.0"
port = 3090
```

### Apply changes

```bash
docker restart foxd
```

## Notes

- `interface = "auto"` automatically selects the active network interface.
- `--network host` is required to monitor LAN traffic.
- `NET_RAW` + `NET_ADMIN` are required for packet capture and netlink access.
- Without volume mount, config and database will be lost on container recreate.

## Access Web Console

Once running:

```text
http://<your-ip>:3090
```

## Uninstall

If installed via script:

```bash
curl -fsSL https://foxd.p8labs.in/install.sh | sudo bash
```

## Docs

Full documentation: [https://foxd.p8labs.in](https://foxd.p8labs.in)

# How It Works

This project for now has two parts a console and a daemon that can be hosted on home server through a docker container or directly.

### Daemon

The work of the daemon is to listen for LAN events and notify to channels. Also serves a API server that console can utilize to show it in UI. I am writing it in rust because it can than be compiled for bare metal.

### Console

This is a simple Svelte web app where admin can see all activites and manage them. we can add rules and other configs through website.

### AI Disclosure

> AI is used to generate docs for this project also used autocompletion in console (web) part. For Daemon no editor AI is used. Browser ChatGPT, Google is used to research and fix some bugs.
