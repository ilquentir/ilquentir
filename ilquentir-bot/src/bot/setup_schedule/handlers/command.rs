use color_eyre::Result;
use teloxide::{payloads::SendMessageSetters, requests::Requester, types::ChatId};
// use tracing::info;

use ilquentir_messages::md;

use crate::bot::Bot;

use super::super::keyboard::create_timepicker_keyboard;

#[tracing::instrument(skip(bot), fields(chat_id=chat_id.0), err)]
pub async fn handle_setup_schedule_command(bot: &Bot, chat_id: ChatId) -> Result<()> {
    bot.send_message(
        chat_id,
        md!("Выбери, в какое время (по твоему часовому поясу) присылать опросы за день:"),
    )
    .reply_markup(create_timepicker_keyboard())
    .await?;

    Ok(())
}
