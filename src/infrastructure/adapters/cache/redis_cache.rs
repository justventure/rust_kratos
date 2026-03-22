use std::time::Duration;

use redis::AsyncCommands;

use crate::domain::errors::DomainError;

#[derive(Clone)]
pub struct RedisCacheConfig {
    pub url: String,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub cache_ttl_secs: u64,
    pub keep_alive_secs: u64,
}

#[derive(Clone)]
pub struct RedisCache {
    connection: redis::aio::ConnectionManager,
}

impl RedisCache {
    pub async fn new(config: &RedisCacheConfig) -> Result<Self, DomainError> {
        let client =
            redis::Client::open(config.url.as_str()).map_err(|e| DomainError::ServiceUnavailable(e.to_string()))?;

        let conn_config = redis::aio::ConnectionManagerConfig::new()
            .set_connection_timeout(Some(Duration::from_secs(config.keep_alive_secs)))
            .set_response_timeout(Some(Duration::from_secs(config.keep_alive_secs)));

        let connection = redis::aio::ConnectionManager::new_with_config(client, conn_config)
            .await
            .map_err(|e| DomainError::ServiceUnavailable(e.to_string()))?;

        Ok(Self { connection })
    }

    pub async fn new_with_retry(config: &RedisCacheConfig) -> Result<Self, DomainError> {
        let mut last_err: Option<DomainError> = None;
        let retry_delay = Duration::from_millis(config.retry_delay_ms);
        for attempt in 1..=config.max_retries {
            match Self::new(config).await {
                Ok(cache) => {
                    tracing::info!(attempt, "Redis connected");
                    return Ok(cache);
                }
                Err(e) => {
                    tracing::warn!(
                        attempt,
                        max = config.max_retries,
                        error = %e,
                        "Redis unavailable, retrying in {:?}", retry_delay
                    );
                    last_err = Some(e);
                    if attempt < config.max_retries {
                        tokio::time::sleep(retry_delay).await;
                    }
                }
            }
        }
        Err(last_err.expect("no error captured but Redis never connected"))
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let mut conn = self.connection.clone();
        conn.get::<_, String>(key).await.ok()
    }

    pub fn connection(&self) -> redis::aio::ConnectionManager {
        self.connection.clone()
    }

    pub async fn set_ex(&self, key: &str, value: &str, ttl_seconds: u64) {
        let mut conn = self.connection.clone();
        let _: Result<(), _> = conn.set_ex(key, value, ttl_seconds).await;
    }

    pub async fn delete(&self, key: &str) {
        let mut conn = self.connection.clone();
        let _: Result<(), _> = conn.del(key).await;
    }
}
