use color_eyre::Result;
use teloxide::{requests::Requester, types::ChatId};

use ilquentir_config::Config;
use ilquentir_messages::md_message;
use ilquentir_models::PgTransaction;

use crate::bot::Bot;

#[tracing::instrument(skip_all, fields(chat_id=chat_id.0), err)]
pub async fn handle_get_stats_command(
    bot: &Bot,
    config: &Config,
    txn: &mut PgTransaction<'_>,
    chat_id: ChatId,
) -> Result<()> {
    let graph_html = ilquentir_python_graph::make_plotly_graph(txn, config, chat_id.0).await?;

    let message = md_message!("stats/get_stat.md", graph_html = graph_html,);
    bot.send_message(chat_id, message).await?;

    Ok(())
}
