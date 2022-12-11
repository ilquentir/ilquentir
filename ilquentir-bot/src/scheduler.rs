use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use color_eyre::Result;
use rasciigraph::{plot, Config};
use sqlx::PgPool;
use teloxide::{dispatching::ShutdownToken as DispatcherShutdownToken, Bot};
use tokio::time::interval;
use tracing::{error, info};

use crate::{
    bot::Dispatcher,
    models::{Poll, PollStat},
};

#[derive(Clone)]
pub struct Scheduler {
    dispatcher_shutdown_token: DispatcherShutdownToken,
    running: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
pub struct ShutdownToken(Arc<AtomicBool>);

impl ShutdownToken {
    /// Stops the scheduler.
    ///
    /// Scheduler waits for the next tick to stop.
    pub fn shutdown(&self) {
        self.0.store(false, std::sync::atomic::Ordering::Release)
    }
}

impl Scheduler {
    pub fn new(dispatcher: &Dispatcher) -> Self {
        Self {
            dispatcher_shutdown_token: dispatcher.shutdown_token(),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn shutdown_token(&self) -> ShutdownToken {
        ShutdownToken(self.running.clone())
    }

    pub async fn start(&self, pool: &PgPool, bot: &Bot) -> Result<()> {
        self.running
            .store(true, std::sync::atomic::Ordering::Release);

        let mut interval = interval(Duration::from_secs(60));

        loop {
            interval.tick().await;
            let schedule_result = handle_scheduled(bot, pool).await;

            if let Err(e) = schedule_result {
                error!(error = %e, "got an error while scheduling");
                self.dispatcher_shutdown_token.shutdown()?.await;

                return Err(e);
            };

            if !self.running() {
                info!("scheduler shut down");

                break;
            }
        }
        Ok(())
    }

    pub fn running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::Acquire)
    }
}

#[tracing::instrument(skip_all, err)]
pub async fn handle_scheduled(bot: &Bot, pool: &PgPool) -> Result<()> {
    let mut txn = pool.begin().await?;

    info!("checking if there exist some unsent polls");
    let polls = Poll::get_pending(&mut txn).await?;
    if polls.is_empty() {
        info!("no polls to send");
    } else {
        info!(unsent_poll_count = polls.len(), "found some unsent polls");
    }

    // FIXME: do not require transaction usage in Poll::get_pending
    txn.commit().await?;

    for poll in polls {
        info!(
            poll_id = poll.id,
            user_id = poll.chat_tg_id,
            "sending scheduled poll"
        );

        let mut txn = pool.begin().await?;

        poll.publish_to_tg(&mut txn, bot).await?;

        txn.commit().await?;
    }

    Ok(())
}

pub fn create_chart(poll_stats: &[PollStat]) -> Result<String> {
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
        Config::default()
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
