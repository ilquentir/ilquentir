use color_eyre::{eyre::eyre, Result};
use sqlx::PgPool;
use teloxide::types::CallbackQuery;

use crate::bot::{callbacks::Scope, daily_events, Bot};

#[tracing::instrument(skip(bot, pool), err)]
pub async fn handle_callback(bot: Bot, pool: PgPool, callback: CallbackQuery) -> Result<()> {
    let data = callback
        .data
        .as_ref()
        .ok_or_else(|| eyre!("callback without data"))?;
    let scope = Scope::from_payload(data).ok_or_else(|| eyre!("scope extraction failed"))?;

    let mut txn = pool.begin().await?;

    match scope {
        Scope::DailyEvents => {
            daily_events::handle_callback(&bot, &mut txn, &callback, data).await?
        }
        Scope::PromoDailyEvents => {
            daily_events::handle_promo_callback(&bot, &mut txn, &callback, data).await?;
        }
    };

    txn.commit().await?;

    Ok(())
}
