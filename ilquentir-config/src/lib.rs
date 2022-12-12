use std::{ops::Deref, sync::Arc, time::Duration};

use color_eyre::Result;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Config(Arc<ConfigInner>);

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self(Arc::new(envy::from_env()?)))
    }
}

impl Deref for Config {
    type Target = ConfigInner;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfigInner {
    /// Url for database connection
    pub database_url: String,
    /// API key for GIPHY
    pub giphy_key: String,
    /// Specified environment
    pub environment: String,
    /// API key for Honeycomb
    pub honeycomb_key: String,
    /// URL for Honeycomb exporter
    pub exporter_url: String,
    /// Scheduler interval: how long should pauses be between updates
    #[serde(with = "humantime_serde")]
    pub scheduler_interval: Duration,
    /// Minimal response delay for today's summary
    #[serde(with = "humantime_serde")]
    pub min_reply_delay: Duration,
}
