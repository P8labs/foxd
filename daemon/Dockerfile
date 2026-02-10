# Multi-stage build for Fox Daemon

# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    libpcap-dev \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libpcap0.8 \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/foxd-daemon /usr/local/bin/foxd-daemon

# Copy config example
COPY config.toml.example /app/config.toml.example

# Create data directory
RUN mkdir -p /data

# Environment variables
ENV FOXD_CONFIG=/app/config.toml
ENV FOXD_DB_PATH=/data/foxd.db
ENV RUST_LOG=foxd_daemon=info

# Expose API port
EXPOSE 8080

# Volume for database and config
VOLUME ["/data", "/app"]

# Note: The container needs to run with --net=host for packet capture
# and --cap-add=NET_ADMIN --cap-add=NET_RAW for network access

CMD ["foxd-daemon"]
