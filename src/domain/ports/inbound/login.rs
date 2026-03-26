use async_trait::async_trait;

use crate::domain::entities::user_profile::UserProfile;
use crate::domain::errors::DomainError;
use crate::domain::value_objects::email::Email;
use crate::domain::value_objects::password::Password;

#[derive(Debug, Clone)]
pub struct LoginCredentials {
    pub identifier: Email,
    pub password: Password,
    pub address: Option<String>,
    pub code: Option<String>,
    pub resend: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoginFlowData {
    pub flow_id: String,
    pub csrf_token: String,
    pub cookies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LoginResult {
    pub session_cookie: String,
    pub user: UserProfile,
}

#[async_trait]
pub trait AuthenticationPort: Send + Sync {
    async fn initiate_login(&self, cookie: Option<&str>) -> Result<LoginFlowData, DomainError>;
    async fn complete_login(
        &self,
        flow: LoginFlowData,
        credentials: LoginCredentials,
    ) -> Result<LoginResult, DomainError>;
}
