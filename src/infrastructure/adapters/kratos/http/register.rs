use std::sync::Arc;

use async_trait::async_trait;
use reqwest::StatusCode;
use tracing::{error, instrument};

use crate::domain::errors::{AuthError, DomainError};
use crate::domain::ports::inbound::registration::{RegistrationData, RegistrationPort};
use crate::domain::ports::outbound::session::SessionPort;
use crate::domain::value_objects::session_cookie::SessionCookie;
use crate::infrastructure::adapters::kratos::client::KratosClient;
use crate::infrastructure::adapters::kratos::http::flows::{fetch_flow, post_flow};
use crate::infrastructure::adapters::kratos::models::errors::KratosFlowError;
use crate::infrastructure::adapters::kratos::models::flows::{FlowResult, PostFlowResult};
use crate::infrastructure::adapters::kratos::models::registration::RegistrationPayload;

pub struct KratosRegistrationAdapter {
    client: Arc<KratosClient>,
    session: Arc<dyn SessionPort>,
}

impl KratosRegistrationAdapter {
    pub fn new(client: Arc<KratosClient>, session: Arc<dyn SessionPort>) -> Self {
        Self { client, session }
    }
}

fn map_registration_error(e: KratosFlowError) -> DomainError {
    if e.is_browser_location_change_required() {
        return DomainError::ServiceUnavailable("Browser location change required".into());
    }
    match (e.status, e.message_id()) {
        (StatusCode::BAD_REQUEST, 4000007) => DomainError::Conflict("Email already exists".into()),
        (StatusCode::BAD_REQUEST, 4000010) => DomainError::InvalidData("Password is too weak".into()),
        (StatusCode::BAD_REQUEST, _) => DomainError::InvalidData(e.message_text().into()),
        (StatusCode::GONE, _) => DomainError::NotFound("registration flow".into()),
        (StatusCode::TOO_MANY_REQUESTS, _) => AuthError::TooManyAttempts.into(),
        (StatusCode::UNPROCESSABLE_ENTITY, _) => DomainError::InvalidData(e.message_text().into()),
        _ => DomainError::ServiceUnavailable(e.to_string()),
    }
}

#[async_trait]
impl RegistrationPort for KratosRegistrationAdapter {
    #[instrument(skip(self, cookie), name = "kratos.initiate_registration")]
    async fn initiate_registration(&self, cookie: Option<&str>) -> Result<String, DomainError> {
        let is_active = self.check_session(cookie).await;
        if is_active {
            error!("Registration attempt with an already active session");
            return Err(AuthError::AlreadyLoggedIn.into());
        }
        let flow = self.fetch_registration_flow(cookie).await?;
        Ok(flow.flow_id.as_str().to_string())
    }

    #[instrument(skip(self, data), name = "kratos.complete_registration")]
    async fn complete_registration(&self, _flow_id: &str, data: RegistrationData) -> Result<String, DomainError> {
        let flow = self.fetch_registration_flow(None).await?;
        let payload = RegistrationPayload::from_data(data, flow.csrf_token.clone());
        let result = self.post_registration_flow(&flow, payload).await?;
        SessionCookie::find_in(result.cookies)
            .map(|c| c.as_str().to_string())
            .ok_or_else(|| {
                error!("Session cookie not found after registration");
                DomainError::Internal("No session cookie was created".into())
            })
    }
}

impl KratosRegistrationAdapter {
    #[instrument(skip(self, cookie), name = "kratos.check_session")]
    async fn check_session(&self, cookie: Option<&str>) -> bool {
        self.session.check_active_session(cookie).await
    }

    #[instrument(skip(self, cookie), name = "kratos.fetch_registration_flow")]
    async fn fetch_registration_flow(&self, cookie: Option<&str>) -> Result<FlowResult, DomainError> {
        fetch_flow(&self.client.client, &self.client.public_url, "registration", cookie)
            .await
            .map_err(map_registration_error)
    }

    #[instrument(skip(self, flow, payload), name = "kratos.post_registration_flow", fields(flow_id = %flow.flow_id.as_str()))]
    async fn post_registration_flow(
        &self,
        flow: &FlowResult,
        payload: RegistrationPayload,
    ) -> Result<PostFlowResult, DomainError> {
        post_flow(
            &self.client.client,
            &self.client.public_url,
            "registration",
            &flow.flow_id,
            serde_json::to_value(payload).map_err(|e| DomainError::InvalidData(e.to_string()))?,
            &flow.cookies,
        )
        .await
        .map_err(map_registration_error)
    }
}
