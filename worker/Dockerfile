# Multi-stage build for Rust Worker
FROM rust:1.80-slim AS builder

WORKDIR /usr/src/kepce

# Install pkg-config and libssl-dev for building reqwest/openssl
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy workspace cargo metadata
COPY Cargo.toml Cargo.lock ./
COPY shared/Cargo.toml ./shared/
COPY api/Cargo.toml ./api/
COPY worker/Cargo.toml ./worker/

# Build skeleton project to cache build dependencies
RUN mkdir -p shared/src api/src worker/src && \
    echo "fn main() {}" > api/src/main.rs && \
    echo "fn main() {}" > worker/src/main.rs && \
    touch shared/src/lib.rs

RUN cargo build --release -p worker

# Copy real source files
COPY shared/src ./shared/src
COPY worker/src ./worker/src

# Recompile the release binary
RUN touch worker/src/main.rs && cargo build --release -p worker

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/kepce/target/release/worker /app/kepce-worker

USER nobody:nogroup
CMD ["/app/kepce-worker"]
