use std::{ops::Deref, path::PathBuf, sync::Arc, time::Duration};

use color_eyre::Result;
use serde::Deserialize;
use time::Date;

/// Configuration for the bot.
///
/// The config is Arc-wrapped internally, so it can be
/// safely cloned and shared between threads.
///
/// Main scenario is to use `Config::from_env()`
/// to get the configuration from the environment.
///
/// # Example
/// ```
/// use ilquentir_config::Config;
///
/// match Config::from_env() {
///    Ok(config) => println!("Config: {config:?}"),
///    Err(e) => println!("Error: {e}"),
/// }
/// ```
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

    /// S3 configuration
    #[serde(default, flatten)]
    pub s3: S3Config,

    /// Configuration for the python-based mood graph
    #[serde(default, flatten)]
    pub graph: GraphConfig,
}

/// Configuration of an S3 storage
#[derive(Debug, Deserialize)]
pub struct S3Config {
    /// Name of the bucket
    #[serde(rename = "aws_s3_bucket")]
    pub bucket: String,
    /// Region of the bucket
    #[serde(rename = "aws_default_region")]
    pub region: String,
    /// S3 endpoint
    #[serde(rename = "aws_s3_endpoint")]
    pub endpoint: String,
    /// Static hostname for the bucket, e.g. `https://ilquentir.fra15.digitaloceanspaces.com`
    #[serde(rename = "aws_s3_static_url")]
    pub static_url: String,
}

/// Configuration for the python-based mood graph
#[derive(Debug, Deserialize)]
pub struct GraphConfig {
    /// Path for the wide_how_was_your_day export
    pub wide_how_was_your_day_path: PathBuf,
    /// Max tolerable age of wide_how_was_your_day table
    #[serde(with = "humantime_serde")]
    pub wide_how_was_your_day_max_age: Duration,
    /// Path to python file, containing plotly graphing function
    pub plotly_python_code_file: PathBuf,

    /// Starting date for the graph
    #[serde(rename = "graph_start_date")]
    pub start_date: Date,
    /// Ending date for the graph
    #[serde(rename = "graph_end_date")]
    pub end_date: Date,
}
