FROM rust:1-slim AS builder

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/heatpump-dashboard /app/heatpump-dashboard
COPY assets /app/assets

EXPOSE 8050
ENV RUST_LOG=info

CMD ["/app/heatpump-dashboard"]
