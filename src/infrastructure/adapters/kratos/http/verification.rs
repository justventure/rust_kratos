use std::sync::Arc;

use async_trait::async_trait;
use reqwest::StatusCode;
use tracing::instrument;

use crate::domain::errors::{AuthError, DomainError};
use crate::domain::ports::inbound::verification::{
    SendCodeRequest, SubmitCodeRequest, VerificationPort, VerifyByLinkRequest,
};
use crate::domain::value_objects::auth_method::AuthMethod;
use crate::infrastructure::adapters::kratos::client::KratosClient;
use crate::infrastructure::adapters::kratos::http::flows::{fetch_flow, post_flow};
use crate::infrastructure::adapters::kratos::models::errors::KratosFlowError;
use crate::infrastructure::adapters::kratos::models::verification::VerificationPayload;

pub struct KratosVerificationAdapter {
    client: Arc<KratosClient>,
}

impl KratosVerificationAdapter {
    pub fn new(client: Arc<KratosClient>) -> Self {
        Self { client }
    }
}

fn map_verification_error(e: KratosFlowError) -> DomainError {
    if e.is_browser_location_change_required() {
        return DomainError::ServiceUnavailable("Browser location change required".into());
    }
    match (e.status, e.message_id()) {
        (StatusCode::BAD_REQUEST, 4070006) => DomainError::InvalidData("Invalid verification code".into()),
        (StatusCode::BAD_REQUEST, 4070001) => DomainError::InvalidData("Invalid email address".into()),
        (StatusCode::BAD_REQUEST, _) => DomainError::InvalidData(e.message_text().into()),
        (StatusCode::GONE, _) => DomainError::NotFound("verification flow".into()),
        (StatusCode::UNAUTHORIZED, _) => AuthError::NotAuthenticated.into(),
        (StatusCode::TOO_MANY_REQUESTS, _) => AuthError::TooManyAttempts.into(),
        (StatusCode::UNPROCESSABLE_ENTITY, _) => DomainError::InvalidData(e.message_text().into()),
        _ => DomainError::ServiceUnavailable(e.to_string()),
    }
}

#[instrument(skip_all, name = "kratos.execute_verification_flow")]
async fn execute_verification_flow(
    client: &KratosClient,
    method: AuthMethod,
    email: Option<String>,
    code: Option<String>,
    transient_payload: Option<serde_json::Value>,
    cookie: Option<&str>,
) -> Result<(), DomainError> {
    let flow = fetch_flow(&client.client, &client.public_url, "verification", cookie)
        .await
        .map_err(map_verification_error)?;
    let payload = VerificationPayload::new(method, email, code, flow.csrf_token.clone(), transient_payload);
    post_flow(
        &client.client,
        &client.public_url,
        "verification",
        &flow.flow_id,
        serde_json::to_value(payload).map_err(|e| DomainError::InvalidData(e.to_string()))?,
        &flow.cookies,
    )
    .await
    .map_err(map_verification_error)?;
    Ok(())
}

#[async_trait]
impl VerificationPort for KratosVerificationAdapter {
    #[instrument(skip_all, name = "kratos.verify_by_link")]
    async fn verify_by_link(&self, request: VerifyByLinkRequest, cookie: Option<&str>) -> Result<(), DomainError> {
        execute_verification_flow(
            &self.client,
            AuthMethod::Link,
            Some(request.email.as_str().to_string()),
            None,
            request.transient_payload,
            cookie,
        )
        .await
    }

    #[instrument(skip_all, name = "kratos.send_verification_code")]
    async fn send_verification_code(&self, request: SendCodeRequest, cookie: Option<&str>) -> Result<(), DomainError> {
        execute_verification_flow(
            &self.client,
            AuthMethod::Code,
            Some(request.email.as_str().to_string()),
            None,
            request.transient_payload,
            cookie,
        )
        .await
    }

    #[instrument(skip_all, name = "kratos.submit_verification_code")]
    async fn submit_verification_code(&self, request: SubmitCodeRequest, cookie: &str) -> Result<(), DomainError> {
        execute_verification_flow(
            &self.client,
            AuthMethod::Code,
            None,
            Some(request.code),
            request.transient_payload,
            Some(cookie),
        )
        .await
    }
}
