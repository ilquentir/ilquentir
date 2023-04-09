FROM lukemathwalker/cargo-chef:latest-rust-1.68 AS chef

## Prepare
FROM chef AS planner
WORKDIR /app
COPY . .
# FIXME: currently presumes aarch64 architecture only
RUN wget https://github.com/protocolbuffers/protobuf/releases/download/v22.1/protoc-22.1-linux-aarch_64.zip \
    && unzip protoc-22.1-linux-aarch_64.zip -d protoc_dir \
    && mv protoc_dir/bin/protoc protoc \
    && rm -rf protoc_dir protoc-22.1-linux-aarch_64.zip

RUN cargo chef prepare --recipe-path recipe.json

## Build
FROM chef AS builder
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
COPY --from=planner /app/protoc /usr/local/bin/protoc
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build -p ilquentir-bot --release

## Deploy
FROM python:slim

# locale
ENV LC_ALL="ru_RU.UTF-8"
ENV LC_CTYPE="ru_RU.UTF-8"
RUN apt-get update && apt-get install locales --assume-yes && locale-gen \
    && apt-get purge -y --auto-remove -o APT::AutoRemove::RecommendsImportant=false \
    && rm -rf /var/lib/apt/lists/*

# python deps
COPY ilquentir-python-graph/python/requirements.txt /requirements.txt
RUN pip install -r requirements.txt && pip cache purge

COPY ilquentir-python-graph/python/main.py /plotly_graph.py

# rust binary
COPY --from=builder ./target/release/ilquentir-bot /ilquentir-bot

CMD ["/ilquentir-bot"]
