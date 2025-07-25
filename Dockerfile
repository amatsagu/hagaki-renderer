# Use slim Rust base on amd64 & arm64 compatible Debian
FROM rust:1.88-slim AS builder

WORKDIR /app
COPY . /app

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/hagaki /usr/local/bin/hagaki
ENV RUST_LOG=info

CMD ["hagaki"]
