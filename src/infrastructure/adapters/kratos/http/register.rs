use std::sync::Arc;

use async_trait::async_trait;
use reqwest::StatusCode;
use tracing::{error, instrument};

use crate::domain::entities::user_profile::UserProfile;
use crate::domain::errors::{AuthError, DomainError};
use crate::domain::ports::inbound::registration::{
    RegistrationData, RegistrationFlowData, RegistrationPort, RegistrationResult,
};
use crate::domain::ports::outbound::session::SessionPort;
use crate::domain::value_objects::flow_id::FlowId;
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
    async fn initiate_registration(&self, cookie: Option<&str>) -> Result<RegistrationFlowData, DomainError> {
        if self.check_session(cookie).await {
            error!("Registration attempt with an already active session");
            return Err(AuthError::AlreadyLoggedIn.into());
        }
        let flow = self.fetch_registration_flow(cookie).await?;
        Ok(RegistrationFlowData {
            flow_id: flow.flow_id.as_str().to_string(),
            csrf_token: flow.csrf_token,
            cookies: flow.cookies,
        })
    }

    #[instrument(skip(self, flow, data), name = "kratos.complete_registration")]
    async fn complete_registration(
        &self,
        flow: RegistrationFlowData,
        data: RegistrationData,
    ) -> Result<RegistrationResult, DomainError> {
        let flow_result = FlowResult {
            flow_id: FlowId::new(&flow.flow_id),
            csrf_token: flow.csrf_token.clone(),
            cookies: flow.cookies,
        };
        let payload = RegistrationPayload::from_data(data, flow.csrf_token);
        let result = self.post_registration_flow(&flow_result, payload).await?;

        let session_cookie = SessionCookie::find_in(result.cookies)
            .map(|c| c.as_str().to_string())
            .ok_or_else(|| {
                error!("Session cookie not found after registration");
                DomainError::Internal("No session cookie was created".into())
            })?;

        let identity = &result.data["identity"];
        let traits = &identity["traits"];

        let user = UserProfile {
            id: identity["id"].as_str().unwrap_or_default().to_string(),
            traits: crate::domain::entities::user_profile::UserTraits {
                email: traits["email"].as_str().unwrap_or_default().to_string(),
                username: traits["username"].as_str().map(|s| s.to_string()),
                geo_location: traits["geo_location"].as_str().map(|s| s.to_string()),
            },
            created_at: serde_json::from_value(identity["created_at"].clone()).ok(),
            updated_at: serde_json::from_value(identity["updated_at"].clone()).ok(),
            state: identity["state"].as_str().map(|s| s.to_string()),
        };

        Ok(RegistrationResult { session_cookie, user })
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
