FROM rust:1-slim AS builder

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl xz-utils && rm -rf /var/lib/apt/lists/*

# Install prebuilt typst binary
RUN ARCH=$(uname -m) && \
    if [ "$ARCH" = "x86_64" ]; then TYPST_ARCH="x86_64-unknown-linux-musl"; \
    elif [ "$ARCH" = "aarch64" ]; then TYPST_ARCH="aarch64-unknown-linux-musl"; \
    else TYPST_ARCH="x86_64-unknown-linux-musl"; fi && \
    curl -L "https://github.com/typst/typst/releases/download/v0.15.1/typst-${TYPST_ARCH}.tar.xz" | tar -xJ -C /tmp && \
    mv /tmp/typst-*/typst /usr/local/bin/typst && \
    chmod +x /usr/local/bin/typst && \
    rm -rf /tmp/typst-*

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/heatpump-dashboard /app/heatpump-dashboard
COPY assets /app/assets

EXPOSE 8050
ENV RUST_LOG=info

CMD ["/app/heatpump-dashboard"]
