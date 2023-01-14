use std::time::Duration;

use color_eyre::Result;
use teloxide::{requests::Requester, types::ChatId};
use tracing::info;

use ilquentir_messages::{md, md_message};
use ilquentir_models::{PgTransaction, Poll, User};

use crate::bot::{
    helpers::{send_poll, set_typing},
    Bot,
};

#[tracing::instrument(skip_all, fields(chat_id=chat_id.0), err)]
pub async fn handle_start(bot: &Bot, txn: &mut PgTransaction<'_>, chat_id: ChatId) -> Result<()> {
    info!(chat_id = chat_id.0, "processing Start command");

    if let Some(user) = User::get_user_by_id(&mut *txn, chat_id.0).await? {
        info!(
            chat_id = chat_id.0,
            user_tg_id = user.tg_id,
            "user already exists and active :)"
        );

        bot.send_message(chat_id, md!("И снова здравствуй :)"))
            .await?;

        return Ok(());
    }

    let user = User::activate(&mut *txn, chat_id.0).await?;
    info!(
        chat_id = chat_id.0,
        user_tg_id = user.tg_id,
        "(re?) activated user"
    );

    let polls = Poll::create_for_user(&mut *txn, &user).await?;

    info!(
        chat_id = chat_id.0,
        user_tg_id = user.tg_id,
        "sending welcome sequence to user"
    );
    bot.send_message(chat_id, md_message!("onboarding/step_1.md"))
        .await?;
    set_typing(bot, chat_id, Some(Duration::from_millis(500))).await?;

    bot.send_message(chat_id, md_message!("onboarding/step_2.md"))
        .await?;
    set_typing(bot, chat_id, Some(Duration::from_millis(1000))).await?;

    bot.send_message(chat_id, md_message!("onboarding/step_3.md"))
        .await?;
    set_typing(bot, chat_id, Some(Duration::from_millis(1000))).await?;

    info!(
        chat_id = chat_id.0,
        user_tg_id = user.tg_id,
        "sent welcome sequence, sending {} initial polls",
        polls.len(),
    );
    set_typing(bot, chat_id, Some(Duration::from_millis(500))).await?;

    for poll in polls {
        send_poll(bot, &mut *txn, poll).await?;
    }

    info!(
        chat_id = chat_id.0,
        user_tg_id = user.tg_id,
        "initial polls sent"
    );

    Ok(())
}
