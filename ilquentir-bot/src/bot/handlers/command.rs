use color_eyre::Result;
use sqlx::PgPool;
use teloxide::{requests::Requester, types::Message};
use tracing::info;

use ilquentir_messages::md;
use ilquentir_models::User;

use crate::bot::{daily_events, Bot, Command};

mod get_stat;
use get_stat::handle_get_stat;

mod start;
use start::handle_start;

#[tracing::instrument(skip(bot, pool), err)]
pub async fn handle_command(bot: Bot, pool: PgPool, msg: Message, command: Command) -> Result<()> {
    let mut txn = pool.begin().await?;

    match command {
        Command::Start => handle_start(&bot, &mut txn, msg.chat.id).await?,
        Command::GetStat => handle_get_stat(&bot, &mut txn, msg.chat.id).await?,
        Command::DailyEventsSettings => {
            daily_events::handle_settings_command(&bot, &mut txn, msg.chat.id).await?
        }

        Command::Stop => {
            info!(chat_id = %msg.chat.id, "processing Stop command");

            let user = User::deactivate(&mut txn, msg.chat.id.0).await?;

            info!(chat_id = %msg.chat.id, user_id = user.tg_id, "disabled user");

            bot.send_message(msg.chat.id, md!("Деактивировали! Надеюсь, ещё увидимся :)"))
                .await?;
        }
    }
    txn.commit().await?;

    Ok(())
}
