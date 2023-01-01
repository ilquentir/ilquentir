use color_eyre::Result;
use time::OffsetDateTime;

use crate::PgTransaction;

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct WideHowWasYourDay {
    #[serde(with = "time::serde::iso8601")]
    pub publication_date: OffsetDateTime,
    pub user_tg_id: i64,
    pub answer_selected_value: Option<i32>,
    pub events: Option<String>,
}

impl WideHowWasYourDay {
    pub async fn collect(txn: &mut PgTransaction<'_>) -> Result<Vec<Self>> {
        Ok(sqlx::query_as!(
            Self,
            r#"
SELECT
    polls.publication_date as "publication_date!",
    polls.chat_tg_id as "user_tg_id!",
    poll_answers.selected_value as "answer_selected_value?",
    ARRAY_TO_STRING(event_polls.events, ', ') as events
FROM polls
LEFT JOIN
    poll_answers
ON
    polls.tg_id = poll_answers.poll_tg_id
LEFT JOIN
    (
        SELECT
            polls.publication_date,
            polls.chat_tg_id as user_tg_id,
            ARRAY_AGG(poll_answers.selected_value_text) as "events"
        FROM polls
        JOIN
            poll_answers
        ON
            polls.tg_id = poll_answers.poll_tg_id
        WHERE
            polls.published
            AND polls.kind = 'daily_events'
        GROUP BY
            polls.publication_date,
            polls.chat_tg_id
    ) event_polls
ON
    polls.publication_date = event_polls.publication_date
WHERE
    polls.published
    AND polls.kind = 'how_was_your_day'
            "#
        )
        .fetch_all(txn)
        .await?)
    }
}
