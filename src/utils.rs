use regex::Regex;
use serde::{Deserialize, Deserializer};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};

use std::env;

pub fn make_logger(prefix: &str, dir: &str) -> WorkerGuard {
    let now = chrono::Local::now()
        .format("%Y-%m-%d %H:%M:%S.%f")
        .to_string();
    let filename = format!("{prefix}_logger_{}", now);
    let appender: RollingFileAppender = tracing_appender::rolling::daily(dir, filename);

    let (non_blocking, guard) = tracing_appender::non_blocking(appender);

    let console_layer = fmt::layer()
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_ansi(true);

    let file_layer = fmt::layer()
        // .json()
        .with_ansi(false)
        .with_writer(non_blocking);

    let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(console_layer)
        .with(file_layer)
        .init();

    // keep this alive till the binary is running
    guard
}

pub fn deserialize_with_env<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    substitute_env_vars(&s).map_err(serde::de::Error::custom)
}

pub fn substitute_env_vars(
    content: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let re = Regex::new(r"\$\{([^}:]+)(?::([^}]*))?\}")?;
    let mut result = content.to_string();

    for captures in re.captures_iter(content) {
        let full_match = &captures[0];
        let env_var = &captures[1];
        let default_value = captures.get(2).map(|m| {
            let val = m.as_str();
            if val.starts_with('-') { &val[1..] } else { val }
        });

        let replacement = match env::var(env_var) {
            Ok(value) if !value.is_empty() => value,
            _ => {
                if let Some(default) = default_value {
                    default.to_string()
                } else {
                    return Err(format!("Environment variable '{}' not found", env_var).into());
                }
            }
        };

        result = result.replace(full_match, &replacement);
    }

    Ok(result)
}

#[macro_export]
macro_rules! with_env_substitution {
    ($field:ident) => {
        #[serde(deserialize_with = "crate::utils::deserialize_with_env")]
        pub $field: String,
    };
}
