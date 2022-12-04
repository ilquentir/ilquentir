use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};

use color_eyre::Result;
use sqlx::PgPool;
use teloxide::{dispatching::ShutdownToken as DispatcherShutdownToken, Bot};
use tokio::time::interval;
use tracing::{error, info};

use crate::bot::{handlers::handle_scheduled, Dispatcher};

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
