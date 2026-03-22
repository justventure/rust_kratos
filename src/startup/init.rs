use std::sync::Arc;

use tokio::signal;
use tokio::signal::unix::SignalKind;
use tracing::{info, warn};

use crate::infrastructure::adapters::cache::redis_cache::RedisCacheConfig;
use crate::infrastructure::adapters::http::server::{self, HttpServerConfig};
use crate::infrastructure::adapters::kratos::client::KratosClientConfig;
use crate::infrastructure::di::container::{AppContainer, ContainerConfig};
use crate::startup::config::Config;
use crate::startup::tracing::TracingHandle;

pub async fn run() -> anyhow::Result<()> {
    let tracing = TracingHandle::init()?;
    let config = Config::from_env()?;
    tracing.set_level(&config.server.log_level)?;

    info!("Starting application...");

    validate_config(&config)?;

    let container_config = ContainerConfig {
        kratos: KratosClientConfig {
            admin_url: config.kratos.admin_url.clone(),
            public_url: config.kratos.public_url.clone(),
            timeout_secs: config.kratos.timeout_secs,
            connect_timeout_secs: config.kratos.connect_timeout_secs,
            pool_idle_timeout_secs: config.kratos.pool_idle_timeout_secs,
            pool_max_idle_per_host: config.kratos.pool_max_idle_per_host,
            max_retries: config.kratos.max_retries,
            retry_delay_ms: config.kratos.retry_delay_ms,
            accept_invalid_certs: config.kratos.accept_invalid_certs,
        },
        redis: RedisCacheConfig {
            url: config.redis.url.clone(),
            max_retries: config.redis.max_retries,
            retry_delay_ms: config.redis.retry_delay_ms,
            cache_ttl_secs: config.redis.cache_ttl_secs,
        },
    };

    let http_config = HttpServerConfig {
        host: config.server.host.clone(),
        port: config.server.port,
        cors_max_age: config.server.cors_max_age,
        cors_allowed_origins: config.server.cors_allowed_origins.clone(),
    };

    let container = Arc::new(AppContainer::new(container_config).await?);

    tokio::select! {
        result = server::start(http_config, container) => result?,
        _ = shutdown_signal() => {
            info!("Shutdown signal received, starting graceful shutdown...");
        }
    }

    Ok(())
}

fn validate_config(config: &Config) -> anyhow::Result<()> {
    if config.kratos.public_url.is_empty() {
        anyhow::bail!("Kratos public URL cannot be empty");
    }
    if config.redis.url.is_empty() {
        anyhow::bail!("Redis URL cannot be empty");
    }
    Ok(())
}

async fn shutdown_signal() {
    let mut sigint = signal::unix::signal(SignalKind::interrupt()).expect("Failed to install SIGINT handler");
    let mut sigterm = signal::unix::signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");
    let mut sigquit = signal::unix::signal(SignalKind::quit()).expect("Failed to install SIGQUIT handler");
    let mut sighup = signal::unix::signal(SignalKind::hangup()).expect("Failed to install SIGHUP handler");

    tokio::select! {
        _ = sigint.recv()  => warn!(signal = "SIGINT",  code = 2,  "Received shutdown signal"),
        _ = sigterm.recv() => info!(signal = "SIGTERM", code = 15, "Received shutdown signal"),
        _ = sigquit.recv() => warn!(signal = "SIGQUIT", code = 3,  "Received shutdown signal"),
        _ = sighup.recv()  => warn!(signal = "SIGHUP",  code = 1,  "Received shutdown signal"),
    }
}
