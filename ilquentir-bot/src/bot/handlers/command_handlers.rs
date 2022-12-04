use std::time::Duration;

use color_eyre::Result;
use sqlx::{Transaction, Postgres};
use teloxide::{Bot, requests::Requester, payloads::SendChatAction, requests::JsonRequest, types::ChatId};
use tracing::info;

use crate::models::{Poll, User};

#[tracing::instrument(skip(bot, txn))]
pub async fn handle_start(bot: &Bot, txn: &mut Transaction<'_, Postgres>, chat_id: i64) -> Result<()> {
    info!(chat_id, "processing Start command");

    let user = User::activate(&mut *txn, chat_id).await?;
    info!(chat_id, user_id = user.tg_id, "(re?) activated user");

    Poll::create_for_user(&mut *txn, &user).await?;

    info!(chat_id, user_id = user.tg_id, "sending welcome sequence to user");
    let chat_id_wrapped = ChatId(chat_id);

    bot.send_message(chat_id_wrapped, "Hello! Ilquentir welcomes you.")
        .await?;
    let payload = SendChatAction::new(chat_id_wrapped, teloxide::types::ChatAction::Typing);
    JsonRequest::new(bot.clone(), payload).await?;

    info!(chat_id, user_id = user.tg_id, "send first part, 'typing' now");
    tokio::time::sleep(Duration::from_millis(1400)).await;

    bot.send_message(chat_id_wrapped, r#"Log your everyday status, track your feelings and notice trends.
Every week we will send interesting stats about you and community's trends."#).await?;
    info!(chat_id, user_id = user.tg_id, "sent welcome sequence");


    Ok(())
}