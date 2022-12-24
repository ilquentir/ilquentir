use teloxide::utils::command::BotCommands;

#[derive(Debug, Clone, BotCommands)]
#[command(rename_rule = "snake_case")]
pub enum Command {
    #[command(description = "Запустить Ильквентир")]
    Start,
    #[command(description = "Настроить опрос про ежедневные соббытия")]
    DailyEventsSettings,
    #[command(description = "Дай стату")]
    GetStat,
    #[command(description = "Выключить Ильквентир (не будет приходить стата и опросы)")]
    Stop,
}
