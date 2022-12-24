use color_eyre::{eyre::eyre, Result};
use teloxide::{
    payloads::{EditMessageReplyMarkupSetters, SendMessageSetters},
    requests::Requester,
    types::{CallbackQuery, ChatId},
};

use ilquentir_messages::{md, md_message};
use ilquentir_models::{PgTransaction, Poll, PollCustomOptions, PollKind};
use tracing::warn;

use crate::bot::Bot;

use super::{keyboard::make_keyboard, options};

#[tracing::instrument(skip(bot, txn), err)]
pub async fn handle_settings_command(
    bot: &Bot,
    txn: &mut PgTransaction<'_>,
    chat_id: ChatId,
) -> Result<()> {
    let keyboard = make_keyboard(txn, chat_id.0).await?;

    bot.send_message(chat_id, md_message!("daily_events/settings.md"))
        .reply_markup(keyboard)
        .await?;

    Ok(())
}

#[tracing::instrument(skip(bot, txn), err)]
pub async fn handle_callback(
    bot: &Bot,
    txn: &mut PgTransaction<'_>,
    callback: &CallbackQuery,
    payload: &str,
) -> Result<()> {
    let user_tg_id = callback.from.id.0 as i64;
    let message = callback
        .message
        .as_ref()
        .ok_or_else(|| eyre!("payload with no message"))?;
    const POLL_KIND: PollKind = PollKind::DailyEvents;

    if options::DONE_BUTTON.matches(payload) {
        bot.edit_message_text(
            user_tg_id.to_string(),
            message.id,
            md_message!("daily_events/settings_done.md"),
        )
        .await?;

        bot.answer_callback_query(&callback.id).await?;

        let pending_daily_events =
            Poll::get_pending_for_user(&mut *txn, user_tg_id, POLL_KIND).await?;

        if pending_daily_events.is_empty() {
            todo!("schedule polls")
        }

        return Ok(());
    }

    if options::ALL_BUTTON.matches(payload) {
        let current = PollCustomOptions::get_for_user(&mut *txn, user_tg_id, POLL_KIND).await?;

        for option in options::ALL_OPTIONS.values() {
            // TODO: bulk insert

            if current.options.contains(option.value()) {
                continue;
            }
            PollCustomOptions::toggle_option(&mut *txn, user_tg_id, POLL_KIND, option.value())
                .await?;
        }
    } else if options::NONE_BUTTON.matches(payload) {
        PollCustomOptions::clear_user_options(txn, user_tg_id, POLL_KIND).await?;
    } else if let Some(option) = options::ALL_OPTIONS.values().find(|o| o.matches(payload)) {
        PollCustomOptions::toggle_option(&mut *txn, user_tg_id, POLL_KIND, option.value()).await?;
    } else {
        warn!("got unknown payload");
    }

    let keyboard = make_keyboard(txn, user_tg_id).await?;

    bot.edit_message_reply_markup(user_tg_id.to_string(), message.id)
        .reply_markup(keyboard)
        .await?;
    bot.answer_callback_query(&callback.id).await?;

    Ok(())
}


#[tracing::instrument(skip(bot, txn), err)]
pub async fn handle_promo_callback(
    bot: &Bot,
    txn: &mut PgTransaction<'_>,
    callback: &CallbackQuery,
    payload: &str,
) -> Result<()> {
    let user_tg_id = callback.from.id.0 as i64;
    let message = callback
        .message
        .as_ref()
        .ok_or_else(|| eyre!("payload with no message"))?;
    const POLL_KIND: PollKind = PollKind::DailyEvents;

    if options::PROMO_NO_BUTTON.matches(payload) {
        bot.edit_message_text(
            user_tg_id.to_string(),
            message.id,
            md_message!("daily_events/promo_no.md"),
        )
        .await?;

    } else if options::PROMO_YES_BUTTON.matches(payload) {
        bot.edit_message_text(
            user_tg_id.to_string(),
            message.id,
            md!("Отлично! Давай попробуем :)"),
        )
        .await?;
        let keyboard = make_keyboard(txn, user_tg_id).await?;

        bot.send_message(user_tg_id.to_string(), md_message!("daily_events/settings.md"))
            .reply_markup(keyboard)
            .await?;
    } else {
        warn!("got unknown payload");
    }

    bot.answer_callback_query(&callback.id).await?;

    Ok(())
}

