use color_eyre::{Report, Result};
use sqlx::PgPool;
use teloxide::{
    adaptors::{trace::Settings, DefaultParseMode, Trace},
    dispatching::{DefaultKey, HandlerExt, UpdateFilterExt},
    prelude::Dispatcher as TgDispatcher,
    requests::{Requester, RequesterExt},
    types::{ParseMode, Update, UpdateKind},
    utils::command::BotCommands,
    Bot as TgBot,
};

use ilquentir_config::Config;
use ilquentir_giphy::GiphyApi;

pub mod commands;
pub mod handlers;
pub mod helpers;

use self::{
    commands::Command,
    handlers::{handle_command, handle_poll_update},
};

pub type Bot = Trace<DefaultParseMode<TgBot>>;
pub type Dispatcher<'a> = TgDispatcher<Bot, Report, DefaultKey>;

pub async fn create_bot() -> Result<Bot> {
    let bot = TgBot::from_env()
        .parse_mode(ParseMode::MarkdownV2)
        .trace(Settings::TRACE_EVERYTHING);
    bot.set_my_commands(commands::Command::bot_commands())
        .await?;

    Ok(bot)
}

pub async fn create_bot_and_dispatcher(
    pool: PgPool,
    giphy: GiphyApi,
    config: &Config,
) -> Result<(Dispatcher, Bot)> {
    let bot = create_bot().await?;

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
        TgDispatcher::builder(bot.clone(), handler)
            .dependencies(dptree::deps![pool, giphy, config.clone()])
            .enable_ctrlc_handler()
            .build(),
        bot,
    ))
}
