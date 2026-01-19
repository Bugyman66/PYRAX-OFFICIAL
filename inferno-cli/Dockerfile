# Inferno Node Docker Image
# Multi-stage build for minimal image size

# Build stage
FROM rust:1.75-bookworm AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libclang-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY . .

# Build both binaries from workspace root
# Cargo workspace puts all binaries in workspace root target directory
WORKDIR /build
RUN cargo build --release -p pyrax-node --bin pyrax-node
RUN cargo build --release -p inferno-cli --bin inferno

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false -m -d /var/lib/inferno inferno

# Copy binaries from builder (workspace root target directory)
COPY --from=builder /build/target/release/pyrax-node /usr/local/bin/pyrax-node
COPY --from=builder /build/target/release/inferno /usr/local/bin/inferno

# Set permissions
RUN chmod +x /usr/local/bin/pyrax-node /usr/local/bin/inferno

# Create data directory
RUN mkdir -p /data && chown inferno:inferno /data

# Switch to non-root user
USER inferno

# Set working directory
WORKDIR /data

# Environment variables
ENV RUST_LOG=info
ENV INFERNO_DATA_DIR=/data

# Expose ports
# P2P port
EXPOSE 30303/tcp
EXPOSE 30303/udp
# RPC port
EXPOSE 28545/tcp
# Dashboard port
EXPOSE 8080/tcp

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:28545 || exit 1

# Default command
ENTRYPOINT ["pyrax-node"]
CMD ["--network", "devnet", "--p2p-port", "30303", "--rpc-port", "28545", "--rpc", "--data-dir", "/data"]

# Labels
LABEL org.opencontainers.image.title="Inferno Node"
LABEL org.opencontainers.image.description="PYRAX Blockchain Node"
LABEL org.opencontainers.image.vendor="PYRAX Chain"
LABEL org.opencontainers.image.source="https://github.com/PYRAX-Chain/PYRAX-OFFICIAL"
LABEL org.opencontainers.image.licenses="MIT"
