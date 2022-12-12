use color_eyre::Result;
use sqlx::FromRow;
use teloxide::types::Poll as TgPoll;
use tracing::info;

use crate::{PgTransaction, Poll};

#[derive(Debug, Clone, FromRow)]
pub struct PollAnswer {
    pub poll_tg_id: String,
    pub selected_value: i32,
}

impl PollAnswer {
    #[tracing::instrument(skip(txn), err)]
    pub async fn save_answer<'t>(txn: &mut PgTransaction<'t>, tg_poll: &TgPoll) -> Result<Poll> {
        info!(tg_poll = tg_poll.id, "saving results for poll");
        let option_ids = tg_poll
            .options
            .iter()
            .enumerate()
            .filter_map(|(idx, option)| (option.voter_count > 0).then_some(idx));

        for choice in option_ids {
            sqlx::query_as!(
                Self,
                r#"
INSERT INTO poll_answers (poll_tg_id, selected_value)
VALUES ($1, $2)
RETURNING poll_tg_id, selected_value
                "#,
                tg_poll.id,
                choice as i32,
            )
            .fetch_one(&mut *txn)
            .await?;
        }

        Poll::get_by_tg_id(txn, &tg_poll.id).await
    }
}
