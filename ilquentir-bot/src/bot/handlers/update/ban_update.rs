use color_eyre::Result;
use sqlx::PgPool;
use teloxide::types::{ChatMemberUpdated, Me};
use tracing::{info, warn};

use ilquentir_models::User;

#[tracing::instrument(skip(pool), err)]
pub async fn handle_ban(pool: PgPool, member_info: ChatMemberUpdated, me: Me) -> Result<()> {
    if !member_info.new_chat_member.kind.is_banned() {
        warn!("tried to handle unknown update");

        return Ok(());
    }

    if member_info.new_chat_member.user.id != me.id {
        warn!("tried to handle ban for some other user");

        return Ok(());
    }

    let mut txn = pool.begin().await?;

    User::deactivate(&mut txn, member_info.chat.id.0).await?;

    info!(
        chat_id = member_info.chat.id.0,
        "deactivated user with username {:?}",
        member_info.chat.username()
    );

    Ok(())
}
