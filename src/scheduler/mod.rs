use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use tokio::time;
use tracing::{error, info, warn};

use crate::{
    common::BackupService,
    config::{CommonConfig, Config},
    service::ServiceFactory,
};

pub struct BackupScheduler {
    services: Vec<Arc<dyn BackupService>>,
    common_config: CommonConfig,
}

impl BackupScheduler {
    pub fn new(config: Config) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut services = Vec::new();

        for service_config in config.services {
            let service =
                ServiceFactory::create_service(service_config, config.common.backup_dir.clone())?;
            services.push(Arc::from(service));
        }

        Ok(Self {
            services,
            common_config: config.common,
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Starting backup scheduler with {} services",
            self.services.len()
        );

        tokio::fs::create_dir_all(&self.common_config.backup_dir).await?;

        let mut handles = Vec::new();

        for service in &self.services {
            let service_clone = Arc::clone(service);
            let common_config = self.common_config.clone();

            let handle = tokio::spawn(Self::run_service_scheduler(service_clone, common_config));

            handles.push(handle);
        }

        // Wait for all schedulers to complete (they shouldn't unless there's an error)
        let results = futures::future::join_all(handles).await;

        for (i, result) in results.into_iter().enumerate() {
            match result {
                Ok(Err(e)) => error!("Service {} scheduler failed: {}", i, e),
                Err(e) => error!("Service {} scheduler panicked: {}", i, e),
                _ => {}
            }
        }

        Ok(())
    }

    async fn run_service_scheduler(
        service: Arc<dyn BackupService>,
        common_config: CommonConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut interval =
            time::interval(Duration::from_secs(service.get_schedule().interval_seconds));

        info!(
            "Started scheduler for service '{}' with interval {} seconds",
            service.alias(),
            service.get_schedule().interval_seconds
        );

        loop {
            interval.tick().await;

            let timestamp = Utc::now().format("%Y-%m-%d_%H:%M:%S.%f").to_string();

            info!("Starting backup for service '{}'", service.alias());

            match service.backup(&timestamp).await {
                Ok(backup_file) => {
                    info!(
                        "Backup completed for '{}': {}",
                        service.alias(),
                        backup_file
                    );
                }
                Err(e) => {
                    error!("Backup failed for '{}': {}", service.alias(), e);
                }
            }

            if let Err(e) = Self::cleanup_old_backups(&common_config, service.alias()).await {
                warn!(
                    "Failed to cleanup old backups for '{}': {}",
                    service.alias(),
                    e
                );
            }
        }
    }

    async fn cleanup_old_backups(
        common_config: &CommonConfig,
        service_name: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let retention_days = common_config.retention_days.unwrap_or(7);
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days);

        info!(
            "Cleaning up backups older than {} days for service '{}'",
            retention_days, service_name
        );

        let mut entries = tokio::fs::read_dir(&common_config.backup_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();

            if file_name.contains(&format!("{}_{}", service_name.replace("-", "_"), "")) {
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(created) = metadata.created() {
                        let created_datetime: DateTime<Utc> = created.into();

                        if created_datetime < cutoff_date {
                            info!("Removing old backup for '{}': {:?}", service_name, path);
                            if let Err(e) = tokio::fs::remove_file(&path).await {
                                warn!("Failed to remove old backup {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
