use std::time::Duration;

use color_eyre::Result;
use sqlx::PgPool;
use teloxide::{
    payloads::{SendAnimation, SendMessage, SendMessageSetters, SendPhoto},
    requests::JsonRequest,
    types::{InputFile, Update, UpdateKind},
    Bot,
};
use tracing::{error, info, warn};

use ilquentir_config::Config;
use ilquentir_giphy::GiphyApi;
use ilquentir_graphs::{daily::chart_daily_stats, weekly::personal_weekly_stat};
use ilquentir_models::{PollAnswer, PollKind, PollStat, PollWeeklyUserStat, User};

use crate::bot::helpers::set_typing;

#[tracing::instrument(skip(bot, pool, giphy, config), err)]
pub async fn handle_poll_update(
    bot: Bot,
    pool: PgPool,
    giphy: GiphyApi,
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
                r#"Спасибо! Вот пример трендов, которые можно замечать:
На этом графике ты видишь результаты опроса ~500 русскоязычных околоайшинкиков, которые отвечали на этот же вопрос в 2022 году.

Отвечая, ты будешь видеть свою статистику, общую статистику и получишь возможность сравниться.
Статистика будет приходить после твоего ответа, если ты хочешь получить её в любой другой момент – попроси меня командой `/get_stat`, буду рад помочь :)"#,
            )
            .reply_to_message_id(message.id);
            JsonRequest::new(bot.clone(), message_payload).await?;
        } else {
            // send generic response
            let cat_gif = tokio::time::timeout(
                Duration::from_secs(2),
                giphy.get_random_cat_gif()
            ).await.unwrap_or_else(|elapsed| {
                error!(?elapsed, "timeout while requesting GIPHY api");

                Ok("https://media0.giphy.com/media/X3Yj4XXXieKYM/giphy-loop.mp4?cid=fd4c87ca9b02f849d4548fc9530a2dbe6e058599dc2630af&rid=giphy-loop.mp4&ct=g".parse().unwrap())
            })?;

            info!(user_id, chat_id, "sending cat gif");
            let photo_payload = SendAnimation::new(chat_id.to_string(), InputFile::url(cat_gif));
            JsonRequest::new(bot.clone(), photo_payload).await?;

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

                        let message_payload = SendMessage::new(
                            chat_id.to_string(),
                            format!(
                                r#"И – статистика за сегодня :\)

```
  %
{graph}
```"#
                            ),
                        )
                        .parse_mode(teloxide::types::ParseMode::MarkdownV2);

                        JsonRequest::new(bot_clone, message_payload).await.unwrap();
                    });

                    let your_stat = PollWeeklyUserStat::get_for_last_week(
                        &mut pool.begin().await?,
                        poll.kind,
                        poll.chat_tg_id,
                    )
                    .await?;
                    let your_stat_descr = personal_weekly_stat(&your_stat);

                    format!(
                        r#"Привет, поймали ответ\! Послушали вас и решили, что раз бота делаем в около российском контексте – не выпендриваться и перейти на русский\. А дальше будем развивать\.

Прошла первая тестовая неделя, спасибо, что поверили и помогаете ^^

Делимся [общим](https://utterstep-public.fra1.digitaloceanspaces.com/first_week.png) графиком оценок настроения и твоими ответов в течение недели\. На общем заметно, что во вторник у многих был эмоциональный подъем, а четверг похоже – трудный день\. Но пятница – спасение\! \(Напомним, что нас пока немного и прожили мы только неделю, так что не стоит делать слишком серьёзных выводов из общей статы :\)\)\.

Стата по тебе:
{your_stat_descr}

Свежую, актуальную стату теперь можно запросить у бота командой `/get_stat` \:\)
Графики докуртим и подружим, будет по красоте\.\)

Хорошей недели и берегите себя\!"#
                    )
                }
                PollKind::FoodAllergy => r#"Meow :\)"#.to_owned(),
            };

            let message_payload = SendMessage::new(chat_id.to_string(), message_text)
                .parse_mode(teloxide::types::ParseMode::MarkdownV2);
            JsonRequest::new(bot.clone(), message_payload).await?;

            set_typing(&bot, chat_id, Some(Duration::from_millis(500))).await?;

            let message_payload = SendMessage::new(chat_id.to_string(), r"Кстати, ты нам сильно поможешь, если ответишь на несколько вопросов про свой опыт с ботом\. Займет минут 5\. Вот [опрос](https://forms.gle/vDrswFF49tNqiYeH6), будем рады \:\)")
                .parse_mode(teloxide::types::ParseMode::MarkdownV2);
            JsonRequest::new(bot.clone(), message_payload).await?;
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
