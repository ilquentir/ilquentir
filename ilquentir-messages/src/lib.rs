pub fn stats_for_today(graph: &str) -> String {
    format!(include_str!("./messages/update/stats_for_today.md"), graph = graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let text = r#"Поймали ответ!

Уже [ско!ро](https://google.com/?q=never) тебе придёт статистика а сегодня, а пока напомню, что доступную стату можно посмотреть по запросу `/get_stat`\ :\)"#;

        for event in pulldown_cmark::Parser::new(text) {
            println!("{:?}", event);
        }
    }
}
