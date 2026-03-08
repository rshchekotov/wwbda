# syntax=docker/dockerfile:1

## BUILD STAGE
FROM rust:1.93.1-slim as builder

RUN apt-get update && \
  apt-get install -y --no-install-recommends pkg-config libssl-dev libsqlite3-dev && \
  rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
COPY bot/ ./bot/
COPY libshogi/ ./libshogi/

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release --bin bot

## RUNTIME STAGE
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/* && \
    mkdir -p /app/db /app/logs && \
    chown 1000:1000 /app/db && \
    chown 1000:1000 /app/logs

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/bot .
COPY CHANGELOG.md .
COPY entrypoint.sh .

RUN chmod +x entrypoint.sh

USER 1000:1000
CMD ["./entrypoint.sh"]

