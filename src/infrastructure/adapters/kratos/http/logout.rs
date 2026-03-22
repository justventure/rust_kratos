use crate::domain::errors::{AuthError, DomainError};
use crate::domain::ports::outbound::identity::IdentityPort;
use crate::domain::ports::outbound::session::SessionPort;
use crate::infrastructure::adapters::cache::redis_cache::RedisCache;
use crate::infrastructure::adapters::kratos::client::KratosClient;
use crate::infrastructure::adapters::kratos::http::identity::KratosIdentityAdapter;
use async_trait::async_trait;
use reqwest::{StatusCode, header};
use std::sync::Arc;

pub struct KratosSessionAdapter {
    client: Arc<KratosClient>,
    identity_adapter: KratosIdentityAdapter,
    cache: Option<RedisCache>,
}

impl KratosSessionAdapter {
    pub fn new(client: Arc<KratosClient>, cache: Option<RedisCache>) -> Self {
        let identity_adapter = KratosIdentityAdapter::new(client.clone(), None, 0);
        Self {
            client,
            identity_adapter,
            cache,
        }
    }

    async fn get_logout_flow(&self, cookie: &str) -> Result<String, DomainError> {
        let url = format!("{}/self-service/logout/browser", self.client.public_url);

        let response = self
            .client
            .client
            .get(&url)
            .header(header::COOKIE, cookie)
            .send()
            .await
            .map_err(|e| DomainError::ServiceUnavailable(e.to_string()))?;

        match response.status() {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                return Err(AuthError::NotAuthenticated.into());
            }
            StatusCode::TOO_MANY_REQUESTS => {
                return Err(DomainError::ServiceUnavailable(
                    "Rate limit exceeded".into(),
                ));
            }
            s if !s.is_success() => {
                return Err(DomainError::ServiceUnavailable(format!(
                    "Failed to get logout flow: {}",
                    s
                )));
            }
            _ => {}
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| DomainError::ServiceUnavailable(e.to_string()))?;

        data["logout_url"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| DomainError::InvalidData("Logout URL not found".into()))
    }

    fn extract_session_token(cookie: &str) -> Option<String> {
        cookie
            .split(';')
            .find(|s| s.trim().starts_with("ory_kratos_session="))
            .and_then(|s| s.trim().strip_prefix("ory_kratos_session="))
            .map(|s| s.to_string())
    }
}

#[async_trait]
impl SessionPort for KratosSessionAdapter {
    async fn logout(&self, cookie: &str) -> Result<(), DomainError> {
        let logout_url = self.get_logout_flow(cookie).await?;

        let response = self
            .client
            .client
            .get(&logout_url)
            .header(header::COOKIE, cookie)
            .send()
            .await
            .map_err(|e| DomainError::ServiceUnavailable(e.to_string()))?;

        let result = match response.status() {
            s if s.is_success() || s == StatusCode::FOUND || s == StatusCode::SEE_OTHER => Ok(()),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Err(AuthError::NotAuthenticated.into())
            }
            s => {
                let error_text = response.text().await.unwrap_or_else(|_| s.to_string());
                Err(DomainError::ServiceUnavailable(format!(
                    "Logout failed: {}",
                    error_text
                )))
            }
        };

        if result.is_ok() {
            if let Some(cache) = &self.cache {
                if let Some(token) = Self::extract_session_token(cookie) {
                    cache.delete(&format!("user_profile:{}", token)).await;
                }
            }
        }

        result
    }

    async fn check_active_session(&self, cookie: Option<&str>) -> bool {
        let Some(cookie_value) = cookie else {
            return false;
        };
        self.identity_adapter
            .get_current_user(cookie_value)
            .await
            .is_ok()
    }

    async fn is_recovery_session(&self, cookie: Option<&str>) -> bool {
        let Some(cookie) = cookie else { return false };

        let url = format!("{}/sessions/whoami", self.client.public_url);

        let Ok(response) = self
            .client
            .client
            .get(&url)
            .header(header::COOKIE, cookie)
            .send()
            .await
        else {
            return false;
        };

        let Ok(data) = response.json::<serde_json::Value>().await else {
            return false;
        };

        data["authentication_methods"]
            .as_array()
            .map(|methods| {
                methods.iter().any(|m| {
                    m["method"].as_str() == Some("link_recovery")
                        || m["method"].as_str() == Some("code_recovery")
                })
            })
            .unwrap_or(false)
    }
}
