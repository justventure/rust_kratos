use async_trait::async_trait;

use crate::domain::errors::DomainError;
use crate::domain::value_objects::email::Email;
use crate::domain::value_objects::password::Password;

#[derive(Debug, Clone)]
pub struct RegistrationData {
    pub email: Email,
    pub username: String,
    pub password: Password,
    pub geo_location: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RegistrationFlowData {
    pub flow_id: String,
    pub csrf_token: String,
    pub cookies: Vec<String>,
}

#[async_trait]
pub trait RegistrationPort: Send + Sync {
    async fn initiate_registration(&self, cookie: Option<&str>) -> Result<RegistrationFlowData, DomainError>;
    async fn complete_registration(
        &self,
        flow: RegistrationFlowData,
        data: RegistrationData,
    ) -> Result<String, DomainError>;
}
