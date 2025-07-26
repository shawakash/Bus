use std::env;

use crate::{config::Config, utils::substitute_env_vars};

#[test]
fn test_env_var_substitution() {
    dotenvy::dotenv().ok();

    unsafe {
        env::set_var("TEST_VAR", "test_value");
        env::set_var("EMPTY_VAR", "");
    }

    // Test basic substitution
    let input = "password = \"${TEST_VAR}\"";
    let result = substitute_env_vars(input).unwrap();
    assert_eq!(result, "password = \"test_value\"");

    // Test with default value
    let input_with_default = "url = \"${MISSING_VAR:-default_url}\"";
    let result = substitute_env_vars(input_with_default).unwrap();
    assert_eq!(result, "url = \"default_url\"");

    // Test multiple substitutions
    let input_multiple = "host = \"${TEST_VAR}\" port = \"${MISSING_VAR:-5432}\"";
    let result = substitute_env_vars(input_multiple).unwrap();
    assert_eq!(result, "host = \"test_value\" port = \"5432\"");

    // Test error case
    let input_error = "password = \"${MISSING_VAR}\"";
    assert!(substitute_env_vars(input_error).is_err());
}

#[test]
fn test_config_parsing_with_env_vars() {
    dotenvy::dotenv().ok();

    unsafe {
        env::set_var("TEST_DB_PASSWORD", "secret123");
        env::set_var("TEST_REDIS_PASSWORD", "redis_secret");
    }

    let toml_content = r#"
            [common]
            backup_dir = "/tmp/backups"
            log_level = "info"

            [[services]]
            type = "postgres"
            alias = "test-db"

            [services.connection]
            host = "localhost"
            password = "${TEST_DB_PASSWORD}"
            database = "${TEST_DB_NAME:-testdb}"

            [services.schedule]
            interval_seconds = 3600
            timezone = "UTC"
            start_time = "02:00"
        "#;

    let substituted = substitute_env_vars(toml_content).unwrap();
    let config: Config = toml::from_str(&substituted).unwrap();

    assert_eq!(config.services[0].connection.get_password(), "secret123");
    assert_eq!(
        config.services[0].connection.database,
        Some("testdb".to_string())
    );
}
