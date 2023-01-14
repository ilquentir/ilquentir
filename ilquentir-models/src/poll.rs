use std::fmt::Debug;

use color_eyre::{eyre::eyre, Result};
use sqlx::FromRow;
use time::OffsetDateTime;
use tracing::{debug, error, info, warn};

use teloxide::types::{MediaKind, Message, MessageKind};

use crate::{PgTransaction, PollKind, User};

#[derive(Debug, Clone, FromRow)]
pub struct Poll {
    pub id: Option<i64>,
    pub tg_id: Option<String>,
    pub tg_message_id: Option<i32>,
    pub chat_tg_id: i64,
    pub kind: PollKind,
    pub publication_date: time::OffsetDateTime,
    pub published: bool,
}

impl Poll {
    #[tracing::instrument(skip(txn), err)]
    pub async fn create(
        txn: &mut PgTransaction<'_>,
        user_tg_id: i64,
        kind: PollKind,
        publication_date: Option<OffsetDateTime>,
    ) -> Result<Self> {
        Self {
            id: None,
            tg_id: None,
            tg_message_id: None,
            chat_tg_id: user_tg_id,
            kind,
            publication_date: publication_date.unwrap_or_else(OffsetDateTime::now_utc),
            published: false,
        }
        .insert(txn)
        .await
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn create_for_user(txn: &mut PgTransaction<'_>, user: &'_ User) -> Result<Vec<Self>> {
        let publication_date = time::OffsetDateTime::now_utc();

        let polls = user
            .subscribed_for_polls()
            .into_iter()
            .map(|kind| Self {
                id: None,
                tg_id: None,
                tg_message_id: None,
                chat_tg_id: user.tg_id,
                kind,
                publication_date,
                published: false,
            })
            .collect::<Vec<_>>();

        let mut inserted = Vec::with_capacity(polls.len());

        for poll in polls {
            inserted.push(poll.insert(&mut *txn).await?);
        }

        Ok(inserted)
    }

    #[tracing::instrument(skip(txn), err)]
    async fn insert(self, txn: &mut PgTransaction<'_>) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"
INSERT INTO polls (
    chat_tg_id,
    tg_id,
    tg_message_id,
    kind,
    publication_date,
    published
)
VALUES ($1, $2, $3, $4, $5, $6)
RETURNING
    id as "id?",
    tg_id,
    tg_message_id,
    chat_tg_id,
    kind as "kind: PollKind",
    publication_date,
    published
"#,
            self.chat_tg_id,
            self.tg_id,
            self.tg_message_id,
            self.kind.to_string(),
            self.publication_date,
            self.published,
        )
        .fetch_one(txn)
        .await?)
    }

    #[tracing::instrument(skip(txn), err, ret)]
    async fn exists_next(&self, txn: &mut PgTransaction<'_>) -> Result<bool> {
        Ok(sqlx::query!(
            r#"
SELECT
    id
FROM
    polls
WHERE
    NOT published
    AND kind = $1
    AND publication_date > $2
    AND chat_tg_id = $3
            "#,
            self.kind.to_string(),
            self.publication_date,
            self.chat_tg_id,
        )
        .fetch_optional(txn)
        .await?
        .is_some())
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn schedule_next(self, txn: &mut PgTransaction<'_>) -> Result<Self> {
        if self.exists_next(txn).await? {
            warn!("trying to schedule next poll while there is already one scheduled");

            return Ok(self);
        }

        // try get custom sending time
        let next_at = self
            .kind
            .schedule_next_custom(txn, self.chat_tg_id, self.publication_date)
            .await?
            // fallback to default behaviour
            .unwrap_or_else(|| self.kind.schedule_next(self.publication_date));

        let poll = Self {
            id: None,
            publication_date: next_at,
            published: false,
            tg_id: None,
            tg_message_id: None,
            chat_tg_id: self.chat_tg_id,
            kind: self.kind,
        };

        poll.insert(txn).await
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn update(self, txn: &mut PgTransaction<'_>) -> Result<Option<Self>> {
        let id = if let Some(id) = self.id {
            id
        } else {
            error!("trying to update unsaved poll");
            return Ok(None);
        };

        Ok(Some(
            sqlx::query_as!(
                Self,
                r#"
UPDATE polls
SET
    tg_id = $2,
    tg_message_id = $3,
    chat_tg_id = $4,
    kind = $5,
    publication_date = $6,
    published = $7
WHERE id = $1
RETURNING
    id as "id?",
    tg_id,
    tg_message_id,
    chat_tg_id,
    kind as "kind: PollKind",
    publication_date,
    published
            "#,
                id,
                self.tg_id,
                self.tg_message_id,
                self.chat_tg_id,
                self.kind.to_string(),
                self.publication_date,
                self.published,
            )
            .fetch_one(txn)
            .await?,
        ))
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn get_by_tg_id<'t, S: AsRef<str> + Debug>(
        txn: &mut PgTransaction<'_>,
        tg_id: S,
    ) -> Result<Poll> {
        Ok(sqlx::query_as!(
            Poll,
            r#"
SELECT
    id as "id?",
    tg_id,
    tg_message_id,
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
        .fetch_one(txn)
        .await?)
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn get_pending(txn: &mut PgTransaction<'_>) -> Result<Vec<Self>> {
        let mut polls = sqlx::query_as!(
            Self,
            r#"
SELECT
    id as "id?",
    polls.tg_id as tg_id,
    tg_message_id,
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
        .fetch_all(txn)
        .await?;
        polls.sort_unstable_by_key(|poll| (poll.chat_tg_id, poll.kind));

        Ok(polls)
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn get_scheduled_for_user(
        txn: &mut PgTransaction<'_>,
        user_tg_id: i64,
        kind: PollKind,
    ) -> Result<Vec<Self>> {
        Ok(sqlx::query_as!(
            Self,
            r#"
SELECT
    id as "id?",
    polls.tg_id as tg_id,
    tg_message_id,
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
    AND polls.publication_date > NOW()
    AND users.active
    AND users.tg_id = $1
    AND polls.kind = $2
ORDER BY
    polls.chat_tg_id
            "#,
            user_tg_id,
            kind.to_string(),
        )
        .fetch_all(txn)
        .await?)
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn set_overdue(self, txn: &mut PgTransaction<'_>) -> Result<()> {
        let updated = sqlx::query!(
            r#"
UPDATE polls
SET
    overdue = True
WHERE
    id = $1
            "#,
            self.id
        )
        .execute(txn)
        .await?
        .rows_affected();

        if updated == 0 {
            warn!("no polls updated");
        }

        Ok(())
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn get_overdue(txn: &mut PgTransaction<'_>, kind: PollKind) -> Result<Vec<Self>> {
        let pg_interval: sqlx::postgres::types::PgInterval = kind
            .overdue_interval()
            .try_into()
            .map_err(|e| eyre!("{e}"))?;

        Ok(sqlx::query_as!(
            Self,
            r#"
SELECT
    polls.id as "id?",
    tg_id,
    tg_message_id,
    chat_tg_id,
    kind as "kind: PollKind",
    publication_date,
    published
FROM
    polls
LEFT JOIN
    poll_answers
ON
    polls.tg_id = poll_answers.poll_tg_id
WHERE
    NOT polls.overdue
    AND polls.published
    AND polls.kind = $1
    AND polls.publication_date < (NOW() - $2::interval)
    AND poll_answers.id IS NULL
            "#,
            kind.to_string(),
            pg_interval
        )
        .fetch_all(txn)
        .await?)
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn disable_pending_for_user(
        txn: &mut PgTransaction<'_>,
        user_tg_id: i64,
        kind: PollKind,
    ) -> Result<u64> {
        Ok(sqlx::query!(
            r#"
DELETE FROM polls
WHERE
    NOT published
    AND publication_date > NOW()
    AND chat_tg_id = $1
    AND kind = $2
            "#,
            user_tg_id,
            kind.to_string(),
        )
        .execute(txn)
        .await?
        .rows_affected())
    }

    #[tracing::instrument(skip(txn), err)]
    pub async fn published_to_tg(
        self,
        txn: &mut PgTransaction<'_>,
        poll_messages: &[Message],
    ) -> Result<Self> {
        // schedule new poll
        let prev_id = self.id;
        let mut poll = self;

        for message in poll_messages {
            // FIXME: do not clone here
            if let MessageKind::Common(message_common) = message.kind.clone() {
                if let MediaKind::Poll(tg_poll) = message_common.media_kind {
                    debug!(
                        poll_id = prev_id,
                        user_tg_id = poll.chat_tg_id,
                        poll_tg_id = tg_poll.poll.id,
                        "poll sent"
                    );

                    // TODO: think about refactoring in two methods
                    // save that poll is published
                    poll.published = true;
                    poll.tg_message_id = Some(message.id.0);
                    poll.tg_id = Some(tg_poll.poll.id.clone());
                    if poll.id.is_some() {
                        poll = poll.update(&mut *txn).await?.expect("post update failed");
                    } else {
                        poll = poll.insert(&mut *txn).await?;
                    }
                    poll.id = None;

                    debug!(
                        poll_id = poll.id,
                        user_tg_id = poll.chat_tg_id,
                        poll_tg_id = tg_poll.poll.id,
                        "saved that poll is published"
                    );
                }
            } else {
                error!(
                    ?message,
                    "got some weird message in response to SendPoll request"
                );
                return Err(color_eyre::eyre::eyre!(
                    "got some weird message in response to SendPoll request"
                ));
            }
        }

        let next_poll = poll.schedule_next(&mut *txn).await?;
        info!(
            poll_id = prev_id,
            next_poll_id = next_poll.id,
            user_tg_id = next_poll.chat_tg_id,
            "scheduled new poll"
        );

        Ok(next_poll)
    }
}
