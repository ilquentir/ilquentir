use std::time::Duration;

use color_eyre::Result;
use sqlx::PgPool;
use teloxide::{payloads::SendMessageSetters, requests::Requester};

use ilquentir_config::Config;
use ilquentir_graphs::{daily::chart_daily_stats, weekly::personal_weekly_stat};
use ilquentir_messages::md_message;
use ilquentir_models::{Poll, PollKind, PollStat, PollWeeklyUserStat};

use crate::bot::{daily_events::keyboard::promo_keyboard, helpers::set_typing, Bot};

#[tracing::instrument(skip(bot, pool, config), err)]
pub async fn poll_answered(bot: &Bot, pool: &PgPool, poll: &Poll, config: Config) -> Result<()> {
    let chat_id = poll.chat_tg_id;
    let kind = poll.kind;
    let publication_date = poll.publication_date;

    let bot_clone = bot.clone();
    let pool_clone = pool.clone();
    tokio::spawn(async move {
        let delay = publication_date + config.min_reply_delay - time::OffsetDateTime::now_utc();
        if delay.is_positive() {
            tokio::time::sleep(delay.unsigned_abs()).await;
        } else {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        let stats = PollStat::get_today_stats(&mut pool_clone.begin().await.unwrap(), kind)
            .await
            .unwrap();
        let graph = chart_daily_stats(&stats).unwrap();

        let message_text = md_message!("voted_poll_reaction/stats_for_today.md", graph = graph);

        bot_clone
            .send_message(chat_id.to_string(), message_text)
            .await
            .unwrap();
    });

    set_typing(bot, chat_id.to_string(), Some(Duration::from_millis(200))).await?;

    let your_stat = PollWeeklyUserStat::get_for_last_week(
        &mut pool.begin().await?,
        PollKind::HowWasYourDay,
        chat_id,
    )
    .await?;
    let your_stat_descr = personal_weekly_stat(&your_stat);

    bot.send_message(
        chat_id.to_string(),
        md_message!(
            "voted_poll_reaction/generic_response.md",
            your_stat_descr = your_stat_descr,
        ),
    )
    .await?;

    if Poll::get_scheduled_for_user(&mut pool.begin().await?, chat_id, PollKind::DailyEvents)
        .await?
        .is_empty()
    {
        set_typing(bot, chat_id.to_string(), Some(Duration::from_secs(2))).await?;

        bot.send_message(chat_id.to_string(), md_message!("daily_events/promo.md"))
            .reply_markup(promo_keyboard())
            .await?;
    }

    Ok(())
}
