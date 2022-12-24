use color_eyre::Result;
use sqlx::FromRow;
use teloxide::types::Poll as TgPoll;
use tracing::info;

use crate::{PgTransaction, Poll};

#[derive(Debug, Clone, FromRow)]
pub struct PollAnswer {
    pub poll_tg_id: String,
    pub selected_value: i32,
    pub selected_value_text: String,
}

impl PollAnswer {
    #[tracing::instrument(skip(txn), err)]
    pub async fn save_answer(txn: &mut PgTransaction<'_>, tg_poll: &TgPoll) -> Result<Poll> {
        info!(tg_poll = tg_poll.id, "saving results for poll");
        let options = tg_poll
            .options
            .iter()
            .enumerate()
            .filter(|(_idx, option)| option.voter_count > 0);

        for (idx, option) in options {
            sqlx::query_as!(
                Self,
                r#"
INSERT INTO poll_answers (poll_tg_id, selected_value, selected_value_text)
VALUES ($1, $2, $3)
RETURNING poll_tg_id, selected_value, selected_value_text
                "#,
                tg_poll.id,
                idx as i32,
                option.text,
            )
            .fetch_one(&mut *txn)
            .await?;
        }

        Poll::get_by_tg_id(txn, &tg_poll.id).await
    }
}
