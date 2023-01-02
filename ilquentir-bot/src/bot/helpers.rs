use std::time::Duration;

use color_eyre::{eyre::eyre, Result};
use ilquentir_models::{PgTransaction, Poll};
use teloxide::{
    payloads::SendPollSetters,
    requests::Requester,
    types::{Message, MessageId, Recipient},
};
use tracing::{info, warn};

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

#[tracing::instrument(skip(bot, txn), err)]
pub async fn send_poll(bot: &Bot, txn: &mut PgTransaction<'_>, poll: Poll) -> Result<Vec<Message>> {
    info!(poll_id = poll.id, "sending poll");

    let options = poll.kind.options(&mut *txn, poll.chat_tg_id).await?;

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
            .await?,
        );
    }

    info!(poll_id = poll.id, "poll sent");

    poll.published_to_tg(&mut *txn, &sent_messages).await?;

    Ok(sent_messages)
}

#[tracing::instrument(skip(bot, txn), err)]
pub async fn overdue_poll(bot: &Bot, txn: &mut PgTransaction<'_>, poll: Poll) -> Result<()> {
    if poll.tg_id.is_none() {
        return Err(eyre!("trying to delete unpublished poll"));
    };

    if let Some(message_id) = poll.tg_message_id {
        let response = bot.delete_message(poll.chat_tg_id.to_string(), MessageId(message_id))
            .await;

        if let Err(err) = response {
            warn!(%err, "failed to delete obsolete message");
        }
    } else {
        info!(poll = poll.id, "post with unknown message id is overdue")
    }
    poll.set_overdue(txn).await?;

    Ok(())
}
