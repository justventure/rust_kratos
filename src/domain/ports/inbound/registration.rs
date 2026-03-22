use crate::domain::errors::DomainError;
use crate::domain::value_objects::email::Email;
use crate::domain::value_objects::password::Password;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct RegistrationData {
    pub email: Email,
    pub username: String,
    pub password: Password,
    pub geo_location: Option<String>,
}

#[async_trait]
pub trait RegistrationPort: Send + Sync {
    async fn initiate_registration(&self, cookie: Option<&str>) -> Result<String, DomainError>;
    async fn complete_registration(
        &self,
        flow_id: &str,
        data: RegistrationData,
    ) -> Result<String, DomainError>;
}
