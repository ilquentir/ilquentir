use color_eyre::Result;
use sqlx::PgPool;
use teloxide::types::Message;

use ilquentir_config::Config;

use crate::bot::{daily_events, get_stats, setup_schedule, Bot, Command};

mod help;
use help::handle_help;

mod start;
use start::handle_start;

mod stop;
use stop::handle_stop;

#[tracing::instrument(skip(bot, pool, config), err)]
pub async fn handle_command(
    bot: Bot,
    pool: PgPool,
    config: Config,
    msg: Message,
    command: Command,
) -> Result<()> {
    let mut txn = pool.begin().await?;
    let chat_id = msg.chat.id;

    match command {
        Command::Start => handle_start(&bot, &mut txn, chat_id).await?,
        Command::DailyEventsSettings => {
            daily_events::handle_settings_command(&bot, &mut txn, chat_id).await?
        }
        Command::SetupSchedule => {
            setup_schedule::handle_setup_schedule_command(&bot, chat_id).await?
        }

        Command::GetStat => {
            get_stats::handle_get_stats_command(&bot, &config, &mut txn, chat_id).await?
        }

        Command::Help => handle_help(&bot, chat_id).await?,

        Command::Stop => handle_stop(&bot, &mut txn, chat_id).await?,
    }
    txn.commit().await?;

    Ok(())
}
