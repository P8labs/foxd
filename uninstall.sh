#!/usr/bin/env bash

set -euo pipefail

BINARY_NAME="foxd"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="/etc/foxd"
SERVICE_NAME="foxd"
SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"

log() {
  echo -e "\033[1;34m[foxd]\033[0m $1"
}

warn() {
  echo -e "\033[1;33m[foxd]\033[0m $1"
}

error() {
  echo -e "\033[1;31mError:\033[0m $1"
  exit 1
}

require_root() {
  if [ "$EUID" -ne 0 ]; then
    error "Please run as root (use sudo)."
  fi
}

stop_service() {
  if systemctl list-units --full -all | grep -q "${SERVICE_NAME}.service"; then
    log "Stopping service..."
    systemctl stop "$SERVICE_NAME" || true
  fi
}

disable_service() {
  if systemctl list-unit-files | grep -q "${SERVICE_NAME}.service"; then
    log "Disabling service..."
    systemctl disable "$SERVICE_NAME" || true
  fi
}

remove_service_file() {
  if [ -f "$SERVICE_FILE" ]; then
    log "Removing systemd service file..."
    rm -f "$SERVICE_FILE"
    systemctl daemon-reload
  fi
}

remove_binary() {
  if [ -f "$INSTALL_DIR/$BINARY_NAME" ]; then
    log "Removing binary..."
    rm -f "$INSTALL_DIR/$BINARY_NAME"
  else
    warn "Binary not found at $INSTALL_DIR/$BINARY_NAME"
  fi
}

remove_config() {
  if [ -d "$CONFIG_DIR" ]; then
    read -rp "Remove config directory ($CONFIG_DIR)? [y/N]: " confirm
    if [[ "$confirm" =~ ^[Yy]$ ]]; then
      log "Removing config directory..."
      rm -rf "$CONFIG_DIR"
    else
      warn "Keeping config directory."
    fi
  fi
}

main() {
  require_root

  log "Uninstalling foxd..."

  stop_service
  disable_service
  remove_service_file
  remove_binary
  remove_config

  log "Uninstall complete."
}

main "$@"