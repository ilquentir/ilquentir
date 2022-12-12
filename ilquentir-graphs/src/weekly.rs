use ilquentir_models::PollWeeklyUserStat;

pub fn personal_weekly_stat(stats: &[PollWeeklyUserStat]) -> String {
    if stats.is_empty() {
        return "На этой неделе от тебя не было вестей :(".to_owned();
    }

    let mut worst_days: Vec<_> =
        stats.iter().filter_map(|stat| (stat.selected_value == 5).then_some(stat.date.to_string())).collect();
    if worst_days.is_empty() {
        worst_days.extend(stats.iter().filter_map(|stat| (stat.selected_value == 4).then_some(stat.date.to_string())));
    }

    let worst_str = if worst_days.is_empty() {
        r"Класс, плохих дней за последнюю неделю не было\!".to_owned()
    } else {
        format!("Самые грустные дни за неделю: {}", worst_days.join(", "))
    }.replace('-', "\\-");

    let mut best_days: Vec<_> =
        stats.iter().filter_map(|stat| (stat.selected_value == 0).then_some(stat.date.to_string())).collect();
    if best_days.is_empty() {
        best_days.extend(stats.iter().filter_map(|stat| (stat.selected_value == 1).then_some(stat.date.to_string())));
    }

    let best_str = if best_days.is_empty() {
        r"На этой неделе было не очень весело, надеюсь, следующая пройдёт лучше\!".to_owned()
    } else {
        format!(r"А вот и лучшие дни последней недели: {}
Не забывай, бывает же классно\!", best_days.join(", "))
    }.replace('-', "\\-");

    let mut result = "Твоя статистика за последние семь дней:\n\n```\n".to_owned();

    for stat in stats {
        result.push_str(&format!("{}: {:+}\n", stat.date, 2 - stat.selected_value));
    }

    result.push_str("```\n\n");
    result.push_str(&format!("{worst_str}\n{best_str}"));

    result
}
