use std::fs::{File, OpenOptions};

use color_eyre::{eyre::ensure, Result};
use time::ext::NumericalStdDuration;
use tokio::process::Command;

use ilquentir_config::Config;
use ilquentir_models::{PgTransaction, WideHowWasYourDay};
use tracing::{debug, info};

#[tracing::instrument(skip(txn, config), err)]
pub async fn make_plotly_graph(
    txn: &mut PgTransaction<'_>,
    config: &Config,
    user_tg_id: i64,
) -> Result<String> {
    let path = &config.wide_how_was_your_day_path;
    let file_ = File::open(path);

    let mut need_update = false;

    let file_ = match file_ {
        Ok(file_) => file_,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => {
                need_update = true;

                OpenOptions::new()
                    .write(true)
                    .read(true)
                    .create_new(true)
                    .open(path)?
            }
            _ => return Err(e.into()),
        },
    };

    if !need_update && file_.metadata()?.modified()?.elapsed()? > 5.std_minutes() {
        need_update = true;
    }

    if need_update {
        info!("wide table export is stale, updating it");

        let wide = WideHowWasYourDay::collect(txn).await?;
        let mut writer = csv::WriterBuilder::new()
            .delimiter(b';')
            .has_headers(true)
            .from_path(path)?;

        for line in wide {
            writer.serialize(line)?;
        }

        info!("wide table updated");
    } else {
        info!("wide table is in actual state");
    }

    let mut command = Command::new("python3.11");

    command
        .arg(&config.plotly_python_code_file)
        .arg(user_tg_id.to_string())
        .arg(path)
        .args(["2022-12-05", "2023-02-15"]);

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
