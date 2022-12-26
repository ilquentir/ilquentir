use color_eyre::Result;
use teloxide::{payloads::SendMessageSetters, requests::Requester, types::ChatId};

use ilquentir_messages::md_message;
use ilquentir_models::PgTransaction;

use super::super::keyboard::user_daily_options;

use crate::bot::Bot;

#[tracing::instrument(skip(bot, txn), err)]
pub async fn handle_settings_command(
    bot: &Bot,
    txn: &mut PgTransaction<'_>,
    chat_id: ChatId,
) -> Result<()> {
    let keyboard = user_daily_options(txn, chat_id.0).await?;

    bot.send_message(chat_id, md_message!("daily_events/settings.md"))
        .reply_markup(keyboard)
        .await?;

    Ok(())
}
