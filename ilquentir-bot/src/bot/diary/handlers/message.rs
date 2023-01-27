use color_eyre::{eyre::eyre, Result};
use ilquentir_messages::md_message;
use sqlx::PgPool;
use teloxide::{
    requests::Requester,
    types::{MediaText, Message},
};

use ilquentir_models::DiaryEntry;

use crate::bot::Bot;

#[tracing::instrument(skip(bot, pool), err)]
pub async fn append_diary_entry(
    bot: Bot,
    pool: PgPool,
    msg: Message,
    msg_text: MediaText,
) -> Result<()> {
    let mut txn = pool.begin().await?;

    let user_tg_id = msg
        .from()
        .map(|u| u.id.0)
        .ok_or_else(|| eyre!("trying to handle message without known author, aborting"))?;

    DiaryEntry::insert(&mut txn, user_tg_id as i64, &msg_text.text).await?;
    bot.send_message(msg.chat.id, md_message!("diary/new_entry.md"))
        .await?;

    txn.commit().await?;

    Ok(())
}
