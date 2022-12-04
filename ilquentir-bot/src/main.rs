use std::env;

use color_eyre::Result;
use sqlx::migrate::Migrator;
use tracing::info;

mod bot;
mod models;
mod poll;
mod scheduler;

use crate::{bot::create_bot_dispatcher, scheduler::Scheduler};

pub static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
#[tracing::instrument]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL should be set");

    let pool = sqlx::PgPool::connect(&db_url).await?;
    MIGRATOR.run(&pool).await?;

    let (mut dispatcher, bot) = create_bot_dispatcher(pool.clone()).await?;
    let scheduler = Scheduler::new(&dispatcher);
    let scheduler_shutdown_token = scheduler.shutdown_token();

    tokio::spawn(async move { scheduler.start(&pool, &bot).await });
    dispatcher.dispatch().await;
    info!("dispatcher stopped working, shutting down scheduler");
    scheduler_shutdown_token.shutdown();

    Ok(())
}
