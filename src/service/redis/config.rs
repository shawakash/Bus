use serde::{Deserialize, Serialize};

use crate::{config::ServiceType, utils::deserialize_with_env};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RedisConnectionConfig {
    pub service_type: ServiceType,
    pub host: String,
    #[serde(default = "default_redis_port")]
    pub port: u16,
    #[serde(deserialize_with = "deserialize_with_env")]
    pub password: String,
    pub cluster_mode: Option<bool>,
    pub sentinel_hosts: Option<Vec<String>>,
    pub master_name: Option<String>,
}

impl RedisConnectionConfig {
    pub fn get_password(&self) -> String {
        self.password.clone()
    }
}

fn default_redis_port() -> u16 {
    6379
}
