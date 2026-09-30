# Varlıklar derleme sırasında indirilip sha256 ile doğrulanır: sabit sürüm +
# doğrulama, bozuk/eksik varlıkla sessizce devam etmeyi engeller.
FROM debian:trixie-slim AS assets
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ARG ORIENTATION_MODEL_URL="https://huggingface.co/PaddlePaddle/PP-LCNet_x1_0_doc_ori_onnx/resolve/main/inference.onnx"
ARG ORIENTATION_MODEL_SHA256="af9a0a4f317ff0709ce752067807f819cb15d883f8ecad89f28df1c6ee2d9c92"
RUN mkdir -p /models \
    && curl -fsSL "$ORIENTATION_MODEL_URL" -o /models/doc_orientation.onnx \
    && echo "$ORIENTATION_MODEL_SHA256  /models/doc_orientation.onnx" | sha256sum -c -

ARG PDFIUM_URL="https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/8066/pdfium-linux-arm64.tgz"
ARG PDFIUM_SHA256="0e6f90dccbc6b81fd5d7106abaf164c4222178f024c204d00d526b60fd2ad535"
RUN curl -fsSL "$PDFIUM_URL" -o /tmp/pdfium.tgz \
    && echo "$PDFIUM_SHA256  /tmp/pdfium.tgz" | sha256sum -c - \
    && tar -xzf /tmp/pdfium.tgz -C /tmp \
    && cp /tmp/lib/libpdfium.so /libpdfium.so

# Binary kaynaktan derlenir: imaja derlenmiş artifact kopyalanmadığı için içerik
# her zaman kaynakla tutarlı kalır.
FROM rust:trixie AS builder
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev g++ \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
RUN cargo build --release -p worker

FROM debian:trixie-slim

# Çalışan imajın hangi commit'e ait olduğunu görünür kılar; deploy tazelik kapısı
# bu değeri okur.
ARG GIT_SHA=unknown
ENV KEPCE_GIT_SHA=$GIT_SHA

# libgomp1: pdfium/onnxruntime OpenMP bağımlılığı.
RUN apt-get update \
    && apt-get install -y ca-certificates libssl3 libstdc++6 libgomp1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=assets /models /app/models
# pdfium çalışma zamanında dinamik yüklenir (Pdfium::bind_to_system_library).
COPY --from=assets /libpdfium.so /usr/lib/libpdfium.so
RUN ldconfig

COPY --from=builder /app/target/release/worker /app/kepce-worker
COPY config /app/config

# `nobody` /app kök dizinine yazamaz; iki önbellek dizini volume ile kalıcı tutulur.
RUN mkdir -p /app/.fastembed_cache /app/cache \
    && chown nobody:nogroup /app/.fastembed_cache /app/cache

USER nobody:nogroup
CMD ["/app/kepce-worker"]
