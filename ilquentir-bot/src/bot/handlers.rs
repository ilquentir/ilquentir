use color_eyre::Result;
use sqlx::PgPool;
use teloxide::{
    requests::Requester,
    types::{Message, Update, UpdateKind},
    Bot,
};

use tracing::{info, warn};

use super::commands::Command;
use crate::models::{Poll, PollAnswer, User};

#[tracing::instrument(skip(bot, pool))]
pub async fn handle_command(bot: Bot, pool: PgPool, msg: Message, command: Command) -> Result<()> {
    let mut txn = pool.begin().await?;

    match command {
        Command::Start => {
            info!(chat_id = %msg.chat.id, "processing Start command");

            let user = User::activate(&mut txn, msg.chat.id.0).await?;
            info!(chat_id = %msg.chat.id, user_id = user.tg_id, "(re?) activated user");

            Poll::create_for_user(&mut txn, &user).await?;

            bot.send_message(msg.chat.id, "Hello! Ilquentir welcomes you.")
                .await?;
        }
        Command::Stop => {
            info!(chat_id = %msg.chat.id, "processing Stop command");

            let user = User::deactivate(&mut txn, msg.chat.id.0).await?;

            info!(chat_id = %msg.chat.id, user_id = user.tg_id, "disabled user");

            bot.send_message(
                msg.chat.id,
                "Deactivation succeeded. Hope you'll be back soon!",
            )
            .await?;
        }
        Command::GetStat => {
            bot.send_message(msg.chat.id, "get_stat is currently unimplemented :(")
                .await?;
        }
    }
    txn.commit().await?;

    Ok(())
}

#[tracing::instrument(skip(bot, pool))]
pub async fn handle_poll_update(bot: Bot, pool: PgPool, update: Update) -> Result<()> {
    let user_id = update.user().map(|u| u.id.0);
    let chat_id = update.chat().map(|c| c.id.0);

    info!(user_id, chat_id, "start processing update");

    if let UpdateKind::Poll(poll) = update.kind {
        let mut txn = pool.begin().await?;

        let poll = PollAnswer::save_answer(&mut txn, &poll).await?;
        bot.send_message(
            poll.chat_tg_id.to_string(),
            "Thanks! When we'll be ready with stats – you'll know!\n\nIn case of having any questions – write @utterstep"
        ).await?;

        txn.commit().await?;
    } else {
        warn!(user_id, chat_id, update = ?update, "unexpected update type");
    }

    Ok(())
}

#[tracing::instrument(skip(bot, pool))]
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
            poll = poll.id,
            user = poll.chat_tg_id,
            "sending scheduled poll"
        );
        let mut txn = pool.begin().await?;

        poll.publish_to_tg(&mut txn, bot).await?;

        txn.commit().await?;
    }

    Ok(())
}
