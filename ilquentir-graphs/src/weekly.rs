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

    let mut worst_days: Vec<_> = stats
        .iter()
        .filter_map(|stat| (stat.selected_value == 4).then(|| display_date(stat.date)))
        .collect();
    if worst_days.is_empty() {
        worst_days.extend(
            stats
                .iter()
                .filter_map(|stat| (stat.selected_value == 3).then(|| display_date(stat.date))),
        );
    }

    let worst_str = if worst_days.is_empty() {
        "Класс, плохих дней за последнюю неделю не было!".to_owned()
    } else {
        format!("Самые грустные дни за неделю: {}.", worst_days.join(", "))
    };

    let mut best_days: Vec<_> = stats
        .iter()
        .filter_map(|stat| (stat.selected_value == 0).then(|| display_date(stat.date)))
        .collect();
    if best_days.is_empty() {
        best_days.extend(
            stats
                .iter()
                .filter_map(|stat| (stat.selected_value == 1).then(|| display_date(stat.date))),
        );
    }

    let best_str = if best_days.is_empty() {
        "На этой неделе было не очень весело, надеюсь, следующая пройдёт лучше!".to_owned()
    } else {
        format!(
            "А вот и лучшие дни последней недели: {}.
Классно, что хорошие дни случаются!",
            best_days.join(", ")
        )
    };

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
    result.push_str(&format!("{worst_str}\n\n{best_str}"));

    result
}
