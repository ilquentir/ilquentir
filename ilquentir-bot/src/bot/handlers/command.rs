use color_eyre::Result;
use sqlx::PgPool;
use teloxide::types::Message;

use crate::bot::{daily_events, setup_schedule, Bot, Command};

mod get_stat;
use get_stat::handle_get_stat;

mod start;
use start::handle_start;

mod stop;
use stop::handle_stop;

#[tracing::instrument(skip(bot, pool), err)]
pub async fn handle_command(bot: Bot, pool: PgPool, msg: Message, command: Command) -> Result<()> {
    let mut txn = pool.begin().await?;

    match command {
        Command::Start => handle_start(&bot, &mut txn, msg.chat.id).await?,
        Command::GetStat => handle_get_stat(&bot, &mut txn, msg.chat.id).await?,
        Command::DailyEventsSettings => {
            daily_events::handle_settings_command(&bot, &mut txn, msg.chat.id).await?
        }
        Command::SetupSchedule => {
            setup_schedule::handle_setup_schedule_command(&bot, msg.chat.id).await?
        }

        Command::Stop => handle_stop(&bot, &mut txn, msg.chat.id).await?,
    }
    txn.commit().await?;

    Ok(())
}
