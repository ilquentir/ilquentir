use std::time::Duration;

use color_eyre::Result;
use sqlx::PgPool;
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{InputFile, Update, UpdateKind},
};
use tracing::{info, warn};

use ilquentir_config::Config;
use ilquentir_giphy::GiphyApi;
use ilquentir_graphs::{daily::chart_daily_stats, weekly::personal_weekly_stat};
use ilquentir_messages::{md, md_message};
use ilquentir_models::{PollAnswer, PollKind, PollStat, PollWeeklyUserStat, User};

use crate::bot::{helpers::set_typing, Bot};

#[tracing::instrument(skip(bot, pool, _giphy, config), err)]
pub async fn handle_poll_update(
    bot: Bot,
    pool: PgPool,
    _giphy: GiphyApi,
    config: Config,
    update: Update,
) -> Result<()> {
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

        // let's finish the txn before lots of communication with external APIs
        txn.commit().await?;

        let chat_id = poll.chat_tg_id;
        set_typing(&bot, chat_id, Some(Duration::from_millis(200))).await?;

        if User::count_answered_polls(&mut pool.begin().await?, poll.chat_tg_id).await? == 1 {
            // first user poll, send detailed info
            let message = bot.send_photo(
                chat_id.to_string(),
                InputFile::url(
                    "https://utterstep-public.fra1.digitaloceanspaces.com/tg_image_2763456418.jpeg"
                        .parse()?,
                ),
            ).await?;
            set_typing(&bot, chat_id, Some(Duration::from_millis(1000))).await?;

            let message_text = md_message!("voted_poll_reaction/first_time.md");

            bot.send_message(chat_id.to_string(), message_text)
                .reply_to_message_id(message.id)
                .await?;
        } else {
            // send generic response
            // let cat_gif = tokio::time::timeout(
            //     Duration::from_secs(2),
            //     giphy.get_random_cat_gif()
            // ).await.unwrap_or_else(|_| {
            //     warn!("timeout while requesting GIPHY api");

            //     Ok("https://media0.giphy.com/media/X3Yj4XXXieKYM/giphy-loop.mp4?cid=fd4c87ca9b02f849d4548fc9530a2dbe6e058599dc2630af&rid=giphy-loop.mp4&ct=g".parse().unwrap())
            // })?;

            // info!(user_id, chat_id, "sending cat gif");
            // bot.send_animation(chat_id.to_string(), InputFile::url(cat_gif))
            //     .await?;

            info!(user_id, chat_id, "sending message");
            let message_text = match poll.kind {
                PollKind::HowWasYourDay => {
                    let bot_clone = bot.clone();
                    let pool_clone = pool.clone();
                    tokio::spawn(async move {
                        let delay = poll.publication_date + config.min_reply_delay
                            - time::OffsetDateTime::now_utc();
                        if delay.is_positive() {
                            tokio::time::sleep(delay.unsigned_abs()).await;
                        } else {
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }

                        let stats = PollStat::get_today_stats(
                            &mut pool_clone.begin().await.unwrap(),
                            poll.kind,
                        )
                        .await
                        .unwrap();
                        let graph = chart_daily_stats(&stats).unwrap();

                        let message_text =
                            md_message!("voted_poll_reaction/stats_for_today.md", graph = graph);

                        bot_clone
                            .send_message(chat_id.to_string(), message_text)
                            .await
                            .unwrap();
                    });

                    let your_stat = PollWeeklyUserStat::get_for_last_week(
                        &mut pool.begin().await?,
                        PollKind::HowWasYourDay,
                        chat_id,
                    )
                    .await?;
                    let your_stat_descr = personal_weekly_stat(&your_stat);

                    md_message!(
                        "voted_poll_reaction/generic_response.md",
                        your_stat_descr = your_stat_descr,
                    )
                }
                PollKind::FoodAllergy => md!("Meow :)"),
            };

            bot.send_message(chat_id.to_string(), message_text).await?;
        }

        info!(
            user_id,
            chat_id,
            poll_id = poll.tg_id,
            "reply sent, commiting txn"
        );
    } else {
        warn!(user_id, chat_id, update = ?update, "unexpected update type");
    }

    Ok(())
}
