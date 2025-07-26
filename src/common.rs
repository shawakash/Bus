#![allow(dead_code)]

use crate::config::{ScheduleConfig, ServiceType};

#[async_trait::async_trait]
pub trait BackupService: Send + Sync {
    async fn backup(
        &self,
        timestamp: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;

    fn get_schedule(&self) -> &ScheduleConfig;
    fn alias(&self) -> &str;
    fn backup_dir(&self) -> &str;
    fn service_type(&self) -> &ServiceType;
}
