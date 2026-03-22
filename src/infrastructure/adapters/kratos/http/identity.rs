use crate::domain::errors::{AuthError, DomainError};
use crate::domain::ports::outbound::identity::IdentityPort;
use crate::infrastructure::adapters::cache::redis_cache::RedisCache;
use crate::infrastructure::adapters::kratos::client::KratosClient;
use crate::infrastructure::adapters::kratos::models::identity::SessionResponse;
use async_trait::async_trait;
use reqwest::header;
use std::sync::Arc;

pub struct KratosIdentityAdapter {
    client: Arc<KratosClient>,
    cache: Option<RedisCache>,
    cache_ttl_secs: u64,
}

impl KratosIdentityAdapter {
    pub fn new(client: Arc<KratosClient>, cache: Option<RedisCache>, cache_ttl_secs: u64) -> Self {
        Self {
            client,
            cache,
            cache_ttl_secs,
        }
    }
}

#[async_trait]
impl IdentityPort for KratosIdentityAdapter {
    async fn get_current_user(
        &self,
        cookie: &str,
    ) -> Result<crate::domain::entities::user_profile::UserProfile, DomainError> {
        let session_token = cookie
            .split(';')
            .find(|s| s.trim().starts_with("ory_kratos_session="))
            .and_then(|s| s.trim().strip_prefix("ory_kratos_session="))
            .unwrap_or(cookie)
            .to_string();

        let cache_key = format!("user_profile:{}", session_token);

        if let Some(cache) = &self.cache {
            if let Some(cached) = cache.get(&cache_key).await {
                if let Ok(profile) = serde_json::from_str(&cached) {
                    return Ok(profile);
                }
            }
        }

        let url =
            format!("{}/sessions/whoami", self.client.public_url).replace("localhost", "127.0.0.1");

        let response = self
            .client
            .client
            .get(&url)
            .header(header::COOKIE, cookie)
            .send()
            .await
            .map_err(|e| DomainError::ServiceUnavailable(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AuthError::NotAuthenticated.into());
        }

        let session: SessionResponse = response
            .json()
            .await
            .map_err(|e| DomainError::ServiceUnavailable(e.to_string()))?;

        let profile: crate::domain::entities::user_profile::UserProfile =
            (session.identity, session.active, session.expires_at).into();

        if let Some(cache) = &self.cache {
            if let Ok(serialized) = serde_json::to_string(&profile) {
                cache
                    .set_ex(&cache_key, &serialized, self.cache_ttl_secs)
                    .await;
            }
        }

        Ok(profile)
    }
}
