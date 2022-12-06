use std::time::Duration;

use color_eyre::Result;
use sqlx::{PgPool, Postgres, Transaction};
use teloxide::{
    payloads::SendChatAction,
    requests::JsonRequest,
    requests::Requester,
    types::{ChatId, Message},
    Bot,
};
use tracing::info;

use crate::{
    bot::{helpers::set_typing, Command},
    models::{Poll, User},
};

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

            bot.send_message(
                msg.chat.id,
                "Deactivation succeeded. Hope you'll be back soon!",
            )
            .await?;
        }
        Command::GetStat => {
            bot.send_message(msg.chat.id, "get_stat is currently unimplemented :(")
                .await?;
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
        info!(chat_id, user_id = user.tg_id, "user already exists and active :)");

        bot.send_message(chat_id_wrapped, "Hello! Nice to see you again :)")
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
    bot.send_message(chat_id_wrapped, "Hello! Ilquentir welcomes you.")
        .await?;
    let payload = SendChatAction::new(chat_id_wrapped, teloxide::types::ChatAction::Typing);
    JsonRequest::new(bot.clone(), payload.clone()).await?;

    set_typing(bot, chat_id, Some(Duration::from_millis(300))).await?;

    bot.send_message(
        chat_id_wrapped,
r#"Log your status daily, track your feelings and notice trends.
Every week we will be sending you stats about personal and communal trends.

Your daily poll will be sent to you at 19:00 UTC. In your first poll, give an answer for the previous day :)"#,
    )
    .await?;
    JsonRequest::new(bot.clone(), payload.clone()).await?;

    info!(
        chat_id,
        user_id = user.tg_id,
        "sent welcome sequence, sending {} initial polls",
        polls.len(),
    );

    set_typing(bot, chat_id, Some(Duration::from_millis(1000))).await?;

    for poll in polls {
        poll.publish_to_tg(&mut *txn, bot).await?;
    }

    info!(chat_id, user_id = user.tg_id, "initial polls sent");

    Ok(())
}
