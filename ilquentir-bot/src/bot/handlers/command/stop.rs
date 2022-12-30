use color_eyre::Result;
use teloxide::{requests::Requester, types::ChatId};
use tracing::info;

use ilquentir_messages::md;
use ilquentir_models::{PgTransaction, User};

use crate::bot::Bot;

#[tracing::instrument(skip(bot, txn), fields(chat_id=chat_id.0), err)]
pub async fn handle_stop(bot: &Bot, txn: &mut PgTransaction<'_>, chat_id: ChatId) -> Result<()> {
    let user = User::deactivate(txn, chat_id.0).await?;

    info!(user = user.tg_id, "disabled user");

    bot.send_message(chat_id, md!("Деактивировали! Надеюсь, ещё увидимся :)"))
        .await?;

    Ok(())
}
