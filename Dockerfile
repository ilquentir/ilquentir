FROM rust:1.65 AS builder
RUN rustup toolchain install nightly-2022-11-17
RUN apt-get update && apt-get install protobuf-compiler --assume-yes

COPY . .
RUN cargo +nightly-2022-11-17 build -p ilquentir-bot --release -Z sparse-registry

FROM debian:bookworm-slim
ENV LC_ALL="ru_RU.UTF-8"
ENV LC_CTYPE="ru_RU.UTF-8"
RUN apt-get update && apt-get install locales --assume-yes && locale-gen

COPY ilquentir-python-graph/python/requirements.txt /requirements.txt
RUN apt-get update \
    && apt-get install python3.11 curl --assume-yes \
    && curl https://bootstrap.pypa.io/get-pip.py | python3.11 \
    && pip install -r requirements.txt

COPY ilquentir-python-graph/python/main.py /plotly_graph.py

COPY --from=builder ./target/release/ilquentir-bot /ilquentir-bot
RUN apt-get update \
    && apt-get install --assume-yes ca-certificates \
    && update-ca-certificates

CMD ["/ilquentir-bot"]
