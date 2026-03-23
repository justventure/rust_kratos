use std::sync::Arc;

use async_trait::async_trait;
use reqwest::StatusCode;
use tracing::debug;

use crate::domain::errors::{AuthError, DomainError};
use crate::domain::ports::inbound::recovery::{RecoveryPort, RecoveryRequest};
use crate::infrastructure::adapters::kratos::client::KratosClient;
use crate::infrastructure::adapters::kratos::http::flows::{fetch_flow, post_flow};
use crate::infrastructure::adapters::kratos::models::errors::KratosFlowError;
use crate::infrastructure::adapters::kratos::models::recovery::RecoveryPayload;

pub struct KratosRecoveryAdapter {
    client: Arc<KratosClient>,
}

impl KratosRecoveryAdapter {
    pub fn new(client: Arc<KratosClient>) -> Self {
        Self { client }
    }
}

fn map_recovery_error(e: KratosFlowError) -> DomainError {
    if e.is_browser_location_change_required() {
        return DomainError::ServiceUnavailable("Browser location change required".into());
    }
    match (e.status, e.message_id()) {
        (StatusCode::BAD_REQUEST, 4060001) => DomainError::InvalidData("Invalid email address".into()),
        (StatusCode::BAD_REQUEST, _) => DomainError::InvalidData(e.message_text().into()),
        (StatusCode::GONE, _) => DomainError::NotFound("recovery flow".into()),
        (StatusCode::UNAUTHORIZED, _) => AuthError::NotAuthenticated.into(),
        (StatusCode::TOO_MANY_REQUESTS, _) => AuthError::TooManyAttempts.into(),
        (StatusCode::UNPROCESSABLE_ENTITY, _) => DomainError::InvalidData(e.message_text().into()),
        _ => DomainError::ServiceUnavailable(e.to_string()),
    }
}

#[async_trait]
impl RecoveryPort for KratosRecoveryAdapter {
    async fn initiate_recovery(&self, request: RecoveryRequest, cookie: Option<&str>) -> Result<(), DomainError> {
        let flow = fetch_flow(&self.client.client, &self.client.public_url, "recovery", cookie)
            .await
            .map_err(map_recovery_error)?;
        let payload = RecoveryPayload::new(request.email.as_str(), flow.csrf_token.clone());
        let result = post_flow(
            &self.client.client,
            &self.client.public_url,
            "recovery",
            &flow.flow_id,
            serde_json::to_value(payload).map_err(|e| DomainError::InvalidData(e.to_string()))?,
            &flow.cookies,
        )
        .await
        .map_err(map_recovery_error)?;
        debug!(
            cookies_count = result.cookies.len(),
            cookies = ?result.cookies,
            "Cookies returned from Kratos"
        );
        Ok(())
    }
}
