# =============================================================================
# Multi-stage Dockerfile for SCTV (Supply Chain Trust Verifier)
# =============================================================================
# This Dockerfile builds all SCTV services in a single multi-stage build
# to maximize layer caching and minimize final image size.
#
# Build targets:
#   - sctv-api: REST/GraphQL API server
#   - sctv-worker: Background job processor
#   - sctv-cli: Command-line interface
#   - sctv-dashboard: Web dashboard (Leptos SSR)
#
# Usage:
#   docker build --target sctv-api -t sctv-api .
#   docker build --target sctv-worker -t sctv-worker .
#   docker build --target sctv-cli -t sctv-cli .
#   docker build --target sctv-dashboard -t sctv-dashboard .
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Build all binaries
# -----------------------------------------------------------------------------
FROM rust:slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY migrations ./migrations

# Build all binaries in release mode
RUN cargo build --release --bin sctv-api && \
    cargo build --release --bin sctv-worker && \
    cargo build --release --bin sctv && \
    cargo build --release --bin sctv-dashboard --features ssr

# -----------------------------------------------------------------------------
# Stage 2: Runtime base image
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime-base

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd --create-home --shell /bin/bash sctv
WORKDIR /home/sctv

# -----------------------------------------------------------------------------
# Stage 3: SCTV API Server
# -----------------------------------------------------------------------------
FROM runtime-base AS sctv-api

# Install curl for healthcheck
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sctv-api /usr/local/bin/sctv-api
COPY --from=builder /app/migrations /home/sctv/migrations

USER sctv

# Default environment variables
ENV SCTV_HOST=0.0.0.0
ENV SCTV_PORT=3000
ENV SCTV_LOG_FORMAT=json
ENV RUST_LOG=info,sctv_api=debug

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

ENTRYPOINT ["sctv-api"]

# -----------------------------------------------------------------------------
# Stage 4: SCTV Worker
# -----------------------------------------------------------------------------
FROM runtime-base AS sctv-worker

COPY --from=builder /app/target/release/sctv-worker /usr/local/bin/sctv-worker

USER sctv

# Default environment variables
ENV SCTV_WORKER_COUNT=4
ENV SCTV_POLL_INTERVAL_MS=1000
ENV SCTV_LOG_FORMAT=json
ENV RUST_LOG=info,sctv_worker=debug

ENTRYPOINT ["sctv-worker"]

# -----------------------------------------------------------------------------
# Stage 5: SCTV CLI
# -----------------------------------------------------------------------------
FROM runtime-base AS sctv-cli

COPY --from=builder /app/target/release/sctv /usr/local/bin/sctv

USER sctv

ENV RUST_LOG=info

ENTRYPOINT ["sctv"]

# -----------------------------------------------------------------------------
# Stage 6: SCTV Dashboard
# -----------------------------------------------------------------------------
FROM runtime-base AS sctv-dashboard

# Install curl for healthcheck
RUN apt-get update && apt-get install -y curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sctv-dashboard /usr/local/bin/sctv-dashboard

USER sctv

ENV SCTV_HOST=0.0.0.0
ENV SCTV_PORT=3000
ENV RUST_LOG=info,sctv_dashboard=debug

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/ || exit 1

ENTRYPOINT ["sctv-dashboard"]
