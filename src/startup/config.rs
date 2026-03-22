use std::env;

use serde::Deserialize;
use tracing::info;

#[derive(Debug, Clone, Copy)]
pub enum Environment {
    Development,
    Production,
    DockerLocal,
}

impl Environment {
    pub fn from_env() -> Self {
        match env::var("APP_ENV")
            .unwrap_or_else(|_| String::from("development"))
            .to_lowercase()
            .as_str()
        {
            "production" => Environment::Production,
            "docker_local" => Environment::DockerLocal,
            _ => Environment::Development,
        }
    }

    pub fn config_filename(&self) -> &str {
        match self {
            Environment::Development => "development",
            Environment::Production => "production",
            Environment::DockerLocal => "docker_local",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub kratos: KratosConfig,
    pub server: ServerConfig,
    pub redis: RedisConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    #[serde(default = "defaults::cache_ttl")]
    pub cache_ttl_secs: u64,
    #[serde(default = "defaults::redis_max_retries")]
    pub max_retries: u32,
    #[serde(default = "defaults::redis_retry_delay")]
    pub retry_delay_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KratosConfig {
    pub admin_url: String,
    pub public_url: String,
    #[serde(default = "defaults::timeout")]
    pub timeout_secs: u64,
    #[serde(default = "defaults::connect_timeout")]
    pub connect_timeout_secs: u64,
    #[serde(default = "defaults::pool_idle_timeout")]
    pub pool_idle_timeout_secs: u64,
    #[serde(default = "defaults::pool_max_idle")]
    pub pool_max_idle_per_host: usize,
    #[serde(default = "defaults::max_retries")]
    pub max_retries: u32,
    #[serde(default = "defaults::retry_delay")]
    pub retry_delay_ms: u64,
    #[serde(default = "defaults::accept_invalid_certs")]
    pub accept_invalid_certs: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    #[serde(default = "defaults::log_level")]
    pub log_level: String,
    #[serde(default = "defaults::cors_max_age")]
    pub cors_max_age: usize,
    #[serde(default = "defaults::cors_allowed_origins")]
    pub cors_allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        dotenvy::dotenv().ok();

        let environment = Environment::from_env();
        let config_filename = environment.config_filename();
        let config_path = format!("config/app/{}", config_filename);

        info!(path = %format!("{}.toml", config_path), "Loading config file");

        let builder = config::Config::builder()
            .add_source(
                config::File::with_name(&config_path)
                    .required(true)
                    .format(config::FileFormat::Toml),
            )
            .add_source(
                config::Environment::with_prefix("APP")
                    .separator("__")
                    .try_parsing(true),
            );

        builder.build()?.try_deserialize()
    }
}

#[rustfmt::skip]
mod defaults {
    pub fn cache_ttl() -> u64 { 300 }
    pub fn redis_max_retries() -> u32 { 5 }
    pub fn redis_retry_delay() -> u64 { 2000 }
    pub fn timeout() -> u64 { 120 }
    pub fn connect_timeout() -> u64 { 30 }
    pub fn pool_idle_timeout() -> u64 { 120 }
    pub fn pool_max_idle() -> usize { 10 }
    pub fn max_retries() -> u32 { 3 }
    pub fn retry_delay() -> u64 { 1000 }
    pub fn accept_invalid_certs() -> bool { false }
    pub fn log_level() -> String { "info".to_string() }
    pub fn cors_max_age() -> usize { 3600 }
    pub fn cors_allowed_origins() -> Vec<String> { vec!["http://localhost:3000".to_string()] }
}
