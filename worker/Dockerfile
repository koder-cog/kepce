# Production Runtime for Precompiled ARM64 Binary
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 libstdc++6 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY target/aarch64-unknown-linux-gnu/release/worker /app/kepce-worker
COPY config /app/config

USER nobody:nogroup
CMD ["/app/kepce-worker"]
