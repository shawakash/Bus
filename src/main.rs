use std::path::PathBuf;

use clap::Parser;
use tracing::info;

use crate::{config::Config, scheduler::BackupScheduler, utils::make_logger};

mod common;
mod config;
mod scheduler;
mod service;
mod utils;

#[derive(Parser)]
#[command(name = "cron")]
#[command(about = "A configurable backup service for databases and redis")]
struct Cli {
    #[arg(short, long)]
    config: PathBuf,
    #[arg(short, long)]
    prefix: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    let content = tokio::fs::read_to_string(&cli.config)
        .await
        .expect("Failed to read config file");

    let config: Config =
        toml::from_str(content.as_str()).expect("[toml]: Failed to parse config file");

    let log_dir = match &config.common.log_dir {
        Some(log_file) => log_file,
        None => "logs",
    };

    let _guard = make_logger(&cli.prefix, log_dir);

    info!("Starting backup service with config: {:?}", cli.config);

    let scheduler = BackupScheduler::new(config).unwrap();
    scheduler.start().await?;

    Ok(())
}

// cargo run -- --prefix cron --config ./config.toml
