use color_eyre::{eyre::Result, Report};
use ilquentir_giphy::GiphyApi;
use sqlx::PgPool;
use teloxide::{
    dispatching::{DefaultKey, HandlerExt, UpdateFilterExt},
    prelude::Dispatcher as TgDispatcher,
    requests::Requester,
    types::{Update, UpdateKind},
    utils::command::BotCommands,
    Bot,
};

pub mod commands;
pub mod handlers;
mod helpers;

use self::{
    commands::Command,
    handlers::{handle_command, handle_poll_update},
};

pub type Dispatcher = TgDispatcher<Bot, Report, DefaultKey>;

pub async fn create_bot_dispatcher(pool: PgPool, giphy: GiphyApi) -> Result<(Dispatcher, Bot)> {
    let bot = Bot::from_env();
    bot.set_my_commands(commands::Command::bot_commands())
        .await?;

    let handler = dptree::entry()
        .branch(
            Update::filter_message().branch(
                dptree::entry()
                    .filter_command::<Command>()
                    .endpoint(handle_command),
            ),
        )
        .branch(
            dptree::entry()
                .filter(|update: Update| matches!(update.kind, UpdateKind::Poll(_)))
                .endpoint(handle_poll_update),
        );

    Ok((
        Dispatcher::builder(bot.clone(), handler)
            .dependencies(dptree::deps![pool, giphy])
            .enable_ctrlc_handler()
            .build(),
        bot,
    ))
}
