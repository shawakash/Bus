use serde::{Deserialize, Serialize};

use crate::{config::ServiceType, utils::deserialize_with_env};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PostgresConnectionConfig {
    pub service_type: ServiceType,
    pub host: String,
    #[serde(default = "default_postgres_port")]
    pub port: u16,
    pub username: String,
    #[serde(deserialize_with = "deserialize_with_env")]
    password: String,
    pub database: String,
    pub schema: Option<String>,
    pub ssl_mode: Option<String>,
    pub connection_timeout: Option<u64>,
}

impl PostgresConnectionConfig {
    pub fn get_password(&self) -> String {
        self.password.clone()
    }
}

fn default_postgres_port() -> u16 {
    5432
}
