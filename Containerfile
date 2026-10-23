FROM rust:latest as builder

WORKDIR /usr/src/crustypump


COPY Cargo.toml Cargo.lock ./



RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true


COPY . .


RUN cargo build --release


FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app


COPY --from=builder /usr/src/crustypump/target/release/crustypump /usr/local/bin/crustypump
RUN chmod +x /usr/local/bin/crustypump


RUN useradd -m appuser || true
RUN chown -R appuser:appuser /usr/local/bin/crustypump /app
USER appuser
ENV RUST_LOG=info
ENTRYPOINT ["/usr/local/bin/crustypump"]