use std::time::Duration;

use color_eyre::Result;
use ilquentir_models::{PgTransaction, Poll};
use teloxide::{
    payloads::SendPollSetters,
    requests::Requester,
    types::{ChatId, Message},
};
use tracing::info;

use super::Bot;

#[tracing::instrument(skip(bot), err)]
pub async fn set_typing(bot: &Bot, chat_id: ChatId, sleep_after: Option<Duration>) -> Result<()> {
    bot.send_chat_action(chat_id.to_string(), teloxide::types::ChatAction::Typing)
        .await?;

    if let Some(sleep_after) = sleep_after {
        tokio::time::sleep(sleep_after).await;
    }

    Ok(())
}

#[tracing::instrument(skip(bot), err)]
pub async fn send_poll<'t>(bot: &Bot, txn: &mut PgTransaction<'t>, poll: &Poll) -> Result<Message> {
    info!(poll_id = poll.id, "sending poll");

    let poll_message = bot
        .send_poll(
            poll.chat_tg_id.to_string(),
            poll.kind.question(),
            poll.kind.options(txn, poll.chat_tg_id).await?,
        )
        .allows_multiple_answers(poll.kind.allows_multiple_answers())
        .await?;

    info!(poll_id = poll.id, "poll sent");

    Ok(poll_message)
}
