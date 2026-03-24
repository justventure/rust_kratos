use std::sync::Arc;

use async_trait::async_trait;
use reqwest::StatusCode;
use tracing::{debug, error, instrument};

use crate::domain::errors::{AuthError, DomainError};
use crate::domain::ports::inbound::login::{AuthenticationPort, LoginCredentials, LoginFlowData};
use crate::domain::ports::outbound::session::SessionPort;
use crate::domain::value_objects::flow_id::FlowId;
use crate::domain::value_objects::session_cookie::SessionCookie;
use crate::infrastructure::adapters::kratos::client::KratosClient;
use crate::infrastructure::adapters::kratos::http::flows::{fetch_flow, post_flow};
use crate::infrastructure::adapters::kratos::models::errors::KratosFlowError;
use crate::infrastructure::adapters::kratos::models::login::LoginPayload;

pub struct KratosAuthenticationAdapter {
    client: Arc<KratosClient>,
    session: Arc<dyn SessionPort>,
}

impl KratosAuthenticationAdapter {
    pub fn new(client: Arc<KratosClient>, session: Arc<dyn SessionPort>) -> Self {
        Self { client, session }
    }
}

fn map_login_error(e: KratosFlowError) -> DomainError {
    error!("Failed to post login flow: {}", e);
    if e.is_browser_location_change_required() {
        return DomainError::ServiceUnavailable("Browser location change required".into());
    }
    match (e.status, e.message_id()) {
        (StatusCode::BAD_REQUEST, 4000006 | 4000010) => AuthError::InvalidCredentials.into(),
        (StatusCode::BAD_REQUEST, _) => DomainError::InvalidData(e.message_text().into()),
        (StatusCode::GONE, _) => DomainError::NotFound("login flow".into()),
        (StatusCode::UNAUTHORIZED, _) => AuthError::NotAuthenticated.into(),
        (StatusCode::TOO_MANY_REQUESTS, _) => AuthError::TooManyAttempts.into(),
        (StatusCode::UNPROCESSABLE_ENTITY, _) => DomainError::InvalidData(e.message_text().into()),
        _ => DomainError::ServiceUnavailable(e.to_string()),
    }
}

#[async_trait]
impl AuthenticationPort for KratosAuthenticationAdapter {
    #[instrument(skip_all, name = "kratos.initiate_login")]
    async fn initiate_login(&self, cookie: Option<&str>) -> Result<LoginFlowData, DomainError> {
        let (is_active, is_recovery) = tokio::join!(
            self.session.check_active_session(cookie),
            self.session.is_recovery_session(cookie),
        );
        if is_active && !is_recovery {
            error!("Login attempt with an already active session");
            return Err(AuthError::AlreadyLoggedIn.into());
        }
        let flow = fetch_flow(&self.client.client, &self.client.public_url, "login", None)
            .await
            .map_err(map_login_error)?;
        Ok(LoginFlowData {
            flow_id: flow.flow_id.as_str().to_string(),
            csrf_token: flow.csrf_token,
            cookies: flow.cookies,
        })
    }

    #[instrument(skip_all, name = "kratos.complete_login")]
    async fn complete_login(&self, flow: LoginFlowData, credentials: LoginCredentials) -> Result<String, DomainError> {
        let payload = LoginPayload::from_credentials(credentials, flow.csrf_token.clone());
        debug!(
            "Login payload: {}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
        let flow_id = FlowId::new(&flow.flow_id);
        let result = post_flow(
            &self.client.client,
            &self.client.public_url,
            "login",
            &flow_id,
            serde_json::to_value(payload).map_err(|e| DomainError::InvalidData(e.to_string()))?,
            &flow.cookies,
        )
        .await
        .map_err(map_login_error)?;
        debug!("Received cookies: {:?}", result.cookies);
        debug!("Response data: {:?}", result.data);
        SessionCookie::find_in(result.cookies)
            .map(|c| c.as_str().to_string())
            .ok_or_else(|| {
                error!("Session cookie not found in response cookies");
                DomainError::Internal("Session token not found".into())
            })
    }
}
