use crate::domain::errors::{AuthError, DomainError};
use crate::domain::ports::inbound::settings::{SettingsData, SettingsPort};
use crate::infrastructure::adapters::kratos::client::KratosClient;
use crate::infrastructure::adapters::kratos::http::flows::{fetch_flow, post_flow};
use crate::infrastructure::adapters::kratos::models::errors::KratosFlowError;
use crate::infrastructure::adapters::kratos::models::settings::SettingsPayload;
use async_trait::async_trait;
use reqwest::StatusCode;
use std::sync::Arc;
use tracing::debug;

pub struct KratosSettingsAdapter {
    client: Arc<KratosClient>,
}

impl KratosSettingsAdapter {
    pub fn new(client: Arc<KratosClient>) -> Self {
        Self { client }
    }
}

fn map_settings_error(e: KratosFlowError) -> DomainError {
    match (e.status, e.message_id()) {
        (StatusCode::FORBIDDEN, _) => AuthError::PrivilegedSessionRequired.into(),
        (StatusCode::UNAUTHORIZED, _) => AuthError::NotAuthenticated.into(),
        (StatusCode::GONE, _) => DomainError::NotFound("settings flow".into()),
        (StatusCode::BAD_REQUEST, 4000010) => {
            DomainError::InvalidData("Password is too weak".into())
        }
        (StatusCode::BAD_REQUEST, _) => DomainError::InvalidData(e.message_text().into()),
        _ => DomainError::ServiceUnavailable(e.to_string()),
    }
}

#[async_trait]
impl SettingsPort for KratosSettingsAdapter {
    async fn initiate_settings(&self, cookie: &str) -> Result<String, DomainError> {
        let flow = fetch_flow(
            &self.client.client,
            &self.client.public_url,
            "settings",
            Some(cookie),
        )
        .await
        .map_err(|e| DomainError::ServiceUnavailable(e.to_string()))?;

        Ok(flow.flow_id.as_str().to_string())
    }

    async fn update_settings(
        &self,
        _flow_id: &str,
        data: SettingsData,
        cookie: &str,
    ) -> Result<(String, Vec<String>), DomainError> {
        let flow = fetch_flow(
            &self.client.client,
            &self.client.public_url,
            "settings",
            Some(cookie),
        )
        .await
        .map_err(|e| DomainError::ServiceUnavailable(e.to_string()))?;

        let payload = SettingsPayload::from_data(data, flow.csrf_token.clone());

        debug!(
            "Settings payload: {}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );

        let result = post_flow(
            &self.client.client,
            &self.client.public_url,
            "settings",
            &flow.flow_id,
            serde_json::to_value(payload).map_err(|e| DomainError::InvalidData(e.to_string()))?,
            &flow.cookies,
        )
        .await
        .map_err(map_settings_error)?;

        debug!("Settings response: {:?}", result.data);
        debug!("Settings response cookies: {:?}", result.cookies);

        let state = result
            .data
            .get("state")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| DomainError::ServiceUnavailable("No state in response".into()))?;

        Ok((state, result.cookies))
    }
}
