use color_eyre::Result;
use rasciigraph::{plot, Config as PlotConfig};

use ilquentir_models::PollStat;

pub fn chart_daily_stats(poll_stats: &[PollStat]) -> Result<String> {
    let minus_two = poll_stats
        .iter()
        .find_map(|stat| (stat.selected_value == 4).then_some(stat.n_selected))
        .unwrap_or_default() as f64
        * 100.;

    let minus_one = poll_stats
        .iter()
        .find_map(|stat| (stat.selected_value == 3).then_some(stat.n_selected))
        .unwrap_or_default() as f64
        * 100.;

    let zero = poll_stats
        .iter()
        .find_map(|stat| (stat.selected_value == 2).then_some(stat.n_selected))
        .unwrap_or_default() as f64
        * 100.;

    let plus_one = poll_stats
        .iter()
        .find_map(|stat| (stat.selected_value == 1).then_some(stat.n_selected))
        .unwrap_or_default() as f64
        * 100.;

    let plus_two = poll_stats
        .iter()
        .find_map(|stat| (stat.selected_value == 0).then_some(stat.n_selected))
        .unwrap_or_default() as f64
        * 100.;

    let total = (minus_two + minus_one + zero + plus_one + plus_two) / 100.0;
    let series = vec![
        minus_two / total,
        minus_two / total,
        minus_two / total,
        minus_one / total,
        minus_one / total,
        minus_one / total,
        zero / total,
        zero / total,
        zero / total,
        plus_one / total,
        plus_one / total,
        plus_one / total,
        plus_two / total,
        plus_two / total,
        plus_two / total,
        0.0,
    ];

    let mut graph = plot(
        series,
        PlotConfig::default()
            .with_offset(2)
            .with_height(10)
            .with_width(16),
    );

    let re_decimal = regex::Regex::new(r"\.\d{2}")?;
    let re_one_digit = regex::Regex::new(r" (\d\D)")?;
    graph = re_decimal.replace_all(&graph, "").to_string();
    graph = re_one_digit.replace_all(&graph, "0$1").to_string();
    graph += " \n    -2     0    +2  ";
    graph = graph.replace(' ', "┄");

    Ok(graph)
}
