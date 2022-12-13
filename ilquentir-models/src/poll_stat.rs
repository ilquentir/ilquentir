use color_eyre::Result;
use sqlx::FromRow;

use crate::{PgTransaction, PollKind};

#[derive(Debug, Clone, FromRow)]
pub struct PollStat {
    pub kind: PollKind,
    pub date: time::Date,
    pub selected_value: i32,
    pub n_selected: i64,
}

impl PollStat {
    #[tracing::instrument(skip(txn), err)]
    pub async fn get_today_stats<'t>(
        txn: &mut PgTransaction<'t>,
        kind: PollKind,
    ) -> Result<Vec<PollStat>> {
        // TODO: get interval from poll kind
        Ok(sqlx::query_as!(
            Self,
            r#"
SELECT
    polls.kind as "kind: PollKind",
    DATE(polls.publication_date - interval '12 hours') as "date!",
    selected_value,
    COUNT(poll_answers.id) as "n_selected!"
FROM poll_answers
JOIN polls
ON poll_answers.poll_tg_id = polls.tg_id
WHERE
    polls.kind = $1
    AND DATE(polls.publication_date - interval '12 hours') = DATE(NOW() - interval '22 hours')
GROUP BY
    selected_value,
    polls.kind,
    DATE(polls.publication_date - interval '12 hours')
            "#,
            kind.to_string()
        )
        .fetch_all(txn)
        .await?)
    }
}
