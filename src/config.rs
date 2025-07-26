use std::{collections::HashMap, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::service::postgres::config::PostgresConnectionConfig;
use crate::service::redis::config::RedisConnectionConfig;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub common: CommonConfig,
    pub services: Vec<ServiceConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommonConfig {
    pub backup_dir: String,
    pub log_level: Option<String>,
    pub log_dir: Option<String>,
    pub retention_days: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServiceConfig {
    #[serde(rename = "type")]
    pub service_type: ServiceType,
    pub alias: String,
    pub schedule: ScheduleConfig,
    #[serde(deserialize_with = "deserialize_connection")]
    pub connection: ConnectionConfig,
    pub backup_options: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceType {
    Postgres,
    Redis,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionConfig {
    Postgres(PostgresConnectionConfig),
    Redis(RedisConnectionConfig),
}

fn deserialize_connection<'de, D>(deserializer: D) -> Result<ConnectionConfig, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let value = serde_json::Value::deserialize(deserializer)?;

    if let Some(service_type) = value.get("service_type").and_then(|v| v.as_str()) {
        match service_type {
            "postgres" => {
                let config: PostgresConnectionConfig =
                    serde_json::from_value(value).map_err(D::Error::custom)?;
                Ok(ConnectionConfig::Postgres(config))
            }
            "redis" => {
                let config: RedisConnectionConfig =
                    serde_json::from_value(value).map_err(D::Error::custom)?;
                Ok(ConnectionConfig::Redis(config))
            }
            _ => Err(D::Error::custom(format!(
                "Unknown service type: {}",
                service_type
            ))),
        }
    } else {
        Err(D::Error::missing_field("type"))
    }
}

impl ConnectionConfig {
    // Type-specific getters
    pub fn as_postgres(&self) -> Option<&PostgresConnectionConfig> {
        match self {
            ConnectionConfig::Postgres(config) => Some(config),
            _ => None,
        }
    }

    pub fn as_redis(&self) -> Option<&RedisConnectionConfig> {
        match self {
            ConnectionConfig::Redis(config) => Some(config),
            _ => None,
        }
    }
}

impl Display for ServiceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ServiceConfig {{ type: {}, alias: {}, schedule: {:?}, connection: {:?} }}",
            self.service_type, self.alias, self.schedule, self.connection
        )
    }
}

impl Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceType::Postgres => write!(f, "postgres"),
            ServiceType::Redis => write!(f, "redis"),
        }
    }
}

impl Display for ScheduleConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ScheduleConfig {{ interval_seconds: {}, timezone: {:?}, start_time: {:?} }}",
            self.interval_seconds, self.timezone, self.start_time
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScheduleConfig {
    pub interval_seconds: u64,
    pub timezone: Option<String>,
    pub start_time: Option<String>,
}
