#!/bin/bash

# MemeChain Quick Start Script
# This script sets up a local development environment for MemeChain

set -e

echo "🚀 MemeChain Quick Start"
echo "========================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if required tools are installed
check_requirements() {
    print_status "Checking system requirements..."
    
    # Check Rust
    if ! command -v rustc &> /dev/null; then
        print_error "Rust is not installed. Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source ~/.cargo/env
    else
        print_success "Rust is installed: $(rustc --version)"
    fi
    
    # Check Cargo
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed. Please install Rust first."
        exit 1
    else
        print_success "Cargo is installed: $(cargo --version)"
    fi
    
    # Check Docker (optional)
    if command -v docker &> /dev/null; then
        print_success "Docker is installed: $(docker --version)"
    else
        print_warning "Docker is not installed. Some features may not be available."
    fi
    
    # Check Node.js (optional)
    if command -v node &> /dev/null; then
        print_success "Node.js is installed: $(node --version)"
    else
        print_warning "Node.js is not installed. Some tools may not be available."
    fi
}

# Install dependencies
install_dependencies() {
    print_status "Installing dependencies..."
    
    # Install Rust tools
    cargo install cargo-watch
    cargo install cargo-audit
    cargo install cargo-tarpaulin
    
    print_success "Dependencies installed successfully!"
}

# Build the project
build_project() {
    print_status "Building MemeChain..."
    
    cd chain
    cargo build --release
    cd ..
    
    print_success "Build completed! Binary available at chain/target/release/memechain"
}

# Initialize blockchain
initialize_blockchain() {
    print_status "Initializing blockchain..."
    
    if [ ! -f config.toml ]; then
        ./chain/target/release/memechain init --chain-id memechain-dev --moniker validator
        print_success "Blockchain initialized with config.toml and genesis.json"
    else
        print_warning "config.toml already exists. Skipping initialization."
    fi
}

# Start development network
start_devnet() {
    print_status "Starting development network..."
    
    if [ ! -f config.toml ]; then
        print_error "config.toml not found. Please run initialization first."
        exit 1
    fi
    
    print_success "Starting MemeChain node..."
    print_status "API will be available at http://localhost:8080"
    print_status "RPC will be available at http://localhost:26657"
    print_status "Press Ctrl+C to stop the node"
    
    ./chain/target/release/memechain start --config config.toml
}

# Run tests
run_tests() {
    print_status "Running tests..."
    
    cd chain
    cargo test --all-features
    cd ..
    
    print_success "Tests completed!"
}

# Show help
show_help() {
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  setup      - Complete setup (requirements, dependencies, build, init)"
    echo "  build      - Build the project"
    echo "  init       - Initialize blockchain"
    echo "  start      - Start development network"
    echo "  test       - Run tests"
    echo "  clean      - Clean build artifacts"
    echo "  help       - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 setup   # Complete setup"
    echo "  $0 start   # Start devnet (requires setup first)"
}

# Clean build artifacts
clean_project() {
    print_status "Cleaning build artifacts..."
    
    cd chain
    cargo clean
    cd ..
    
    rm -rf data/
    rm -f config.toml genesis.json
    
    print_success "Clean completed!"
}

# Main setup function
setup() {
    print_status "Starting complete setup..."
    
    check_requirements
    install_dependencies
    build_project
    initialize_blockchain
    
    print_success "Setup completed successfully!"
    echo ""
    echo "Next steps:"
    echo "1. Start the development network: $0 start"
    echo "2. Run tests: $0 test"
    echo "3. View API documentation: http://localhost:8080/health"
    echo ""
    echo "Useful commands:"
    echo "  make devnet    # Start with Makefile"
    echo "  make test      # Run tests with Makefile"
    echo "  make clean     # Clean with Makefile"
}

# Main script logic
case "${1:-help}" in
    "setup")
        setup
        ;;
    "build")
        build_project
        ;;
    "init")
        initialize_blockchain
        ;;
    "start")
        start_devnet
        ;;
    "test")
        run_tests
        ;;
    "clean")
        clean_project
        ;;
    "help"|*)
        show_help
        ;;
esac 