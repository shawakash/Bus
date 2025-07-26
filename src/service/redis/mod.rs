use tracing::{info, warn};

use crate::{
    common::BackupService,
    config::{ScheduleConfig, ServiceConfig, ServiceType},
    service::redis::config::RedisConnectionConfig,
};

pub mod config;

pub struct RedisJob {
    service_type: ServiceType,
    alias: String,
    schedule: ScheduleConfig,
    connection: RedisConnectionConfig,
    backup_options: Option<std::collections::HashMap<String, String>>,
    backup_dir: String,
}

impl RedisJob {
    pub fn new(config: ServiceConfig, backup_dir: String) -> Self {
        Self {
            service_type: ServiceType::Postgres,
            alias: config.alias,
            schedule: config.schedule,
            connection: config.connection.as_redis().unwrap().clone(),
            backup_options: config.backup_options,
            backup_dir,
        }
    }
}

#[async_trait::async_trait]
impl BackupService for RedisJob {
    fn service_type(&self) -> &ServiceType {
        &self.service_type
    }

    fn alias(&self) -> &str {
        &self.alias
    }

    fn backup_dir(&self) -> &str {
        &self.backup_dir
    }

    async fn backup(
        &self,
        timestamp: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let backup_file = format!(
            "{}/redis_{}_{}.rdb",
            self.backup_dir(),
            self.alias(),
            timestamp
        );

        info!(
            "Creating Redis backup for {}: {}",
            self.alias(),
            backup_file,
        );

        let redis_url = format!("redis://{}:{}", self.connection.host, self.connection.port);

        let client = redis::Client::open(redis_url)?;
        let mut con = client.get_async_connection().await?;

        if !self.connection.get_password().is_empty() {
            let _: () = redis::cmd("AUTH")
                .arg(&self.connection.get_password())
                .query_async(&mut con)
                .await?;
        }

        let backup_method = self
            .backup_options
            .as_ref()
            .and_then(|opts| opts.get("method"))
            .map(|s| s.as_str())
            .unwrap_or("rdb");

        match backup_method {
            "rdb" => {
                let mut cmd = tokio::process::Command::new("redis-cli");
                cmd.args([
                    "-h",
                    &self.connection.host,
                    "-p",
                    &self.connection.port.to_string(),
                ]);

                if !self.connection.get_password().is_empty() {
                    cmd.args(["-a", &self.connection.get_password()]);
                }

                cmd.args(["--rdb", &backup_file]);

                let output = cmd.output().await?;

                if !output.status.success() {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    return Err(format!(
                        "Redis RDB backup failed for {}: {}",
                        self.alias(),
                        error_msg
                    )
                    .into());
                }
            }
            "save" => {
                let _: String = redis::cmd("SAVE").query_async(&mut con).await?;

                let output = tokio::process::Command::new("docker")
                    .args([
                        "cp",
                        &format!("{}:/data/dump.rdb", self.connection.host),
                        &backup_file,
                    ])
                    .output()
                    .await?;

                if !output.status.success() {
                    return Err(
                        format!("Failed to copy Redis dump file for {}", self.alias()).into(),
                    );
                }
            }
            _ => {
                return Err(format!("Unknown backup method: {}", backup_method).into());
            }
        }

        let compressed_file = format!("{}.gz", backup_file);
        let output = tokio::process::Command::new("gzip")
            .arg(&backup_file)
            .output()
            .await?;

        if !output.status.success() {
            warn!("Failed to compress Redis backup for {}", self.alias());
            Ok(backup_file)
        } else {
            info!(
                "Redis backup compressed for {}: {}",
                self.alias(),
                compressed_file
            );
            Ok(compressed_file)
        }
    }

    fn get_schedule(&self) -> &crate::config::ScheduleConfig {
        &self.schedule
    }
}
