use std::time::Duration;

use color_eyre::Result;
use teloxide::{payloads::SendChatAction, requests::JsonRequest, Bot};

#[tracing::instrument(skip(bot), err)]
pub async fn set_typing(bot: &Bot, chat_id: i64, sleep_after: Option<Duration>) -> Result<()> {
    let typing_payload =
        SendChatAction::new(chat_id.to_string(), teloxide::types::ChatAction::Typing);
    JsonRequest::new(bot.clone(), typing_payload).await?;

    if let Some(sleep_after) = sleep_after {
        tokio::time::sleep(sleep_after).await;
    }

    Ok(())
}
