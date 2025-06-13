# MemeChain Dockerfile
# High-performance Layer 1 blockchain for NFTs and meme tokens

# Use Rust 1.70+ as base image
FROM rust:1.70-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libclang-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY chain/Cargo.toml chain/Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release

# Remove dummy main.rs and copy actual source code
RUN rm src/main.rs
COPY chain/src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Create memechain user
RUN useradd -r -s /bin/false memechain

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/memechain /usr/local/bin/

# Create data directory
RUN mkdir -p /app/data && chown memechain:memechain /app/data

# Copy configuration files
COPY config.toml /app/
COPY genesis.json /app/

# Switch to memechain user
USER memechain

# Expose ports
EXPOSE 8080 26657 26656

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Default command
CMD ["memechain", "start", "--config", "/app/config.toml"] 