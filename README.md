# Ilquentir

Ilquentir is a Telegram bot, which helps to track your mood and generate graphs from it.

Main features:
* Track your mood daily
* Track events, which may affect your mood
* Generate graphs from your mood data

## Development

1. Ensure [Rust](https://rust-lang.org) is installed
2. Ensure you have `portoc` available in your `$PATH`, you can get it from
  [here](https://github.com/protocolbuffers/protobuf#protocol-compiler-installation),
  or from your package manager, e.g. `brew install protobuf` on macOS
3. Obtain requred tokens:
   1. Telegram bot token, you can get one from [Bot Father](https://t.me/BotFather)
   2. [Honeycomb.io](https://honeycomb.io) API KEY
   3. S3-compatible storage credentials, I use [DigitalOcean Spaces](https://www.digitalocean.com/products/spaces/) for this
4. Create `.env` file with variables described in [Deployment](#deployment) section
5. Run, for example via `cargo run`

## Deployment

1. Populate following env variables:
  * `DATABASE_URL` – config URL for Postgres DB to be used
  * `TELOXIDE_TOKEN` – Telegram bot token, you can get one from [Bot Father](https://t.me/BotFather)
  * `EXPORTER_URL` – OTLP exporter URL, tested with https://api.honeycomb.io:443
  * `HONEYCOMB_KEY` – Honeycomb.io API KEY
  * `ENVIRONMENT` – application environment
  * `RUST_LOG` – desired log level
  * `SCHEDULER_INTERVAL` – how often scheduler should run, e.g. `1s`, `5m`, `1h`
  * Following settings are used for [ilquentir-python-graph](./ilquentir-python-graph/) interop, which handles the generation of everyday mood graphs:
    * `WIDE_HOW_WAS_YOUR_DAY_PATH` – path, where wide aggregated data should be stored
    * `WIDE_HOW_WAS_YOUR_DAY_MAX_AGE` – how long wide aggregated data should be stored before refreshing, e.g. `1s`, `5m`, `1h`
    * `PLOTLY_PYTHON_CODE_FILE` – path to python script, which will be executed to generate plotly graphs
  * And, as usual, lots of settings for some S3-compatible storage, I use [DigitalOcean Spaces](https://www.digitalocean.com/products/spaces/) for this:
    * `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY` – usual AWS credentials
    * `AWS_DEFAULT_REGION` – S3 region, e.g. `fra1`
    * `AWS_S3_ENDPOINT` – S3 endpoint, e.g. `https://fra1.digitaloceanspaces.com`
    * `AWS_S3_BUCKET` – S3 bucket name
    * `AWS_S3_STATIC_URL` – S3 bucket URL, e.g. `https://ilquentir.fra1.digitaloceanspaces.com`
