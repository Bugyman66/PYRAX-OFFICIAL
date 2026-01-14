# PYRAX Node Dockerfile
# Multi-stage build for production deployment

# Build stage
FROM rust:1.85-bookworm AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    clang \
    libclang-dev \
    librocksdb-dev \
    cmake \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files first for better caching
COPY Cargo.toml Cargo.lock ./

# Copy all workspace members (required for Cargo workspace resolution)
COPY pyrax-node ./pyrax-node
COPY pyrax-wallet ./pyrax-wallet
COPY pyrax-desktop/src-tauri ./pyrax-desktop/src-tauri

# Build only the node in release mode
RUN cargo build --release --package pyrax-node

# Runtime stage
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    librocksdb7.8 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false pyrax

# Copy the binary from builder
COPY --from=builder /build/target/release/pyrax-node /usr/local/bin/pyrax-node

# Copy genesis files
COPY pyrax-node/genesis /app/genesis

# Create data directories
RUN mkdir -p /app/data /app/dag_cache && chown -R pyrax:pyrax /app

USER pyrax

# Expose ports
# 8545 - HTTP JSON-RPC
# 8546 - WebSocket JSON-RPC
# 30303 - P2P TCP
# 30304 - P2P UDP
EXPOSE 8545 8546 30303 30304/udp

# Default command
ENTRYPOINT ["pyrax-node"]
CMD ["--network", "devnet", "--rpc", "--rpc-addr", "0.0.0.0:8545", "--datadir", "/app/data"]
