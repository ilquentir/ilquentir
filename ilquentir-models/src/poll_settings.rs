use color_eyre::Result;
use sqlx::FromRow;
use time::Time;

use crate::{PgTransaction, PollKind};

#[derive(Debug, Clone, FromRow)]
pub struct PollSettings {
    pub poll_kind: PollKind,
    pub user_tg_id: i64,
    pub send_at_utc: Option<Time>,
}

impl PollSettings {
    #[tracing::instrument(skip(txn), err)]
    pub async fn get(
        txn: &mut PgTransaction<'_>,
        user_tg_id: i64,
        poll_kind: PollKind,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query_as!(
            Self,
            r#"
SELECT
    poll_kind as "poll_kind: _",
    user_tg_id,
    send_at_utc
FROM
    poll_settings
WHERE
    user_tg_id = $1
    AND poll_kind = $2
            "#,
            user_tg_id,
            poll_kind.to_string()
        )
        .fetch_optional(txn)
        .await?)
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn set_send_at(
        txn: &mut PgTransaction<'_>,
        user_tg_id: i64,
        poll_kind: PollKind,
        send_at_utc: Time,
    ) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
INSERT INTO poll_settings (
    user_tg_id,
    poll_kind,
    send_at_utc
)
VALUES ($1, $2, $3)
ON CONFLICT ON CONSTRAINT poll_settings_poll_kind_user_tg_id_key DO
UPDATE SET
    send_at_utc = $3
RETURNING
    user_tg_id,
    poll_kind as "poll_kind: _",
    send_at_utc
            "#,
            user_tg_id,
            poll_kind.to_string(),
            send_at_utc
        )
        .fetch_one(txn)
        .await?)
    }
}
