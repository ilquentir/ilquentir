use std::sync::Arc;

use aws_sdk_s3::{config::Region, primitives::ByteStream, Client};
use bytes::Bytes;
use color_eyre::{
    eyre::{bail, ensure},
    Result,
};
use csv_async::Terminator;
use tokio::{
    fs::{File, OpenOptions},
    process::Command,
    sync::RwLock,
};
use tracing::{debug, info};

use ilquentir_config::Config;
use ilquentir_models::{PgTransaction, WideHowWasYourDay};
use url::Url;

#[derive(Debug, Clone)]
pub struct Plotter {
    // FIXME: I use Arc<RwLock<File>> because I recreate the file on every update.
    // I think it's not the best solution, but I don't know how to do it better RN,
    // truncating it via File::set_len() doesn't work.
    wide: Arc<RwLock<File>>,
    aws_client: Client,
    config: Config,
}

impl Plotter {
    #[tracing::instrument(skip_all, err)]
    pub async fn new(txn: &mut PgTransaction<'_>, config: Config) -> Result<Self> {
        let wide_path = &config.graph.wide_how_was_your_day_path;

        let file_ = OpenOptions::new()
            .read(true)
            .write(true)
            .open(wide_path)
            .await;

        let wide = match file_ {
            Ok(file_) => file_,
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => {
                    info!("not found wide, creating");

                    OpenOptions::new()
                        .write(true)
                        .read(true)
                        .create_new(true)
                        .open(wide_path)
                        .await?
                }
                _ => bail!(e),
            },
        };
        let wide = Arc::new(RwLock::new(wide));

        let shared_config = aws_config::from_env()
            .endpoint_url(&config.s3.endpoint)
            .region(Region::new(config.s3.region.clone()))
            .load()
            .await;

        let aws_client = aws_sdk_s3::Client::new(&shared_config);

        let this = Self {
            wide,
            aws_client,
            config,
        };
        this.check_regenerate_wide(txn, true).await?;

        Ok(this)
    }

    #[tracing::instrument(skip_all, err)]
    async fn check_regenerate_wide(
        &self,
        txn: &mut PgTransaction<'_>,
        force: bool,
    ) -> Result<bool> {
        info!("updating wide table");

        // acquire exclusive lock to write to wide export file
        let mut wide_file = self.wide.write().await;

        // check, whether wide export is in actual state
        let elapsed_since_update = wide_file.metadata().await?.modified()?.elapsed()?;
        debug!(?elapsed_since_update, "got following freshness info");

        if !force && elapsed_since_update < self.config.graph.wide_how_was_your_day_max_age {
            info!("wide table is in actual state");

            return Ok(false);
        }
        let wide_table = WideHowWasYourDay::collect(txn).await?;

        // clear contents of wide export
        *wide_file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .open(&self.config.graph.wide_how_was_your_day_path)
            .await?;

        // export data anew
        let mut writer = csv_async::AsyncWriterBuilder::new()
            .terminator(Terminator::CRLF)
            .has_headers(true)
            .create_serializer(&mut *wide_file);

        for line in wide_table {
            writer.serialize(line).await?;
        }
        drop(writer);
        wide_file.sync_all().await?;

        info!("wide table updated");

        Ok(true)
    }

    /// Create and upload user's interactive statistics.
    ///
    /// Returns URL of exported data
    #[tracing::instrument(skip(self, txn), err)]
    pub async fn create_plot(&self, txn: &mut PgTransaction<'_>, user_tg_id: i64) -> Result<Url> {
        self.check_regenerate_wide(txn, false).await?;

        let graph = self.plot(user_tg_id).await?;

        self.upload_to_s3(&graph, user_tg_id).await
    }

    /// Creates graph by calling python generator
    ///
    /// # Warning
    ///
    /// This function is guaranteed to fail if there is no data in wide table.
    /// This is known issue and will be (hopefully :)) fixed in future.
    #[tracing::instrument(skip(self), err)]
    async fn plot(&self, user_tg_id: i64) -> Result<String> {
        let script_path = &self.config.graph.plotly_python_code_file;
        let wide_path = &self.config.graph.wide_how_was_your_day_path;

        let _wide = self.wide.read().await;

        let mut command = Command::new("python3.11");

        command
            .arg(script_path)
            .arg(wide_path)
            .arg(user_tg_id.to_string())
            .args([
                self.config.graph.start_date.to_string(),
                self.config.graph.end_date.to_string(),
            ]);

        info!(python_command = ?command, "running python code");
        let output = command.output().await?;
        debug!(output = ?output, "got output from python");

        ensure!(
            output.status.success(),
            "failed to run python code: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        Ok(String::from_utf8(output.stdout)?)
    }

    /// Uploads gresulting graph to S3 (DigitalOcean Spaces, currently)
    #[tracing::instrument(skip(self, graph), fields(graph_len = graph.len()), err)]
    async fn upload_to_s3(&self, graph: &str, user_tg_id: i64) -> Result<Url> {
        let key = format!(
            "{env}/graphs-v0/{user_tg_id}_{uuid}.html",
            env = &self.config.environment,
            uuid = uuid::Uuid::new_v4(),
        );

        self.aws_client
            .put_object()
            .bucket(&self.config.s3.bucket)
            .acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead)
            .key(&key)
            .content_type("text/html")
            .body(ByteStream::from(Bytes::from(graph.as_bytes().to_vec())))
            .send()
            .await?;

        let static_hostname = &self.config.s3.static_url;

        Ok(Url::parse(&format!("{static_hostname}/{key}"))?)
    }
}
