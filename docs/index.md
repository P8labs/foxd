---
layout: home

hero:
  name: foxd
  text: LAN Monitoring Daemon
  tagline: A lightweight Rust daemon for observing local networks, tracking device presence, and reacting to changes in real time.
  actions:
    - theme: brand
      text: Get Started
      link: /guide/getting-started
    - theme: alt
      text: API Reference
      link: /api/

features:
  - title: Passive Device Discovery
    details: Continuously observes your LAN using ARP traffic, DHCP signals, and the Linux neighbor table to detect devices without active probing.
  - title: Real-Time State Tracking
    details: Maintains an internal state machine for each device, tracking joins, disconnects, and intermittent connectivity with minimal false positives.
  - title: Notifications & Automation
    details: Trigger notifications or actions when devices appear, disappear, or violate rules. Supports Telegram, ntfy.sh, webhooks, and API-driven outputs.
  - title: Embedded Web Console & API
    details: Ships with an embedded web UI and REST API for managing devices, rules, and configuration. No external web server required.
  - title: Single Static Binary
    details: Compiles into a single Rust binary. Designed to run reliably on Raspberry Pi, home servers, and bare-metal Linux systems.
---

## About foxd

foxd is a local network monitoring daemon written in Rust. It passively listens to LAN activity using packet capture and Linux netlink APIs to infer device presence, identity, and state changes.

Instead of relying on router integrations or cloud services, foxd observes the network directly. It correlates ARP traffic, DHCP activity, and kernel neighbor updates to build a real-time view of all devices on the network.

The daemon runs continuously in the background and exposes both a REST API and a built-in web console. Device state, rules, and configuration are stored locally, allowing foxd to restart cleanly and operate offline.

foxd is designed for environments where reliability, low overhead, and local control matter more than dashboards and vendor lock-in.

## About P8labs

P8labs is an independent engineering lab focused on building small, composable infrastructure tools.

The goal is not large platforms or cloud services, but durable software that runs close to the system, respects user control, and remains understandable over time. Projects from P8labs emphasize local-first operation, minimal dependencies, and clear internal architecture.

foxd reflects that philosophy: a focused daemon that does one job well, integrates cleanly with other systems, and stays out of the way once deployed.
