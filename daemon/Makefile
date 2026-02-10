.PHONY: help build build-release run test clean install docker-build docker-run

help:
	@echo "Fox Daemon - Makefile targets:"
	@echo "  build         - Build debug binary"
	@echo "  build-release - Build optimized release binary"
	@echo "  run           - Run daemon in development mode (requires sudo)"
	@echo "  test          - Run all tests"
	@echo "  clean         - Clean build artifacts"
	@echo "  install       - Install systemd service (requires sudo)"
	@echo "  docker-build  - Build Docker image"
	@echo "  docker-run    - Run Docker container"

build:
	cargo build

build-release:
	cargo build --release

run:
	sudo RUST_LOG=foxd_daemon=debug cargo run

test:
	cargo test

clean:
	cargo clean
	rm -f *.db *.db-journal

install: build-release
	@echo "Installing foxd-daemon..."
	sudo cp target/release/foxd-daemon /usr/local/bin/
	sudo mkdir -p /etc/foxd
	sudo cp config.toml.example /etc/foxd/config.toml.example
	@if [ ! -f /etc/foxd/config.toml ]; then \
		echo "Creating default config..."; \
		sudo cp config.toml.example /etc/foxd/config.toml; \
	fi
	sudo cp foxd-daemon.service /etc/systemd/system/
	sudo systemctl daemon-reload
	@echo "Installation complete!"
	@echo "Edit /etc/foxd/config.toml and then run:"
	@echo "  sudo systemctl enable foxd-daemon"
	@echo "  sudo systemctl start foxd-daemon"

docker-build:
	docker build -t foxd-daemon:latest .

docker-run:
	docker-compose up -d

docker-stop:
	docker-compose down

docker-logs:
	docker-compose logs -f
