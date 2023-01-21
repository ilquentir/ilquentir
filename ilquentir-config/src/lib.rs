use std::{ops::Deref, path::PathBuf, sync::Arc, time::Duration};

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
    /// Specified environment
    pub environment: String,
    /// API key for Honeycomb
    pub honeycomb_key: String,
    /// URL for Honeycomb exporter
    pub exporter_url: String,
    /// Scheduler interval: how long should pauses be between updates
    #[serde(with = "humantime_serde")]
    pub scheduler_interval: Duration,

    /// Path for the wide_how_was_your_day export
    pub wide_how_was_your_day_path: PathBuf,
    /// Max tolerable age of wide_how_was_your_day table
    #[serde(with = "humantime_serde")]
    pub wide_how_was_your_day_max_age: Duration,
    /// Path to python file, containing plotly graphing function
    pub plotly_python_code_file: PathBuf,

    /// Response delay for today's summary
    #[deprecated = "we do not send today's summary manually"]
    #[serde(with = "humantime_serde")]
    pub reply_delay: Duration,
    /// Minimal response delay for today's summary
    #[deprecated = "we do not send today's summary manually"]
    #[serde(with = "humantime_serde", default = "default_min_reply_delay")]
    pub min_reply_delay: Duration,
}

fn default_min_reply_delay() -> Duration {
    Duration::from_secs(60 * 3)
}
