use color_eyre::Result;
use sqlx::FromRow;
use tracing::{info, warn};

use crate::{PgTransaction, PollKind};

#[derive(Debug, Clone, FromRow)]
struct PollCustomOptionsRaw {
    option_text: String,
}

#[derive(Debug, Clone)]
pub struct PollCustomOptions {
    pub poll_kind: PollKind,
    pub user_tg_id: i64,
    pub options: Vec<String>,
}

impl PollCustomOptions {
    #[tracing::instrument(skip(txn), err)]
    pub async fn get_for_user(
        txn: &mut PgTransaction<'_>,
        user_tg_id: i64,
        poll_kind: PollKind,
    ) -> Result<Self> {
        let options = sqlx::query_as!(
            PollCustomOptionsRaw,
            r#"
SELECT
    option_text
FROM
    poll_custom_options
WHERE
    user_tg_id = $1
    AND poll_kind = $2
            "#,
            user_tg_id,
            poll_kind.to_string()
        )
        .fetch_all(txn)
        .await?
        .into_iter()
        .map(|option| option.option_text)
        .collect();

        Ok(Self {
            poll_kind,
            user_tg_id,
            options,
        })
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn toggle_option(
        txn: &mut PgTransaction<'_>,
        user_tg_id: i64,
        poll_kind: PollKind,
        option: &str,
    ) -> Result<()> {
        let poll_kind = poll_kind.to_string();

        info!("checking, if option is enabled or not");

        let exists = sqlx::query!(
            r#"
SELECT
    id
FROM
    poll_custom_options
WHERE
    poll_kind = $1
    AND user_tg_id = $2
    AND option_text = $3
            "#,
            poll_kind,
            user_tg_id,
            option,
        )
        .fetch_optional(&mut *txn)
        .await?
        .is_some();

        if exists {
            info!("option exists, removing");

            let deleted = sqlx::query!(
                r#"
DELETE FROM poll_custom_options
WHERE
    poll_kind = $1
    AND user_tg_id = $2
    AND option_text = $3
                "#,
                poll_kind,
                user_tg_id,
                option,
            )
            .execute(txn)
            .await?
            .rows_affected();

            if deleted == 0 {
                warn!("tried to enable already enabled option")
            }
        } else {
            let inserted = sqlx::query!(
                r#"
INSERT INTO poll_custom_options (
    poll_kind, user_tg_id, option_text
)
VALUES ($1, $2, $3)
                "#,
                poll_kind.to_string(),
                user_tg_id,
                option,
            )
            .execute(txn)
            .await?
            .rows_affected();

            if inserted == 0 {
                warn!("tried to enable already enabled option")
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn clear_user_options(
        txn: &mut PgTransaction<'_>,
        user_tg_id: i64,
        poll_kind: PollKind,
    ) -> Result<()> {
        sqlx::query!(
            r#"
DELETE FROM poll_custom_options
WHERE
    user_tg_id = $1
    AND poll_kind = $2
            "#,
            user_tg_id,
            poll_kind.to_string(),
        )
        .execute(txn)
        .await?;

        Ok(())
    }
}
