# Production Runtime for Precompiled ARM64 Binary
FROM debian:trixie-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY target/aarch64-unknown-linux-gnu/release/api /app/kepce-api
COPY db /app/db
COPY config /app/config

RUN mkdir -p /app/static/avatars /app/uploads && chown -R nobody:nogroup /app/static /app/uploads

EXPOSE 8000

USER nobody:nogroup
CMD ["/app/kepce-api"]
