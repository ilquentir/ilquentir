FROM rust:1.65 AS builder
RUN rustup toolchain install nightly-2022-11-17
RUN apt-get update && apt-get install protobuf-compiler --assume-yes

COPY . .
RUN cargo +nightly-2022-11-17 build -p ilquentir-bot --release -Z sparse-registry

FROM debian:buster-slim
COPY --from=builder ./target/release/ilquentir-bot /ilquentir-bot
RUN apt-get update \
    && apt-get install --assume-yes ca-certificates \
    && update-ca-certificates \
    && apt-cache clean

CMD ["/ilquentir-bot"]
