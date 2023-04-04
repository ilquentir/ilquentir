# Build
FROM rust:1.68 AS builder

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN apt-get update && apt-get install protobuf-compiler --assume-yes

COPY . .
RUN cargo build -p ilquentir-bot --release

# Deploy
FROM debian:bookworm-slim

# locale
ENV LC_ALL="ru_RU.UTF-8"
ENV LC_CTYPE="ru_RU.UTF-8"
RUN apt-get update && apt-get install locales --assume-yes && locale-gen

# python deps
COPY ilquentir-python-graph/python/requirements.txt /requirements.txt
RUN apt-get update \
    && apt-get install python3.11 curl --assume-yes \
    && curl https://bootstrap.pypa.io/get-pip.py | python3.11 \
    && pip install -r requirements.txt

COPY ilquentir-python-graph/python/main.py /plotly_graph.py

# rust binary
COPY --from=builder ./target/release/ilquentir-bot /ilquentir-bot
RUN apt-get update \
    && apt-get install --assume-yes ca-certificates \
    && update-ca-certificates

CMD ["/ilquentir-bot"]
