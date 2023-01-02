use std::time::Duration;

use color_eyre::Result;
use sqlx::PgPool;
use teloxide::{payloads::SendMessageSetters, requests::Requester};

use ilquentir_messages::md_message;
use ilquentir_models::{Poll, PollKind, User};

use crate::bot::{daily_events, helpers::set_typing, Bot};

#[tracing::instrument(skip(bot, pool), err)]
pub async fn poll_answered(bot: &Bot, pool: &PgPool, poll: &Poll) -> Result<()> {
    let chat_id = poll.chat_tg_id;
    let kind = poll.kind;

    set_typing(bot, chat_id.to_string(), Some(Duration::from_millis(200))).await?;

    if User::count_answered_polls(&mut pool.begin().await?, chat_id, kind).await? == 1 {
        // onboarding
        bot.send_message(
            chat_id.to_string(),
            md_message!("onboarding/step_4_after_response.md"),
        )
        .await?;
        set_typing(bot, chat_id.to_string(), Some(Duration::from_millis(1000))).await?;

        bot.send_message(
            chat_id.to_string(),
            md_message!("onboarding/step_5_after_response.md"),
        )
        .reply_markup(daily_events::keyboard::promo())
        .await?;

        return Ok(());
    }

    // main flow
    bot.send_message(
        chat_id.to_string(),
        md_message!("promo/interactive_graphs.md"),
    )
    .await?;

    if Poll::get_scheduled_for_user(&mut pool.begin().await?, chat_id, PollKind::DailyEvents)
        .await?
        .is_empty()
    {
        set_typing(bot, chat_id.to_string(), Some(Duration::from_millis(2500))).await?;

        bot.send_message(chat_id.to_string(), md_message!("promo/daily_events.md"))
            .reply_markup(daily_events::keyboard::promo())
            .await?;
    }

    Ok(())
}
