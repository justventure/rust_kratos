use std::sync::Arc;

use async_trait::async_trait;
use tracing::instrument;

use crate::application::commands::CommandHandler;
use crate::domain::entities::user_profile::UserProfile;
use crate::domain::errors::DomainError;
use crate::domain::ports::inbound::login::{AuthenticationPort, LoginCredentials};

pub struct LoginCommand {
    pub credentials: LoginCredentials,
    pub cookie: Option<String>,
}

pub struct LoginCommandResult {
    pub session_cookie: String,
    pub user: UserProfile,
}

pub struct LoginCommandHandler {
    auth_port: Arc<dyn AuthenticationPort>,
}

impl LoginCommandHandler {
    pub fn new(auth_port: Arc<dyn AuthenticationPort>) -> Self {
        Self { auth_port }
    }
}

#[async_trait]
impl CommandHandler<LoginCommand, LoginCommandResult> for LoginCommandHandler {
    #[instrument(skip_all, name = "command.login")]
    async fn handle(&self, command: LoginCommand) -> Result<LoginCommandResult, DomainError> {
        let flow = self.auth_port.initiate_login(command.cookie.as_deref()).await?;
        let result = self.auth_port.complete_login(flow, command.credentials).await?;
        Ok(LoginCommandResult {
            session_cookie: result.session_cookie,
            user: result.user,
        })
    }
}
