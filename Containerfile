# 1. Base image with cargo-chef
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /usr/src/crustypump

# 2. Planner stage to determine dependency recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# 3. Builder stage: build dependencies first (cached), then the real application
FROM chef AS builder
COPY --from=planner /usr/src/crustypump/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin crustypump

# 4. Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/crustypump/target/release/crustypump /usr/local/bin/crustypump
RUN chmod +x /usr/local/bin/crustypump \
    && useradd -m appuser \
    && chown -R appuser:appuser /usr/local/bin/crustypump /app

USER appuser
ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/crustypump"]
