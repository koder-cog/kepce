# Binary kaynaktan derlenir: imaja derlenmiş artifact kopyalanmadığı için içerik
# her zaman kaynakla tutarlı kalır.
FROM rust:trixie AS builder
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev g++ \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
RUN cargo build --release -p api

FROM debian:trixie-slim

# Çalışan imajın hangi commit'e ait olduğunu görünür kılar; deploy tazelik kapısı
# bu değeri okur.
ARG GIT_SHA=unknown
ENV KEPCE_GIT_SHA=$GIT_SHA

RUN apt-get update && apt-get install -y ca-certificates libssl3 libstdc++6 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/api /app/kepce-api
COPY db /app/db
COPY config /app/config

RUN mkdir -p /app/static/avatars /app/uploads && chown -R nobody:nogroup /app/static /app/uploads

EXPOSE 8000

USER nobody:nogroup
CMD ["/app/kepce-api"]
