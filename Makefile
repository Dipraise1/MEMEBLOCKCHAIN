# MemeChain Makefile
# High-performance Layer 1 blockchain for NFTs and meme tokens

.PHONY: help install build test clean devnet faucet explorer docs

# Default target
help:
	@echo "MemeChain - High-performance Layer 1 blockchain for NFTs and meme tokens"
	@echo ""
	@echo "Available commands:"
	@echo "  install     - Install dependencies and build tools"
	@echo "  build       - Build the blockchain binary"
	@echo "  test        - Run all tests"
	@echo "  clean       - Clean build artifacts"
	@echo "  devnet      - Start local development network"
	@echo "  faucet      - Start testnet faucet"
	@echo "  explorer    - Start block explorer"
	@echo "  docs        - Generate documentation"
	@echo "  docker      - Build and run with Docker"
	@echo "  lint        - Run code linting"
	@echo "  format      - Format code"
	@echo "  security    - Run security checks"

# Install dependencies
install:
	@echo "Installing dependencies..."
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
	source ~/.cargo/env && cargo install cargo-watch
	source ~/.cargo/env && cargo install cargo-audit
	source ~/.cargo/env && cargo install cargo-tarpaulin
	@echo "Dependencies installed successfully!"

# Build the blockchain
build:
	@echo "Building MemeChain..."
	cargo build --release
	@echo "Build completed! Binary available at target/release/memechain"

# Run tests
test:
	@echo "Running tests..."
	cargo test --all-features
	cargo test --doc

# Run tests with coverage
test-coverage:
	@echo "Running tests with coverage..."
	cargo tarpaulin --out Html --output-dir coverage

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -rf data/
	rm -f config.toml genesis.json
	@echo "Clean completed!"

# Start local development network
devnet: build
	@echo "Starting local development network..."
	@if [ ! -f config.toml ]; then \
		echo "Initializing blockchain..."; \
		./target/release/memechain init --chain-id memechain-dev --moniker validator; \
	fi
	./target/release/memechain start --config config.toml

# Start testnet faucet
faucet:
	@echo "Starting testnet faucet..."
	cd tools/faucet && npm install && npm start

# Start block explorer
explorer:
	@echo "Starting block explorer..."
	cd tools/explorer && npm install && npm start

# Generate documentation
docs:
	@echo "Generating documentation..."
	cargo doc --no-deps --open

# Build and run with Docker
docker:
	@echo "Building Docker image..."
	docker build -t memechain .
	@echo "Running MemeChain in Docker..."
	docker run -p 8080:8080 -p 26657:26657 memechain

# Run code linting
lint:
	@echo "Running code linting..."
	cargo clippy --all-features -- -D warnings
	cargo fmt -- --check

# Format code
format:
	@echo "Formatting code..."
	cargo fmt

# Run security checks
security:
	@echo "Running security checks..."
	cargo audit
	cargo clippy --all-features -- -D warnings

# Watch for changes and rebuild
watch:
	@echo "Watching for changes..."
	cargo watch -x check -x test -x run

# Create release
release: clean build test security
	@echo "Creating release..."
	tar -czf memechain-$(shell git describe --tags --always).tar.gz target/release/memechain config.toml genesis.json
	@echo "Release created: memechain-$(shell git describe --tags --always).tar.gz"

# Setup development environment
setup-dev: install
	@echo "Setting up development environment..."
	@if [ ! -f config.toml ]; then \
		./target/release/memechain init --chain-id memechain-dev --moniker validator; \
	fi
	@echo "Development environment ready!"

# Run integration tests
integration-test: build
	@echo "Running integration tests..."
	./scripts/integration-tests.sh

# Performance benchmarks
benchmark: build
	@echo "Running performance benchmarks..."
	cargo bench

# Generate API documentation
api-docs:
	@echo "Generating API documentation..."
	cd sdk/js && npm run docs
	cd sdk/rust && cargo doc --no-deps

# Deploy to testnet
deploy-testnet: build
	@echo "Deploying to testnet..."
	./scripts/deploy-testnet.sh

# Deploy to mainnet
deploy-mainnet: build
	@echo "Deploying to mainnet..."
	./scripts/deploy-mainnet.sh

# Backup data
backup:
	@echo "Backing up blockchain data..."
	tar -czf backup-$(shell date +%Y%m%d-%H%M%S).tar.gz data/ config.toml genesis.json

# Restore data
restore:
	@echo "Restoring blockchain data..."
	@if [ -f "$(BACKUP_FILE)" ]; then \
		tar -xzf $(BACKUP_FILE); \
		echo "Data restored from $(BACKUP_FILE)"; \
	else \
		echo "Please specify BACKUP_FILE=<filename>"; \
		exit 1; \
	fi

# Monitor blockchain
monitor:
	@echo "Starting blockchain monitoring..."
	./scripts/monitor.sh

# Update dependencies
update-deps:
	@echo "Updating dependencies..."
	cargo update
	cargo audit --fix

# Check for vulnerabilities
audit:
	@echo "Checking for vulnerabilities..."
	cargo audit

# Run specific module tests
test-nft:
	@echo "Running NFT module tests..."
	cargo test --package memechain --lib modules::nft

test-meme:
	@echo "Running Meme token module tests..."
	cargo test --package memechain --lib modules::meme

test-common:
	@echo "Running Common module tests..."
	cargo test --package memechain --lib modules::common

# Generate test data
generate-test-data:
	@echo "Generating test data..."
	./scripts/generate-test-data.sh

# Validate configuration
validate-config:
	@echo "Validating configuration..."
	@if [ -f config.toml ]; then \
		./target/release/memechain validate-config --config config.toml; \
	else \
		echo "No config.toml found. Run 'make setup-dev' first."; \
	fi

# Show blockchain status
status:
	@echo "Blockchain status:"
	@if [ -f data/blockchain.db ]; then \
		echo "Database: Found"; \
	else \
		echo "Database: Not found"; \
	fi
	@if [ -f config.toml ]; then \
		echo "Config: Found"; \
	else \
		echo "Config: Not found"; \
	fi
	@if [ -f genesis.json ]; then \
		echo "Genesis: Found"; \
	else \
		echo "Genesis: Not found"; \
	fi 