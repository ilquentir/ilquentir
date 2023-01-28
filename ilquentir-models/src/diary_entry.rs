use color_eyre::Result;
use sqlx::FromRow;
use time::OffsetDateTime;

use crate::PgTransaction;

#[derive(Debug, Clone, FromRow)]
pub struct DiaryEntry {
    pub user_tg_id: i64,
    pub text: String,
    pub date_created: OffsetDateTime,
}

impl DiaryEntry {
    #[tracing::instrument(skip(txn), err)]
    pub async fn insert(txn: &mut PgTransaction<'_>, user_tg_id: i64, text: &str) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
INSERT INTO diary_entries (
    user_tg_id, text
)
VALUES ($1, $2)
RETURNING
    user_tg_id,
    text,
    date_created
            "#,
            user_tg_id,
            text
        )
        .fetch_one(txn)
        .await?)
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn get_for_user(txn: &mut PgTransaction<'_>, user_tg_id: i64) -> Result<Vec<Self>> {
        Ok(sqlx::query_as!(
            Self,
            r#"
SELECT
    user_tg_id,
    text,
    date_created
FROM
    diary_entries
WHERE
    user_tg_id = $1
            "#,
            user_tg_id
        )
        .fetch_all(txn)
        .await?)
    }
}
