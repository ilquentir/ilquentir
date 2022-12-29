use std::sync::{atomic::AtomicBool, Arc};

use color_eyre::Result;
use sqlx::PgPool;
use strum::IntoEnumIterator;
use teloxide::dispatching::ShutdownToken as DispatcherShutdownToken;
use tokio::time::{interval, MissedTickBehavior};
use tracing::{error, info};

use ilquentir_config::Config;
use ilquentir_models::{Poll, PollKind};

use crate::bot::{
    helpers::{overdue_poll, send_poll},
    Bot, Dispatcher,
};

#[derive(Clone)]
pub struct Scheduler {
    #[allow(dead_code)]
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

    pub async fn start(&self, pool: &PgPool, bot: &Bot, config: &Config) -> Result<()> {
        self.running
            .store(true, std::sync::atomic::Ordering::Release);

        let mut interval = interval(config.scheduler_interval);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            interval.tick().await;
            let send_result = handle_scheduled_delivery(bot, pool).await;
            if let Err(e) = send_result {
                error!(error = %e, "got an error while sending");
            };

            let overdue_result = handle_scheduled_overdue(bot, pool).await;
            if let Err(e) = overdue_result {
                error!(error = %e, "got an error while processing overdue");
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
pub async fn handle_scheduled_delivery(bot: &Bot, pool: &PgPool) -> Result<()> {
    let mut txn = pool.begin().await?;

    info!("checking if there exist some unsent polls");
    let polls = Poll::get_pending(&mut txn).await?;
    if polls.is_empty() {
        info!("no polls to send");
    } else {
        info!(unsent_poll_count = polls.len(), "found some unsent polls");
    }

    // FIXME: do not require txn usage in Poll::get_pending
    txn.commit().await?;

    for poll in polls {
        info!(
            poll_id = poll.id,
            user_id = poll.chat_tg_id,
            "sending scheduled poll"
        );

        let mut txn = pool.begin().await?;

        send_poll(bot, &mut txn, poll).await?;

        txn.commit().await?;
    }

    Ok(())
}

#[tracing::instrument(skip_all, err)]
pub async fn handle_scheduled_overdue(bot: &Bot, pool: &PgPool) -> Result<()> {
    info!("checking if there exist some overdue polls");
    for kind in PollKind::iter() {
        let mut txn = pool.begin().await?;

        let polls = Poll::get_overdue(&mut txn, kind).await?;
        if polls.is_empty() {
            info!(%kind, "no polls to overdue");
        } else {
            info!(%kind, overdue_polls_kind = polls.len(), "found some overdue polls");
        }

        txn.commit().await?;

        for poll in polls {
            info!(
                poll_id = poll.id,
                user_id = poll.chat_tg_id,
                "sending scheduled poll"
            );

            let mut txn = pool.begin().await?;

            overdue_poll(bot, &mut txn, poll).await?;

            txn.commit().await?;
        }
    }

    Ok(())
}
