use color_eyre::Result;
use teloxide::{requests::Requester, types::ChatId};

use ilquentir_graphs::{daily::chart_daily_stats, weekly::personal_weekly_stat};
use ilquentir_messages::md_message;
use ilquentir_models::{PgTransaction, PollKind, PollStat, PollWeeklyUserStat};

use crate::bot::Bot;

#[tracing::instrument(skip(bot, txn), err)]
pub async fn handle_get_stat(
    bot: &Bot,
    txn: &mut PgTransaction<'_>,
    chat_id: ChatId,
) -> Result<()> {
    let stats = PollStat::get_today_stats(txn, PollKind::HowWasYourDay).await?;
    let graph = chart_daily_stats(&stats)?;

    let your_stat =
        PollWeeklyUserStat::get_for_last_week(txn, PollKind::HowWasYourDay, chat_id.0).await?;
    let your_stat_descr = personal_weekly_stat(&your_stat);

    let message = md_message!(
        "stats/get_stat.md",
        graph = graph,
        your_stat_descr = your_stat_descr,
    );
    bot.send_message(chat_id, message).await?;

    Ok(())
}
