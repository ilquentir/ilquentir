use ilquentir_models::PollWeeklyUserStat;
use time::{Date, Weekday};

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

    format!("{weekday}, {date}")
}

pub fn personal_weekly_stat(stats: &[PollWeeklyUserStat]) -> String {
    if stats.is_empty() {
        return r"На этой неделе от тебя не было вестей :\(".to_owned();
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
        r"Класс, плохих дней за последнюю неделю не было\!".to_owned()
    } else {
        format!(r"Самые грустные дни за неделю: {}\.", worst_days.join(", "))
    }
    .replace('-', r"\-");

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
        r"На этой неделе было не очень весело, надеюсь, следующая пройдёт лучше\!".to_owned()
    } else {
        format!(
            r"А вот и лучшие дни последней недели: {}\.
Не забывай, что бывает классно\!",
            best_days.join(", ")
        )
    }
    .replace('-', r"\-");

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
