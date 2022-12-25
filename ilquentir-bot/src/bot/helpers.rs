use std::time::Duration;

use color_eyre::Result;
use ilquentir_models::{PgTransaction, Poll};
use teloxide::{
    payloads::SendPollSetters,
    requests::Requester,
    types::{Message, Recipient},
};
use tracing::info;

use super::Bot;

/// Telegram only allows <= 10 options per poll
const TELEGRAM_POLL_OPTIONS_LIMIT: usize = 10;

#[tracing::instrument(skip(bot), err)]
pub async fn set_typing(
    bot: &Bot,
    chat_id: impl Into<Recipient> + std::fmt::Debug,
    sleep_after: Option<Duration>,
) -> Result<()> {
    bot.send_chat_action(chat_id, teloxide::types::ChatAction::Typing)
        .await?;

    if let Some(sleep_after) = sleep_after {
        tokio::time::sleep(sleep_after).await;
    }

    Ok(())
}

#[tracing::instrument(skip(bot), err)]
pub async fn send_poll<'t>(
    bot: &Bot,
    txn: &mut PgTransaction<'t>,
    poll: &Poll,
) -> Result<Vec<Message>> {
    info!(poll_id = poll.id, "sending poll");

    let options = poll.kind.options(txn, poll.chat_tg_id).await?;

    let mut chunk_size = TELEGRAM_POLL_OPTIONS_LIMIT;
    while options.len() % chunk_size == 1 && chunk_size > 1 {
        chunk_size -= 1;
    }

    let mut sent_messages = vec![];

    for options_chunk in options.chunks(chunk_size) {
        sent_messages.push(
            bot.send_poll(
                poll.chat_tg_id.to_string(),
                poll.kind.question(),
                options_chunk.iter().cloned(),
            )
            .allows_multiple_answers(poll.kind.allows_multiple_answers())
            .await?
        );
    }

    info!(poll_id = poll.id, "poll sent");

    Ok(sent_messages)
}
