mod config;

use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use config::Config;
use serde::Deserialize;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, Duration};

macro_rules! log_info {
    ($($arg:tt)*) => {
        eprintln!("[{}] {}", Local::now().format("%Y-%m-%d %H:%M:%S"), format!($($arg)*))
    };
}

macro_rules! log_error {
    ($($arg:tt)*) => {
        eprintln!("[{}] ERROR: {}", Local::now().format("%Y-%m-%d %H:%M:%S"), format!($($arg)*))
    };
}

#[derive(Parser, Debug)]
#[command(name = "moonblokz-log-collector")]
#[command(about = "MoonBlokz Log Collector - Downloads logs from telemetry hub", long_about = None)]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,
}

#[derive(Debug, Deserialize)]
struct LogEntry {
    item_id: u64,
    timestamp: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct DownloadResponse {
    logs: Vec<LogEntry>,
    update_interval: u64,
}

struct LogCollector {
    config: Config,
    client: reqwest::Client,
    last_id: u64,
}

impl LogCollector {
    fn new(config: Config) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { config, client, last_id: 0 })
    }

    async fn run(&mut self) -> Result<()> {
        // Ensure log file exists or can be created
        self.ensure_log_file().await?;

        log_info!("Log collector started. Downloading from: {}", self.config.hub_url);
        log_info!("Writing logs to: {}", self.config.log_file.display());

        let mut poll_interval: u64 = 60; // Default until we get first response

        loop {
            match self.fetch_and_save_logs().await {
                Ok((count, new_interval)) => {
                    poll_interval = new_interval;
                    if count > 0 {
                        log_info!("Downloaded and saved {} log entries (last_id: {})", count, self.last_id);
                    }
                }
                Err(e) => {
                    log_error!("Error during log download: {}", e);
                    // Error handling for 401 is done in fetch_and_save_logs
                    // If we get here with a 401, the function has already returned it as an error
                }
            }

            sleep(Duration::from_secs(poll_interval)).await;
        }
    }

    async fn ensure_log_file(&self) -> Result<()> {
        // Try to open the file for appending, create if it doesn't exist
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.log_file)
            .await
            .with_context(|| format!("Failed to open or create log file: {}", self.config.log_file.display()))?;

        Ok(())
    }

    async fn fetch_and_save_logs(&mut self) -> Result<(usize, u64)> {
        let url = format!("{}/download?last_log_message_id={}", self.config.hub_url.trim_end_matches('/'), self.last_id);

        let response = self
            .client
            .get(&url)
            .header("X-Api-Key", &self.config.api_key)
            .send()
            .await
            .context("Failed to send request to hub")?;

        let status = response.status();

        if !status.is_success() {
            match status.as_u16() {
                401 => {
                    // Return a specific error and terminate
                    anyhow::bail!("401 Unauthorized: Invalid API key. Terminating.");
                }
                400 => {
                    log_error!("400 Bad Request from server");
                    return Ok((0, 60));
                }
                500..=599 => {
                    log_error!("Server error: {}", status);
                    return Ok((0, 60));
                }
                _ => {
                    log_error!("Unexpected status code: {}", status);
                    return Ok((0, 60));
                }
            }
        }

        let download_response: DownloadResponse = response.json().await.context("Failed to parse JSON response")?;

        let log_count = download_response.logs.len();
        let update_interval = download_response.update_interval;

        if log_count > 0 {
            self.append_logs_to_file(&download_response.logs).await?;

            // Update last_id to the highest item_id seen
            if let Some(max_id) = download_response.logs.iter().map(|entry| entry.item_id).max() {
                self.last_id = max_id;
            }
        }

        Ok((log_count, update_interval))
    }

    async fn append_logs_to_file(&self, logs: &[LogEntry]) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.log_file)
            .await
            .with_context(|| format!("Failed to open log file for writing: {}", self.config.log_file.display()))?;

        for entry in logs {
            let line = format!("{}:{}\n", entry.timestamp, entry.message);
            file.write_all(line.as_bytes()).await.context("Failed to write log entry to file")?;
        }

        file.flush().await.context("Failed to flush log file")?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let config = Config::from_file(&args.config).context("Failed to load configuration")?;

    let mut collector = LogCollector::new(config)?;

    collector.run().await
}
