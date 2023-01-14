use color_eyre::Result;
use teloxide::{requests::Requester, types::ChatId};

use ilquentir_messages::md_message;

use crate::bot::Bot;

#[tracing::instrument(skip_all, fields(chat_id=chat_id.0), err)]
pub async fn handle_help(bot: &Bot, chat_id: ChatId) -> Result<()> {
    bot.send_message(chat_id, md_message!("help/main.md"))
        .await?;

    Ok(())
}
