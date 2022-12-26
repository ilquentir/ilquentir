use ilquentir_models::PollWeeklyUserStat;
use time::{macros::format_description, Date, Weekday};

pub fn display_date(date: Date) -> String {
    let weekday = match date.weekday() {
        Weekday::Monday => "пн",
        Weekday::Tuesday => "вт",
        Weekday::Wednesday => "ср",
        Weekday::Thursday => "чт",
        Weekday::Friday => "пт",
        Weekday::Saturday => "сб",
        Weekday::Sunday => "вс",
    };

    let format = format_description!("[day].[month]");

    format!(
        "{weekday} ({date})",
        date = date.format(format).expect("formatting failed")
    )
}

pub fn personal_weekly_stat(stats: &[PollWeeklyUserStat]) -> String {
    if stats.is_empty() {
        return "На этой неделе от тебя не было вестей :(".to_owned();
    }

    let mut result = "```\n".to_owned();

    for stat in stats {
        result.push_str(&format!(
            "{date}: {rate:+}{beer}\n",
            date = display_date(stat.date),
            rate = 2 - stat.selected_value,
            beer = if let Weekday::Friday = stat.date.weekday() {
                " 🍻"
            } else {
                ""
            }
        ));
    }

    result.push_str("```\n");

    result
}
