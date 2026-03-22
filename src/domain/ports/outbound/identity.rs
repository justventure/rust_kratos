use async_trait::async_trait;

use crate::domain::entities::user_profile::UserProfile;
use crate::domain::errors::DomainError;

#[async_trait]
pub trait IdentityPort: Send + Sync {
    async fn get_current_user(&self, cookie: &str) -> Result<UserProfile, DomainError>;
}
