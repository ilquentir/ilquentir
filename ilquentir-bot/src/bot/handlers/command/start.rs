use std::time::Duration;

use color_eyre::Result;
use teloxide::{requests::Requester, types::ChatId};
use tracing::info;

use ilquentir_messages::md;
use ilquentir_models::{PgTransaction, Poll, User};

use crate::bot::{
    helpers::{send_poll, set_typing},
    Bot,
};

#[tracing::instrument(skip(bot, txn), err)]
pub async fn handle_start(bot: &Bot, txn: &mut PgTransaction<'_>, chat_id: ChatId) -> Result<()> {
    info!(chat_id = chat_id.0, "processing Start command");

    if let Some(user) = User::get_user_by_id(&mut *txn, chat_id.0).await? {
        info!(
            chat_id = chat_id.0,
            user_id = user.tg_id,
            "user already exists and active :)"
        );

        bot.send_message(chat_id, md!("И снова здравствуй :)"))
            .await?;

        return Ok(());
    }

    let user = User::activate(&mut *txn, chat_id.0).await?;
    info!(
        chat_id = chat_id.0,
        user_id = user.tg_id,
        "(re?) activated user"
    );

    let polls = Poll::create_for_user(&mut *txn, &user).await?;

    info!(
        chat_id = chat_id.0,
        user_id = user.tg_id,
        "sending welcome sequence to user"
    );
    bot.send_message(chat_id, md!("Ильквентир приветствует тебя! :)"))
        .await?;
    set_typing(bot, chat_id, Some(Duration::from_millis(300))).await?;

    bot.send_message(
        chat_id,
        md!(
            r#"Я помогу тебе трекать своё состояние ежедневно и замечать тренды.

Раз в неделю я буду присылать подробную стату про тебя и прошедшую неделю."#
        ),
    )
    .await?;

    info!(
        chat_id = chat_id.0,
        user_id = user.tg_id,
        "sent welcome sequence, sending {} initial polls",
        polls.len(),
    );
    set_typing(bot, chat_id, Some(Duration::from_millis(500))).await?;

    for poll in polls {
        let poll_message = send_poll(bot, &mut *txn, &poll).await?;

        poll.published_to_tg(&mut *txn, poll_message).await?;
    }

    info!(
        chat_id = chat_id.0,
        user_id = user.tg_id,
        "initial polls sent"
    );

    Ok(())
}