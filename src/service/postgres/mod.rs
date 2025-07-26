use tracing::{info, warn};

use crate::common::BackupService;
use crate::config::{ScheduleConfig, ServiceConfig, ServiceType};
use crate::service::postgres::config::PostgresConnectionConfig;

pub mod config;

pub struct PostgresJob {
    service_type: ServiceType,
    alias: String,
    schedule: ScheduleConfig,
    connection: PostgresConnectionConfig,
    backup_options: Option<std::collections::HashMap<String, String>>,
    backup_dir: String,
}

impl PostgresJob {
    pub fn new(config: ServiceConfig, backup_dir: String) -> Self {
        Self {
            service_type: ServiceType::Postgres,
            alias: config.alias,
            schedule: config.schedule,
            connection: config.connection.as_postgres().unwrap().clone(),
            backup_options: config.backup_options,
            backup_dir,
        }
    }
}

#[async_trait::async_trait]
impl BackupService for PostgresJob {
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
            "{}/postgres_{}_{}.sql",
            self.backup_dir(),
            self.alias(),
            timestamp
        );

        info!(
            "Creating PostgreSQL backup for {}: {}",
            self.alias(),
            backup_file,
        );

        let mut cmd = tokio::process::Command::new("pg_dump");
        cmd.env("PGPASSWORD", &self.connection.get_password())
            .args([
                "-h",
                self.connection.host.as_str(),
                "-p",
                self.connection.port.to_string().as_str(),
                "-U",
                self.connection.username.as_str(),
                "-d",
                self.connection.database.as_str(),
                "-f",
                &backup_file,
                "--verbose",
                "--no-password",
            ]);

        if let Some(ref options) = self.backup_options {
            for (key, value) in options {
                match key.as_str() {
                    "schema_only" if value == "true" => {
                        cmd.arg("--schema-only");
                    }
                    "data_only" if value == "true" => {
                        cmd.arg("--data-only");
                    }
                    "format" => {
                        cmd.args(["--format", value]);
                    }
                    "exclude_table" => {
                        cmd.args(["--exclude-table", value]);
                    }
                    _ => {}
                }
            }
        }

        let output = cmd.output().await?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("pg_dump failed for {}: {}", self.alias(), error_msg).into());
        }

        let compressed_file = format!("{}.gz", backup_file);
        let output = tokio::process::Command::new("gzip")
            .arg(&backup_file)
            .output()
            .await?;

        if !output.status.success() {
            warn!("Failed to compress PostgreSQL backup for {}", self.alias());
            Ok(backup_file)
        } else {
            info!(
                "PostgreSQL backup compressed for {}: {}",
                self.alias(),
                compressed_file
            );
            Ok(compressed_file)
        }
    }

    fn get_schedule(&self) -> &ScheduleConfig {
        &self.schedule
    }
}
