use color_eyre::Result;
use sqlx::FromRow;
use time::Date;

use crate::{PgTransaction, PollKind};

#[derive(Debug, Clone, FromRow)]
pub struct PollWeeklyUserStat {
    kind: PollKind,
    date: Date,
    selected_value: i32,
}

impl PollWeeklyUserStat {
    #[tracing::instrument(skip(txn), err)]
    pub async fn get_for_last_week<'t>(
        txn: &mut PgTransaction<'t>,
        kind: PollKind,
    ) -> Result<Vec<Self>> {
        Ok(sqlx::query_as!(
            Self,
            r#"
SELECT
    polls.kind as "kind: PollKind",
    DATE(polls.publication_date) as "date!",
    poll_answers.selected_value
FROM poll_answers
JOIN polls
ON poll_answers.poll_tg_id = polls.tg_id
WHERE
    polls.kind = $1
    AND polls.publication_date BETWEEN NOW() - interval '7 days' AND NOW() - interval '1 minute'
ORDER BY "date!"
            "#,
            kind.to_string()
        )
        .fetch_all(txn)
        .await?)
    }
}
