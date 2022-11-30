# Ilquentir

Telegram bot, fetch

## Deployment

1. Populate following env variables:
  * `DATABASE_URL` – config URL for Postgres DB to be used
  * `RUST_LOG` – desired log level

## Development

1. Ensure [Rust](https://rust-lang.org) is installed
2. Create `.env` file with variables described in [Deployment](#deployment) section
3. Run, for example via `cargo run`