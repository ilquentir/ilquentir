use std::env;

use color_eyre::Result;
use ilquentir_giphy::GiphyApi;
use sqlx::migrate::Migrator;
use tracing::info;

mod bot;
mod models;
mod poll;
mod scheduler;
mod tracing_setup;

use crate::{bot::create_bot_dispatcher, scheduler::Scheduler};

pub static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_setup::setup()?;

    let db_url = env::var("DATABASE_URL")?;

    let pool = sqlx::PgPool::connect(&db_url).await?;
    MIGRATOR.run(&pool).await?;

    let giphy = GiphyApi::new(&env::var("GIPHY_KEY")?)?;

    let (mut dispatcher, bot) = create_bot_dispatcher(pool.clone(), giphy).await?;
    let scheduler = Scheduler::new(&dispatcher);
    let scheduler_shutdown_token = scheduler.shutdown_token();

    tokio::spawn(async move { scheduler.start(&pool, &bot).await });
    dispatcher.dispatch().await;
    info!("dispatcher stopped working, shutting down scheduler");
    scheduler_shutdown_token.shutdown();

    tracing_setup::teardown();

    Ok(())
}
