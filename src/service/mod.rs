use crate::{
    common::BackupService,
    config::{ServiceConfig, ServiceType},
    service::{postgres::PostgresJob, redis::RedisJob},
};

pub mod postgres;
pub mod redis;

pub struct ServiceFactory;

impl ServiceFactory {
    pub fn create_service(
        config: ServiceConfig,
        backup_dir: String,
    ) -> Result<Box<dyn BackupService>, Box<dyn std::error::Error + Send + Sync>> {
        match config.service_type {
            ServiceType::Postgres => Ok(Box::new(PostgresJob::new(config, backup_dir))),
            ServiceType::Redis => Ok(Box::new(RedisJob::new(config, backup_dir))),
            // _ => Err(format!("Unknown service type: {}", config.service_type).into()),
        }
    }
}
