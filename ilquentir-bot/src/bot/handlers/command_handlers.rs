use std::time::Duration;

use color_eyre::Result;
use ilquentir_messages::md_message_payload;
use sqlx::{PgPool, Postgres, Transaction};
use teloxide::{
    requests::JsonRequest,
    requests::Requester,
    types::{ChatId, Message},
    Bot,
};
use tracing::info;

use ilquentir_graphs::{daily::chart_daily_stats, weekly::personal_weekly_stat};
use ilquentir_models::{Poll, PollKind, PollStat, PollWeeklyUserStat, User};

use crate::bot::{helpers::set_typing, Command};

#[tracing::instrument(skip(bot, pool), err)]
pub async fn handle_command(bot: Bot, pool: PgPool, msg: Message, command: Command) -> Result<()> {
    let mut txn = pool.begin().await?;

    match command {
        Command::Start => {
            handle_start(&bot, &mut txn, msg.chat.id.0).await?;
        }
        Command::Stop => {
            info!(chat_id = %msg.chat.id, "processing Stop command");

            let user = User::deactivate(&mut txn, msg.chat.id.0).await?;

            info!(chat_id = %msg.chat.id, user_id = user.tg_id, "disabled user");

            bot.send_message(msg.chat.id, "Деактивировали! Надеюсь, ещё увидимся :)")
                .await?;
        }
        Command::GetStat => {
            let stats =
                PollStat::get_today_stats(&mut pool.begin().await?, PollKind::HowWasYourDay)
                    .await?;
            let graph = chart_daily_stats(&stats)?;

            let your_stat = PollWeeklyUserStat::get_for_last_week(
                &mut pool.begin().await?,
                PollKind::HowWasYourDay,
                msg.chat.id.0,
            )
            .await?;
            let your_stat_descr = personal_weekly_stat(&your_stat);

            let message_payload = md_message_payload!(
                msg.chat.id,
                "stats/get_stat.md",
                graph = graph,
                your_stat_descr = your_stat_descr,
            );
            JsonRequest::new(bot, message_payload).await?;
        }
    }
    txn.commit().await?;

    Ok(())
}

#[tracing::instrument(skip(bot, txn), err)]
pub async fn handle_start(
    bot: &Bot,
    txn: &mut Transaction<'_, Postgres>,
    chat_id: i64,
) -> Result<()> {
    info!(chat_id, "processing Start command");
    let chat_id_wrapped = ChatId(chat_id);

    if let Some(user) = User::get_user_by_id(&mut *txn, chat_id).await? {
        info!(
            chat_id,
            user_id = user.tg_id,
            "user already exists and active :)"
        );

        bot.send_message(chat_id_wrapped, "И снова здравствуй :)")
            .await?;

        return Ok(());
    }

    let user = User::activate(&mut *txn, chat_id).await?;
    info!(chat_id, user_id = user.tg_id, "(re?) activated user");

    let polls = Poll::create_for_user(&mut *txn, &user).await?;

    info!(
        chat_id,
        user_id = user.tg_id,
        "sending welcome sequence to user"
    );
    bot.send_message(chat_id_wrapped, "Ильквентир приветствует тебя! :)")
        .await?;
    set_typing(bot, chat_id, Some(Duration::from_millis(300))).await?;

    bot.send_message(
        chat_id_wrapped,
        r#"Я помогу тебе трекать своё состояние ежедневно и замечать тренды.

Раз в неделю я буду присылать подробную стату про тебя и прошедшую неделю."#,
    )
    .await?;

    info!(
        chat_id,
        user_id = user.tg_id,
        "sent welcome sequence, sending {} initial polls",
        polls.len(),
    );
    set_typing(bot, chat_id, Some(Duration::from_millis(500))).await?;

    for poll in polls {
        poll.publish_to_tg(&mut *txn, bot).await?;
    }

    info!(chat_id, user_id = user.tg_id, "initial polls sent");

    Ok(())
}
