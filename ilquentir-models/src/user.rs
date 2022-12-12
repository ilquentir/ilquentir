use color_eyre::Result;
use sqlx::FromRow;

use crate::{PgTransaction, PollKind};

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub tg_id: i64,
    pub active: bool,
}

impl User {
    #[tracing::instrument(skip(txn), err)]
    pub async fn get_user_by_id<'t>(
        txn: &mut PgTransaction<'t>,
        user_id: i64,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query_as!(
            Self,
            r#"
SELECT tg_id, active
FROM users
WHERE
    tg_id = $1
    AND active
            "#,
            user_id,
        )
        .fetch_optional(&mut *txn)
        .await?)
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn activate<'t>(txn: &mut PgTransaction<'t>, user_id: i64) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
INSERT INTO users (tg_id, active)
VALUES ($1, true)
ON CONFLICT (tg_id) DO UPDATE SET active = true
RETURNING tg_id, active
            "#,
            user_id,
        )
        .fetch_one(txn)
        .await?)
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn deactivate<'t>(txn: &mut PgTransaction<'t>, user_id: i64) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
INSERT INTO users (tg_id, active)
VALUES ($1, false)
ON CONFLICT (tg_id) DO UPDATE SET active = false
RETURNING tg_id, active
            "#,
            user_id,
        )
        .fetch_one(txn)
        .await?)
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn count_answered_polls<'t>(
        txn: &mut PgTransaction<'t>,
        user_id: i64,
    ) -> Result<i64> {
        Ok(sqlx::query!(
            r#"
SELECT COUNT(DISTINCT poll.tg_id) as "n_answered!"
FROM polls AS poll
JOIN poll_answers AS answer
ON poll.tg_id = answer.poll_tg_id
WHERE poll.chat_tg_id = $1
            "#,
            user_id,
        )
        .fetch_one(txn)
        .await?
        .n_answered)
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn get_active<'t>(txn: &mut PgTransaction<'t>) -> Result<Vec<Self>> {
        Ok(sqlx::query_as!(
            Self,
            r#"
SELECT tg_id, active
FROM users
WHERE active
            "#
        )
        .fetch_all(txn)
        .await?)
    }

    #[tracing::instrument]
    pub fn subscribed_for_polls(&self) -> Vec<PollKind> {
        vec![PollKind::HowWasYourDay]
    }
}
