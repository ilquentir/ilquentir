use color_eyre::Result;
use time::OffsetDateTime;

use crate::PgTransaction;

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct WideHowWasYourDay {
    #[serde(with = "time::serde::rfc3339")]
    pub poll_date_about: OffsetDateTime,
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
    date_trunc('day', polls.publication_date - INTERVAL '12 hours') as "poll_date_about!",
    polls.chat_tg_id as "user_tg_id!",
    poll_answers.selected_value as "answer_selected_value?",
    event_polls.events
FROM polls
LEFT JOIN
    poll_answers
ON
    polls.tg_id = poll_answers.poll_tg_id
LEFT JOIN
(
    SELECT
        polls.publication_date,
        polls.chat_tg_id as chat_tg_id,
        ARRAY_TO_STRING(ARRAY_AGG('• ' || poll_answers.selected_value_text), ',<br>') as "events"
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
    polls.chat_tg_id = event_polls.chat_tg_id
    AND polls.publication_date = event_polls.publication_date
WHERE
    polls.published
    AND polls.kind = 'how_was_your_day'
            "#
        )
        .fetch_all(txn)
        .await?)
    }
}
