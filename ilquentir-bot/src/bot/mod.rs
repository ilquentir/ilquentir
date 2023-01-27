use color_eyre::{Report, Result};
use sqlx::PgPool;
use teloxide::{
    adaptors::{trace::Settings, DefaultParseMode, Trace},
    dispatching::{DefaultKey, HandlerExt, UpdateFilterExt},
    prelude::Dispatcher as TgDispatcher,
    requests::{Requester, RequesterExt},
    types::{ParseMode, Update},
    utils::command::BotCommands,
    Bot as TgBot,
};

use ilquentir_config::Config;
use ilquentir_python_graph::Plotter;

pub(self) mod callbacks;
pub mod commands;
pub mod handlers;
pub mod helpers;

mod daily_events;
mod diary;
mod extractors;
mod get_stats;
mod how_was_your_day;
mod setup_schedule;

use self::{
    commands::Command,
    handlers::{handle_ban, handle_callback, handle_command, handle_poll_update},
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

pub async fn create_bot_and_dispatcher(pool: PgPool, config: &Config) -> Result<(Dispatcher, Bot)> {
    let bot = create_bot().await?;
    let plotter = Plotter::new(&mut pool.begin().await?, config.clone()).await?;

    let handler = dptree::entry()
        // generic Command handler
        .branch(
            Update::filter_message().chain(
                dptree::entry()
                    .filter_command::<Command>()
                    .endpoint(handle_command),
            ),
        )
        // save Poll response
        .branch(
            dptree::entry()
                .filter_map(extractors::get_poll)
                .endpoint(handle_poll_update),
        )
        // handle the case in which user is banned the bot
        .branch(Update::filter_my_chat_member().endpoint(handle_ban))
        // generic callbacks handler
        .branch(Update::filter_callback_query().endpoint(handle_callback))
        // setup schedule WebApp
        .branch(
            Update::filter_message()
                .filter_map(extractors::get_web_app_data)
                .endpoint(setup_schedule::handle_webapp),
        )
        // any other text message – append to diary
        .branch(
            Update::filter_message()
                .filter_map(extractors::get_message_text)
                .endpoint(diary::handlers::message::append_diary_entry),
        );

    Ok((
        TgDispatcher::builder(bot.clone(), handler)
            .dependencies(dptree::deps![pool, config.clone(), plotter])
            .enable_ctrlc_handler()
            .build(),
        bot,
    ))
}
