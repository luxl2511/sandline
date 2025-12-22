# Build stage
FROM rust:slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build for release (skip SQLx compile-time verification due to IPv6)
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary from builder
COPY --from=builder /app/target/release/dakar-planner-backend /app/dakar-planner-backend

# Set environment variables
ENV RUST_LOG=info,dakar_planner_backend=debug
ENV HOST=0.0.0.0
ENV PORT=8080

EXPOSE 8080

CMD ["/app/dakar-planner-backend"]
