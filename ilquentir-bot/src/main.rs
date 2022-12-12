use color_eyre::Result;
use sqlx::migrate::Migrator;
use tracing::info;

use ilquentir_config::Config;
use ilquentir_giphy::GiphyApi;

mod bot;
mod scheduler;
mod tracing_setup;

use crate::{bot::create_bot_and_dispatcher, scheduler::Scheduler};

pub static MIGRATOR: Migrator = sqlx::migrate!("../ilquentir-models/migrations");

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;
    tracing_setup::setup(&config)?;

    let pool = sqlx::PgPool::connect(&config.database_url).await?;
    MIGRATOR.run(&pool).await?;

    let giphy = GiphyApi::new(&config.giphy_key)?;

    let (mut dispatcher, bot) = create_bot_and_dispatcher(pool.clone(), giphy, &config).await?;
    let scheduler = Scheduler::new(&dispatcher);
    let scheduler_shutdown_token = scheduler.shutdown_token();

    tokio::spawn(async move { scheduler.start(&pool, &bot, &config).await });
    dispatcher.dispatch().await;
    info!("dispatcher stopped working, shutting down scheduler");
    scheduler_shutdown_token.shutdown();

    tracing_setup::teardown();

    Ok(())
}
