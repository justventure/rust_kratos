use async_trait::async_trait;

use crate::domain::errors::DomainError;
use crate::domain::value_objects::email::Email;

#[derive(Debug, Clone)]
pub struct RecoveryRequest {
    pub email: Email,
}

#[async_trait]
pub trait RecoveryPort: Send + Sync {
    async fn initiate_recovery(&self, request: RecoveryRequest, cookie: Option<&str>) -> Result<(), DomainError>;
}
