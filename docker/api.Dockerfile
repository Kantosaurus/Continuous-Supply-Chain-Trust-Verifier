# =============================================================================
# Dockerfile for SCTV API Server
# =============================================================================
# Optimized build for the REST/GraphQL API server.
#
# Build:
#   docker build -f docker/api.Dockerfile -t sctv-api .
#
# Run:
#   docker run -p 3000:3000 -e DATABASE_URL=postgres://... sctv-api
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
RUN cargo build --release --bin sctv-api

# -----------------------------------------------------------------------------
# Stage 2: Runtime
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd --create-home --shell /bin/bash sctv
WORKDIR /home/sctv

# Copy binary and migrations
COPY --from=builder /app/target/release/sctv-api /usr/local/bin/sctv-api
COPY migrations ./migrations

# Set ownership
RUN chown -R sctv:sctv /home/sctv

USER sctv

# Environment defaults
ENV SCTV_HOST=0.0.0.0
ENV SCTV_PORT=3000
ENV SCTV_LOG_FORMAT=json
ENV SCTV_ENABLE_CORS=true
ENV RUST_LOG=info,sctv_api=info,tower_http=info

EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -sf http://localhost:3000/health || exit 1

CMD ["sctv-api"]
