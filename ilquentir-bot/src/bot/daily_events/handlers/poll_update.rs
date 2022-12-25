use std::time::Duration;

use color_eyre::Result;
use sqlx::PgPool;
use teloxide::{requests::Requester, types::ChatId};

use ilquentir_messages::md_message;
use ilquentir_models::{Poll, PollKind, User};

use crate::bot::{helpers::set_typing, Bot};

#[tracing::instrument(skip(bot, pool), err)]
pub async fn poll_answered(bot: &Bot, pool: &PgPool, poll: &Poll) -> Result<()> {
    if User::count_answered_polls(
        &mut pool.begin().await?,
        poll.chat_tg_id,
        PollKind::DailyEvents,
    )
    .await?
        == 1
    {
        let chat_id = ChatId(poll.chat_tg_id);

        set_typing(bot, chat_id, Some(Duration::from_millis(200))).await?;
        bot.send_message(
            chat_id,
            md_message!("daily_events/first_poll_answer_reaction.md"),
        )
        .await?;
    }

    Ok(())
}
