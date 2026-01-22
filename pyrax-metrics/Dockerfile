# Build stage
FROM rust:1.75-bookworm as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release --bin pyrax-metrics

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/pyrax-metrics /usr/local/bin/pyrax-metrics

# Copy default config
COPY config /config

# Expose ports
EXPOSE 9092 8080

# Set default command
CMD ["pyrax-metrics", "--config", "/config/devnet.toml"]
