# Build
FROM rust:1.68 AS builder

ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
RUN apt-get update && apt-get install protobuf-compiler --assume-yes

COPY . .
RUN cargo build -p ilquentir-bot --release

# Deploy
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
