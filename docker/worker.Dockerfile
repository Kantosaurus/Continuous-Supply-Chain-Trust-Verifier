# =============================================================================
# Dockerfile for SCTV Worker
# =============================================================================
# Optimized build for the background job processor.
#
# Build:
#   docker build -f docker/worker.Dockerfile -t sctv-worker .
#
# Run:
#   docker run -e DATABASE_URL=postgres://... sctv-worker
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Build
# -----------------------------------------------------------------------------
FROM rust:slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests and source
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY migrations ./migrations

# Build application
RUN cargo build --release --bin sctv-worker

# -----------------------------------------------------------------------------
# Stage 2: Runtime
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd --create-home --shell /bin/bash sctv
WORKDIR /home/sctv

# Copy binary
COPY --from=builder /app/target/release/sctv-worker /usr/local/bin/sctv-worker

USER sctv

# Environment defaults
ENV SCTV_WORKER_COUNT=4
ENV SCTV_POLL_INTERVAL_MS=1000
ENV SCTV_SHUTDOWN_TIMEOUT_SECS=30
ENV SCTV_LOG_FORMAT=json
ENV RUST_LOG=info,sctv_worker=info

CMD ["sctv-worker"]
