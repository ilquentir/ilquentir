use color_eyre::{eyre::eyre, Result};
use teloxide::{
    payloads::{EditMessageReplyMarkupSetters, SendMessageSetters},
    requests::Requester,
    types::CallbackQuery,
};
use tracing::warn;

use ilquentir_messages::md_message;
use ilquentir_models::{PgTransaction, Poll, PollCustomOptions, PollKind};

use crate::bot::{helpers::send_poll, Bot};

use super::super::{keyboard::user_daily_options, options};

#[tracing::instrument(skip(bot, txn), err)]
pub async fn handle_callback(
    bot: &Bot,
    txn: &mut PgTransaction<'_>,
    callback: &CallbackQuery,
    payload: &str,
) -> Result<()> {
    const DAILY_EVENTS: PollKind = PollKind::DailyEvents;

    let user_tg_id = callback.from.id.0 as i64;
    let message = callback
        .message
        .as_ref()
        .ok_or_else(|| eyre!("payload with no message"))?;
    let current = PollCustomOptions::get_for_user(&mut *txn, user_tg_id, DAILY_EVENTS).await?;

    if options::DONE_BUTTON.matches(payload) {
        if current.options.is_empty() {
            Poll::disable_pending_for_user(&mut *txn, user_tg_id, DAILY_EVENTS).await?;

            bot.edit_message_text(
                user_tg_id.to_string(),
                message.id,
                md_message!("daily_events/settings_empty.md"),
            )
            .await?;

            return Ok(());
        }

        bot.edit_message_text(
            user_tg_id.to_string(),
            message.id,
            md_message!("daily_events/settings_done.md"),
        )
        .await?;

        bot.answer_callback_query(&callback.id).await?;

        let pending_daily_events_polls =
            Poll::get_scheduled_for_user(&mut *txn, user_tg_id, DAILY_EVENTS).await?;

        if pending_daily_events_polls.is_empty() {
            let poll = Poll::create(&mut *txn, user_tg_id, DAILY_EVENTS, None).await?;
            send_poll(bot, &mut *txn, poll).await?;
        }

        return Ok(());
    }

    if options::ALL_BUTTON.matches(payload) {
        let to_update: Vec<_> = options::ALL_OPTIONS
            .values()
            .filter(|o| !current.options.contains(o.value()))
            .collect();

        if to_update.is_empty() {
            bot.answer_callback_query(&callback.id).await?;

            return Ok(());
        }
        for option in to_update {
            // TODO: bulk insert
            PollCustomOptions::toggle_option(&mut *txn, user_tg_id, DAILY_EVENTS, option.value())
                .await?;
        }
    } else if options::NONE_BUTTON.matches(payload) {
        if current.options.is_empty() {
            bot.answer_callback_query(&callback.id).await?;

            return Ok(());
        }

        PollCustomOptions::clear_user_options(txn, user_tg_id, DAILY_EVENTS).await?;
    } else if let Some(option) = options::ALL_OPTIONS.values().find(|o| o.matches(payload)) {
        PollCustomOptions::toggle_option(&mut *txn, user_tg_id, DAILY_EVENTS, option.value())
            .await?;
    } else {
        warn!("got unknown payload");
    }

    let keyboard = user_daily_options(txn, user_tg_id).await?;

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
    const POLL_KIND: PollKind = PollKind::DailyEvents;

    if options::PROMO_NO_BUTTON.matches(payload) {
        bot.send_message(
            user_tg_id.to_string(),
            md_message!("promo/daily_events_no.md"),
        )
        .await?;
    } else if options::PROMO_YES_BUTTON.matches(payload) {
        let keyboard = user_daily_options(txn, user_tg_id).await?;

        bot.send_message(
            user_tg_id.to_string(),
            md_message!("daily_events/settings.md"),
        )
        .reply_markup(keyboard)
        .await?;
    } else {
        warn!("got unknown payload");
    }

    bot.answer_callback_query(&callback.id).await?;

    Ok(())
}
