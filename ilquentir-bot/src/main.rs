use std::{env, time::Duration, sync::{atomic::AtomicBool, Arc}};

use color_eyre::Result;
use sqlx::migrate::Migrator;
use tokio::time::interval;
use tracing::{info, error};

mod bot;
mod models;
mod poll;

use crate::bot::{handlers::handle_scheduled, create_bot_dispatcher};

pub static MIGRATOR: Migrator = sqlx::migrate!();

#[tracing::instrument]
#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL should be set");

    let pool = sqlx::PgPool::connect(&db_url).await?;
    MIGRATOR.run(&pool).await?;

    let (mut dispatcher, bot) = create_bot_dispatcher(pool.clone()).await?;
    let dispatcher_shutdown_token = dispatcher.shutdown_token();
    let scheduler_shutdown_token = Arc::new(AtomicBool::new(false));
    let scheduler_token_clone = scheduler_shutdown_token.clone();

    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60));

        loop {
            interval.tick().await;
            let schedule_result = handle_scheduled(&bot, &pool).await;

            if let Err(e) = schedule_result {
                error!(schedule_error = %e, "got an error while scheduling");
                dispatcher_shutdown_token.shutdown()?.await;

                return Err(e);
            };

            if scheduler_token_clone.load(std::sync::atomic::Ordering::Acquire) {
                info!("scheduler shut down");

                break;
            }
        }
        Ok(())
    });
    dispatcher.dispatch().await;
    info!("dispatcher stopped working, shutting down scheduler");
    scheduler_shutdown_token.store(true, std::sync::atomic::Ordering::Release);

    Ok(())
}
