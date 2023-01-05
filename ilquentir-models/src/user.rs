use color_eyre::Result;
use sqlx::FromRow;
use strum::IntoEnumIterator;

use crate::{PgTransaction, Poll, PollKind};

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub tg_id: i64,
    pub active: bool,
}

impl User {
    #[tracing::instrument(skip(txn), err)]
    pub async fn get_user_by_id(txn: &mut PgTransaction<'_>, user_tg_id: i64) -> Result<Option<Self>> {
        Ok(sqlx::query_as!(
            Self,
            r#"
SELECT tg_id, active
FROM users
WHERE
    tg_id = $1
    AND active
            "#,
            user_tg_id,
        )
        .fetch_optional(&mut *txn)
        .await?)
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn activate(txn: &mut PgTransaction<'_>, user_tg_id: i64) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
INSERT INTO users (tg_id, active)
VALUES ($1, true)
ON CONFLICT (tg_id) DO UPDATE SET active = true
RETURNING tg_id, active
            "#,
            user_tg_id,
        )
        .fetch_one(txn)
        .await?)
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn deactivate(txn: &mut PgTransaction<'_>, user_tg_id: i64) -> Result<Self> {
        for kind in PollKind::iter() {
            Poll::disable_pending_for_user(&mut *txn, user_tg_id, kind).await?;
        }

        Ok(sqlx::query_as!(
            Self,
            r#"
UPDATE
    users
SET
    active = false
WHERE
    tg_id = $1
RETURNING
    tg_id, active
            "#,
            user_tg_id,
        )
        .fetch_one(txn)
        .await?)
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn count_answered_polls(
        txn: &mut PgTransaction<'_>,
        user_tg_id: i64,
        kind: PollKind,
    ) -> Result<i64> {
        Ok(sqlx::query!(
            r#"
SELECT
    COUNT(DISTINCT poll.tg_id) as "n_answered!"
FROM
    polls AS poll
JOIN
    poll_answers AS answer
ON
    poll.tg_id = answer.poll_tg_id
WHERE
    poll.chat_tg_id = $1
    AND poll.kind = $2
            "#,
            user_tg_id,
            kind.to_string(),
        )
        .fetch_one(txn)
        .await?
        .n_answered)
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn get_active(txn: &mut PgTransaction<'_>) -> Result<Vec<Self>> {
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
