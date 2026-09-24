# Production Runtime for Precompiled ARM64 Binary
#
# Harici varlıklar (ONNX yön modeli + pdfium) derleme (CI) aşamasında indirilip
# sha256 ile doğrulanır; çalışma zamanı imajına yalnızca gerekli dosyalar kopyalanır.
# Tedarik zinciri güvenliği: sabitlenmiş sürüm + sha256; doğrulama başarısızsa
# build kırılır (sessizce bozuk varlıkla devam etmez).

FROM debian:trixie-slim AS assets
RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 1) Sayfa yönü ONNX modeli (PP-LCNet_x1_0_doc_ori, 224x224, 4-sınıf 0/90/180/270)
ARG ORIENTATION_MODEL_URL="https://huggingface.co/PaddlePaddle/PP-LCNet_x1_0_doc_ori_onnx/resolve/main/inference.onnx"
ARG ORIENTATION_MODEL_SHA256="af9a0a4f317ff0709ce752067807f819cb15d883f8ecad89f28df1c6ee2d9c92"
RUN mkdir -p /models \
    && curl -fsSL "$ORIENTATION_MODEL_URL" -o /models/doc_orientation.onnx \
    && echo "$ORIENTATION_MODEL_SHA256  /models/doc_orientation.onnx" | sha256sum -c -

# 2) PDF rasterizasyonu için libpdfium.so (aarch64, sabitlenmiş chromium/8066 sürümü)
ARG PDFIUM_URL="https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/8066/pdfium-linux-arm64.tgz"
ARG PDFIUM_SHA256="0e6f90dccbc6b81fd5d7106abaf164c4222178f024c204d00d526b60fd2ad535"
RUN curl -fsSL "$PDFIUM_URL" -o /tmp/pdfium.tgz \
    && echo "$PDFIUM_SHA256  /tmp/pdfium.tgz" | sha256sum -c - \
    && tar -xzf /tmp/pdfium.tgz -C /tmp \
    && cp /tmp/lib/libpdfium.so /libpdfium.so

FROM debian:trixie-slim

# libgomp1: pdfium/onnxruntime OpenMP bağımlılığı.
RUN apt-get update \
    && apt-get install -y ca-certificates libssl3 libstdc++6 libgomp1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=assets /models /app/models
# pdfium çalışma zamanında dinamik yüklenir (Pdfium::bind_to_system_library).
COPY --from=assets /libpdfium.so /usr/lib/libpdfium.so
RUN ldconfig

COPY target/aarch64-unknown-linux-gnu/release/worker /app/kepce-worker
COPY config /app/config

USER nobody:nogroup
CMD ["/app/kepce-worker"]