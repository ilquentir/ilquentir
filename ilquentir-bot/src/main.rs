use std::{env, time::Duration};

use color_eyre::Result;
use sqlx::migrate::Migrator;
use teloxide::{
    dispatching::{HandlerExt, UpdateFilterExt},
    prelude::Dispatcher,
    requests::Requester,
    types::{Update, UpdateKind},
    utils::command::BotCommands,
    Bot,
};
use tokio::time::interval;
use tracing::error;

mod bot;
mod models;
mod poll;

use bot::{
    commands::Command,
    handlers::{handle_command, handle_poll_update, handle_scheduled},
};

pub static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL should be set");

    let pool = sqlx::PgPool::connect(&db_url).await?;

    // migrate on start
    MIGRATOR.run(&pool).await?;
    // Note that the compiler won't pick up new migrations if no Rust source files have changed.
    // You can create a Cargo build script to work around this with `sqlx migrate build-script`.

    let bot = Bot::from_env();
    bot.set_my_commands(bot::commands::Command::bot_commands())
        .await?;

    let handler = dptree::entry()
        .branch(
            Update::filter_message().branch(
                dptree::entry()
                    .filter_command::<Command>()
                    .endpoint(handle_command),
            ),
        )
        .branch(
            dptree::entry()
                .filter(|update: Update| matches!(update.kind, UpdateKind::Poll(_)))
                .endpoint(handle_poll_update),
        );

    let mut dispatcher = Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![pool.clone()])
        .enable_ctrlc_handler()
        .build();
    let shutdown_token = dispatcher.shutdown_token();
    let scheduler = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60));

        loop {
            interval.tick().await;
            let schedule_result = handle_scheduled(bot.clone(), pool.clone()).await;

            if let Err(e) = schedule_result {
                error!(schedule_error = %e, "got an error while scheduling");
                shutdown_token.shutdown()?.await;
                break;
            };
        }

        #[allow(unreachable_code)]
        Result::<()>::Ok(())
    });

    tokio::join!(dispatcher.dispatch(), scheduler).1??;

    Ok(())
}
