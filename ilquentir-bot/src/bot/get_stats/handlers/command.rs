use color_eyre::Result;
use teloxide::{requests::Requester, types::ChatId};

use ilquentir_messages::md_message;
use ilquentir_models::PgTransaction;
use ilquentir_python_graph::Plotter;

use crate::bot::{helpers::set_typing, Bot};

#[tracing::instrument(skip_all, fields(chat_id=chat_id.0), err)]
pub async fn handle_get_stats_command(
    bot: &Bot,
    plotter: &Plotter,
    txn: &mut PgTransaction<'_>,
    chat_id: ChatId,
) -> Result<()> {
    set_typing(bot, chat_id, None).await?;

    let graph_url = plotter.create_plot(txn, chat_id.0).await?;

    let message = md_message!("stats/get_stat.md", graph_url = graph_url);
    bot.send_message(chat_id, message).await?;

    Ok(())
}
