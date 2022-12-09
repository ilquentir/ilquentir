use std::fmt::Debug;

use color_eyre::Result;
use sqlx::{FromRow, Postgres, Transaction};
use teloxide::{
    payloads::{SendPoll, SendPollSetters},
    requests::JsonRequest,
    types::{MediaKind, MessageKind, Poll as TgPoll},
    Bot,
};
use tracing::{debug, error, info};

use crate::poll::PollKind;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub tg_id: i64,
    pub active: bool,
}

type PgTransaction<'t> = Transaction<'t, Postgres>;

impl User {
    #[tracing::instrument(skip(transaction), err)]
    pub async fn get_user_by_id<'t>(
        transaction: &mut PgTransaction<'t>,
        user_id: i64,
    ) -> Result<Option<Self>> {
        Ok(sqlx::query_as!(
            Self,
            r#"
SELECT tg_id, active
FROM users
WHERE
    tg_id = $1
    AND active
            "#,
            user_id,
        )
        .fetch_optional(&mut *transaction)
        .await?)
    }

    #[tracing::instrument(skip(transaction), err)]
    pub async fn activate<'t>(transaction: &mut PgTransaction<'t>, user_id: i64) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
INSERT INTO users (tg_id, active)
VALUES ($1, true)
ON CONFLICT (tg_id) DO UPDATE SET active = true
RETURNING tg_id, active
            "#,
            user_id,
        )
        .fetch_one(transaction)
        .await?)
    }

    #[tracing::instrument(skip(transaction), err)]
    pub async fn deactivate<'t>(transaction: &mut PgTransaction<'t>, user_id: i64) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
