# Production Runtime for Precompiled ARM64 Binary
FROM debian:trixie-slim
WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY target/aarch64-unknown-linux-gnu/release/moderator /app/kepce-moderator

ENV PORT=8002
EXPOSE 8002

CMD ["/app/kepce-moderator"]
