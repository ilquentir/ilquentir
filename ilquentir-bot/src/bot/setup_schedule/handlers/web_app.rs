use color_eyre::{eyre::eyre, Result};
use sqlx::PgPool;
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{Message, MessageWebAppData, ReplyMarkup},
};
use time::{ext::NumericalDuration, macros::format_description, OffsetDateTime, UtcOffset};

use ilquentir_messages::md_message;
use ilquentir_models::{Poll, PollKind, PollSettings};

use crate::bot::Bot;

pub async fn handle_webapp(
    bot: Bot,
    pool: PgPool,
    msg: Message,
    data: MessageWebAppData,
) -> Result<()> {
    let (timestamp, offset) = data
        .web_app_data
        .data
        .split_once('_')
        .ok_or_else(|| eyre!("unknown WebApp data format"))?;

    let offset: i32 = -(offset.parse()?);
    let offset = UtcOffset::from_whole_seconds(offset * 60)?;

    let timestamp_nanos = timestamp.parse::<i128>()? * 1000000;
    let send_at_datetime = OffsetDateTime::from_unix_timestamp_nanos(timestamp_nanos)?;
    let send_at_utc = send_at_datetime.time();

    let mut txn = pool.begin().await?;

    for kind in [PollKind::DailyEvents, PollKind::HowWasYourDay] {
        PollSettings::set_send_at(&mut txn, msg.chat.id.0, kind, send_at_utc).await?;

        for mut poll in Poll::get_scheduled_for_user(&mut txn, msg.chat.id.0, kind).await? {
            let new_publication_date = {
                let new_publication_date = poll.publication_date.replace_time(send_at_utc);

                if new_publication_date < OffsetDateTime::now_utc() {
                    new_publication_date + 1.days()
                } else {
                    new_publication_date
                }
            };

            poll.publication_date = new_publication_date;
            poll.update(&mut txn).await?;
        }
    }

    txn.commit().await?;

    bot.send_message(
        msg.chat.id,
        md_message!(
            "settings/setup_schedule_done.md",
            time = send_at_datetime
                .to_offset(offset)
                .time()
                .format(format_description!("[hour]:[minute]"))?,
            offset = offset.format(format_description!(
                "[offset_hour sign:mandatory]:[offset_minute]"
            ))?,
        ),
    )
    .reply_markup(ReplyMarkup::kb_remove())
    .await?;

    Ok(())
}
