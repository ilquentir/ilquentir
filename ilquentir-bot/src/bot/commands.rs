use teloxide::utils::command::BotCommands;

#[derive(Debug, Clone, BotCommands)]
#[command(rename_rule = "snake_case")]
pub enum Command {
    #[command(description = "Start your journey with Ilqentir.")]
    Start,
    #[command(description = "Get today's stat.")]
    GetStat,
    #[command(description = "Disable Ilquentir reminders and periodic stats.")]
    Stop,
}
