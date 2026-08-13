# Multi-stage build for Rust API
FROM rust:1-slim AS builder

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

RUN cargo build --release -p api

# Copy real source files
COPY shared/src ./shared/src
COPY api/src ./api/src
COPY api/prompts ./api/prompts
COPY db ./db

# Recompile the release binary
RUN rm -rf target/release/deps/libshared* target/release/deps/shared* target/release/deps/libapi* target/release/deps/api* target/release/.fingerprint/shared* target/release/.fingerprint/api* target/release/api && cargo build --release -p api

# Runtime stage
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/kepce/target/release/api /app/kepce-api
COPY db /app/db

RUN mkdir -p /app/static/avatars /app/uploads && chown -R nobody:nogroup /app/static /app/uploads

EXPOSE 8000

USER nobody:nogroup
CMD ["/app/kepce-api"]
