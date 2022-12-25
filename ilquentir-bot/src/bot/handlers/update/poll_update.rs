use color_eyre::Result;
use sqlx::PgPool;
use teloxide::{
    requests::Requester,
    types::{ChatId, Update, UpdateKind},
};
use tracing::{info, warn};

use ilquentir_config::Config;
use ilquentir_giphy::GiphyApi;
use ilquentir_messages::md;
use ilquentir_models::{PollAnswer, PollKind};

use crate::bot::{daily_events, how_was_your_day, Bot};

#[tracing::instrument(skip(bot, pool, _giphy, config), err)]
pub async fn handle_poll_update(
    bot: Bot,
    pool: PgPool,
    _giphy: GiphyApi,
    config: Config,
    update: Update,
) -> Result<()> {
    let user_id = update.user().map(|u| u.id.0);
    let chat_id = update.chat().map(|c| c.id);
    let chat_id_trace = chat_id.map(|c_id| c_id.0);

    info!(user_id, chat_id = chat_id_trace, "start processing update");

    if let UpdateKind::Poll(tg_poll) = update.kind {
        info!(
            user_id,
            chat_id = chat_id_trace,
            poll_id = tg_poll.id,
            "got Poll update, saving data"
        );
        let mut txn = pool.begin().await?;

        let poll = PollAnswer::save_answer(&mut txn, &tg_poll).await?;
        info!(
            user_id,
            chat_id = chat_id_trace,
            poll_id = poll.tg_id,
            "data saved, sending the reply"
        );

        // let's finish the txn before lots of communication with external APIs
        txn.commit().await?;

        let chat_id = ChatId(poll.chat_tg_id);

        // send generic response
        // let cat_gif = tokio::time::timeout(
        //     Duration::from_secs(2),
        //     giphy.get_random_cat_gif()
        // ).await.unwrap_or_else(|_| {
        //     warn!("timeout while requesting GIPHY api");

        //     Ok("https://media0.giphy.com/media/X3Yj4XXXieKYM/giphy-loop.mp4?cid=fd4c87ca9b02f849d4548fc9530a2dbe6e058599dc2630af&rid=giphy-loop.mp4&ct=g".parse().unwrap())
        // })?;

        // info!(user_id, chat_id = chat_id.0, "sending cat gif");
        // bot.send_animation(chat_id.to_string(), InputFile::url(cat_gif))
        //     .await?;

        info!(user_id, chat_id = chat_id.0, "sending message");
        match poll.kind {
            PollKind::HowWasYourDay => {
                how_was_your_day::poll_answered(&bot, &pool, &poll, config.clone()).await?
            }
            PollKind::FoodAllergy => {
                bot.send_message(chat_id.to_string(), md!("Meow :)"))
                    .await?;
            }
            PollKind::DailyEvents => daily_events::poll_answered(&bot, &pool, &poll).await?,
        };

        info!(
            user_id,
            chat_id = chat_id_trace,
            poll_id = poll.tg_id,
            "reply sent, commiting txn"
        );
    } else {
        warn!(user_id, chat_id = chat_id_trace, update = ?update, "unexpected update type");
    }

    Ok(())
}