INSERT INTO users (tg_id, active)
VALUES ($1, false)
ON CONFLICT (tg_id) DO UPDATE SET active = false
RETURNING tg_id, active
            "#,
            user_id,
        )
        .fetch_one(transaction)
        .await?)
    }

    #[tracing::instrument(skip(transaction), err)]
    pub async fn count_answered_polls<'t>(
        transaction: &mut PgTransaction<'t>,
        user_id: i64,
    ) -> Result<i64> {
        Ok(sqlx::query!(
            r#"
SELECT COUNT(DISTINCT poll.tg_id) as "n_answered!"
FROM polls AS poll
JOIN poll_answers AS answer
ON poll.tg_id = answer.poll_tg_id
WHERE poll.chat_tg_id = $1
            "#,
            user_id,
        )
        .fetch_one(transaction)
        .await?
        .n_answered)
    }

    #[tracing::instrument(skip(transaction), err)]
    pub async fn get_active<'t>(transaction: &mut PgTransaction<'t>) -> Result<Vec<Self>> {
        Ok(sqlx::query_as!(
            Self,
            r#"
SELECT tg_id, active
FROM users
WHERE active
            "#
        )
        .fetch_all(transaction)
        .await?)
    }

    #[tracing::instrument]
    pub fn subscribed_for_polls(&self) -> Vec<PollKind> {
        vec![PollKind::HowWasYourDay]
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Poll {
    pub id: Option<i64>,
    pub tg_id: Option<String>,
    pub chat_tg_id: i64,
    pub kind: PollKind,
    pub publication_date: time::OffsetDateTime,
    pub published: bool,
}

impl Poll {
    #[tracing::instrument(err)]
    pub async fn create_for_user(
        transaction: &mut Transaction<'_, Postgres>,
        user: &User,
    ) -> Result<Vec<Self>> {
        let publication_date = time::OffsetDateTime::now_utc();

        let polls = user
            .subscribed_for_polls()
            .into_iter()
            .map(|kind| Poll {
                id: None,
                tg_id: None,
                chat_tg_id: user.tg_id,
                kind,
                publication_date,
                published: false,
            })
            .collect::<Vec<_>>();

        let mut inserted = Vec::with_capacity(polls.len());

        for poll in polls {
            inserted.push(poll.insert(&mut *transaction).await?);
        }

        Ok(inserted)
    }

    #[tracing::instrument(skip(transaction), err)]
    async fn insert<'t>(self, transaction: &mut PgTransaction<'t>) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
INSERT INTO polls (
    chat_tg_id,
    kind,
    publication_date,
    published
)
VALUES ($1, $2, $3, $4)
RETURNING
    id as "id?",
    tg_id,
    chat_tg_id,
    kind as "kind: PollKind",
    publication_date,
    published
"#,
            self.chat_tg_id,
            self.kind.to_string(),
            self.publication_date,
            self.published,
        )
        .fetch_one(transaction)
        .await?)
    }

    #[tracing::instrument(skip(transaction), err)]
    pub async fn schedule_next<'t>(self, transaction: &mut PgTransaction<'t>) -> Result<Self> {
        let next_at = self.kind.schedule_next(self.publication_date);
        let poll = Self {
            publication_date: next_at,
            published: false,
            ..self
        };

        poll.insert(transaction).await
    }

    #[tracing::instrument(skip(transaction), err)]
    pub async fn update<'t>(self, transaction: &mut PgTransaction<'t>) -> Result<Option<Self>> {
        let id = if let Some(id) = self.id {
            id
        } else {
            error!("trying to update unsaved poll");
            return Ok(None);
        };

        dbg!(&self);

        Ok(Some(
            sqlx::query_as!(
                Self,
                r#"
UPDATE polls
SET
    tg_id = $2,
    chat_tg_id = $3,
    kind = $4,
    publication_date = $5,
    published = $6
WHERE id = $1
RETURNING
    id as "id?",
    tg_id,
    chat_tg_id,
    kind as "kind: PollKind",
    publication_date,
    published
            "#,
                id,
                self.tg_id,
                self.chat_tg_id,
                self.kind.to_string(),
                self.publication_date,
                self.published,
            )
            .fetch_one(transaction)
            .await?,
        ))
    }

    #[tracing::instrument(skip(transaction), err)]
    pub async fn get_by_tg_id<'t, S: AsRef<str> + Debug>(
        transaction: &mut PgTransaction<'t>,
        tg_id: S,
    ) -> Result<Poll> {
        Ok(sqlx::query_as!(
            Poll,
            r#"
SELECT
    id as "id?",
    tg_id,
    chat_tg_id,
    kind as "kind: PollKind",
    publication_date,
    published
FROM polls
WHERE
    tg_id = $1
            "#,
            tg_id.as_ref(),
        )
        .fetch_one(transaction)
        .await?)
    }

    #[tracing::instrument(skip(transaction), err)]
    pub async fn get_pending<'t>(transaction: &mut PgTransaction<'t>) -> Result<Vec<Self>> {
        Ok(sqlx::query_as!(
            Self,
            r#"
SELECT
    id as "id?",
    polls.tg_id as tg_id,
    chat_tg_id,
    kind as "kind: PollKind",
    publication_date,
    published
FROM polls
JOIN users
ON
    polls.chat_tg_id = users.tg_id
WHERE
    NOT polls.published
    AND polls.publication_date < NOW()
    AND users.active
            "#
        )
        .fetch_all(transaction)
        .await?)
    }

    #[tracing::instrument(skip(bot, transaction), err)]
    pub async fn publish_to_tg<'t>(
        mut self,
        transaction: &mut PgTransaction<'t>,
        bot: &Bot,
    ) -> Result<Self> {
        info!(poll_id = self.id, "sending poll");
        let poll_options = self.kind.options();

        let poll_payload = SendPoll::new(
            self.chat_tg_id.to_string(),
            self.kind.question(),
            poll_options,
        )
        .allows_multiple_answers(self.kind.allows_multiple_answers());
        let poll_message = JsonRequest::new(bot.clone(), poll_payload).await?;

        info!(poll_id = self.id, "poll sent");

        // FIXME: do not clone here
        if let MessageKind::Common(message) = poll_message.kind.clone() {
            if let MediaKind::Poll(tg_poll) = message.media_kind {
                debug!(
                    poll_id = self.id,
                    user_id = self.chat_tg_id,
                    poll_tg_id = tg_poll.poll.id,
                    "poll sent"
                );

                // TODO: think about refactoring in two methods
                // save that poll is published
                self.published = true;
                self.tg_id = Some(tg_poll.poll.id.clone());
                let poll = self
                    .update(&mut *transaction)
                    .await?
                    .expect("post update failed");
                debug!(
                    poll_id = poll.id,
                    user_id = poll.chat_tg_id,
                    poll_tg_id = tg_poll.poll.id,
                    "saved that poll is published"
                );

                // schedule new poll
                let prev_id = poll.id;
                let next_poll = poll.schedule_next(&mut *transaction).await?;
                info!(
                    poll_id = prev_id,
                    next_poll_id = next_poll.id,
                    user_id = next_poll.chat_tg_id,
                    "scheduled new poll"
                );

                return Ok(next_poll);
            }
        }

        error!(
            ?poll_message,
            "got some weird message in response to SendPoll request"
        );
        Err(color_eyre::eyre::eyre!(
            "got some weird message in response to SendPoll request"
        ))
    }

    #[tracing::instrument(skip(transaction), err)]
    pub async fn get_poll_stats<'t>(
        transaction: &mut PgTransaction<'t>,
        kind: PollKind,
    ) -> Result<Vec<PollStat>> {
        Ok(sqlx::query_as!(
            PollStat,
            r#"
SELECT
    selected_value,
    COUNT(poll_answers.id) as "n_selected!"
FROM poll_answers
JOIN polls
ON poll_answers.poll_tg_id = polls.tg_id
WHERE
    polls.kind = $1
    AND polls.publication_date BETWEEN NOW() - interval '24 hours' AND NOW()
GROUP BY selected_value
            "#,
            kind.to_string()
        )
        .fetch_all(transaction)
        .await?)
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct PollStat {
    pub selected_value: i32,
    pub n_selected: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct PollAnswer {
    pub poll_tg_id: String,
    pub selected_value: i32,
}

impl PollAnswer {
    #[tracing::instrument(skip(transaction), err)]
    pub async fn save_answer<'t>(
        transaction: &mut PgTransaction<'t>,
        tg_poll: &TgPoll,
    ) -> Result<Poll> {
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
            .fetch_one(&mut *transaction)
            .await?;
        }

        Poll::get_by_tg_id(transaction, &tg_poll.id).await
    }
}
