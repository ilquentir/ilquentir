use std::time::Duration;

use color_eyre::Result;
use ilquentir_giphy::GiphyApi;
use sqlx::PgPool;
use teloxide::{
    payloads::{SendMessage, SendMessageSetters, SendPhoto, SendAnimation},
    requests::{JsonRequest, Requester},
    types::{InputFile, Update, UpdateKind},
    Bot,
};

use tracing::{info, warn};

use crate::{
    bot::helpers::set_typing,
    models::{Poll, PollAnswer, User},
};

#[tracing::instrument(skip(bot, pool, giphy), err)]
pub async fn handle_poll_update(bot: Bot, pool: PgPool, giphy: GiphyApi, update: Update) -> Result<()> {
    let user_id = update.user().map(|u| u.id.0);
    let chat_id = update.chat().map(|c| c.id.0);

    info!(user_id, chat_id, "start processing update");

    if let UpdateKind::Poll(tg_poll) = update.kind {
        info!(
            user_id,
            chat_id,
            poll_id = tg_poll.id,
            "got Poll update, saving data"
        );
        let mut txn = pool.begin().await?;

        let poll = PollAnswer::save_answer(&mut txn, &tg_poll).await?;
        info!(
            user_id,
            chat_id,
            poll_id = poll.tg_id,
            "data saved, sending the reply"
        );

        let chat_id = poll.chat_tg_id;
        set_typing(&bot, chat_id, Some(Duration::from_millis(200))).await?;

        if User::count_answered_polls(&mut txn, poll.chat_tg_id).await? == 1 {
            // first user poll, send detailed info
            let photo_payload = SendPhoto::new(
                chat_id.to_string(),
                InputFile::url(
                    "https://utterstep-public.fra1.digitaloceanspaces.com/tg_image_2763456418.jpeg"
                        .parse()?,
                ),
            );
            let message = JsonRequest::new(bot.clone(), photo_payload).await?;
            set_typing(&bot, chat_id, Some(Duration::from_millis(1000))).await?;

            let message_payload = SendMessage::new(
                chat_id.to_string(),
                r#"Thanks! This is an example of a graph you will be getting.
Here you can see how ~500 Russian IT specialists answered the same question in 2022.

Keep answering to see your personal dynamics per se and in comparison to the community."#,
            )
            .reply_to_message_id(message.id);
            JsonRequest::new(bot.clone(), message_payload).await?;
        } else {
            // send generic response
            let cat_gif = giphy.get_random_cat_gif().await?;

            info!(user_id, chat_id, "sending cat gif");
            let photo_payload = SendAnimation::new(
                chat_id.to_string(),
                InputFile::url(cat_gif),
            );
            JsonRequest::new(bot.clone(), photo_payload).await?;

            info!(user_id, chat_id, "sending message");
            bot.send_message(
                chat_id.to_string(),
                r#"Thanks! When we'll be ready with stats – you'll know!
In case of having any questions – don't hesitate to write @utterstep :)"#
            ).await?;
        }

        info!(
            user_id,
            chat_id,
            poll_id = poll.tg_id,
            "reply sent, commiting txn"
        );

        txn.commit().await?;
    } else {
        warn!(user_id, chat_id, update = ?update, "unexpected update type");
    }

    Ok(())
}

#[tracing::instrument(skip(bot, pool), err)]
pub async fn handle_scheduled(bot: &Bot, pool: &PgPool) -> Result<()> {
    let mut txn = pool.begin().await?;

    info!("checking if there exist some unsent polls");
    let polls = Poll::get_pending(&mut txn).await?;
    if polls.is_empty() {
        info!("no polls to send");
    } else {
        info!(unsent_poll_count = polls.len(), "found some unsent polls");
    }

    // FIXME: do not require transaction usage in Poll::get_pending
    txn.commit().await?;

    for poll in polls {
        info!(
            poll_id = poll.id,
            user_id = poll.chat_tg_id,
            "sending scheduled poll"
        );

        let mut txn = pool.begin().await?;

        poll.publish_to_tg(&mut txn, bot).await?;

        txn.commit().await?;
    }

    Ok(())
}
