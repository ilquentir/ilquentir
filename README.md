# Ilquentir

**TODO** describe repo

## Deployment

1. Populate following env variables:
  * `DATABASE_URL` – config URL for Postgres DB to be used
  * `TELOXIDE_TOKEN` – Telegram bot token, you can get one from [Bot Father](https://t.me/BotFather)
  * `EXPORTER_URL` – OTLP exporter URL, tested with https://api.honeycomb.io:443
  * `HONEYCOMB_KEY` – Honeycomb.io API KEY
  * `ENVIRONMENT` – application environment
  * `RUST_LOG` – desired log level

## Development

1. Ensure [Rust](https://rust-lang.org) is installed
2. Create `.env` file with variables described in [Deployment](#deployment) section
3. Run, for example via `cargo run`