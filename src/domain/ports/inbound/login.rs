use crate::domain::errors::DomainError;
use crate::domain::value_objects::email::Email;
use crate::domain::value_objects::password::Password;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct LoginCredentials {
    pub identifier: Email,
    pub password: Password,
    pub address: Option<String>,
    pub code: Option<String>,
    pub resend: Option<String>,
}

#[async_trait]
pub trait AuthenticationPort: Send + Sync {
    async fn initiate_login(&self, cookie: Option<&str>) -> Result<String, DomainError>;
    async fn complete_login(
        &self,
        flow_id: &str,
        credentials: LoginCredentials,
    ) -> Result<String, DomainError>;
}
