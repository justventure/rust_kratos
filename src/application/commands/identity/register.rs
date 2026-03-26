use std::sync::Arc;

use async_trait::async_trait;
use tracing::instrument;

use crate::application::commands::CommandHandler;
use crate::domain::entities::user_profile::UserProfile;
use crate::domain::errors::DomainError;
use crate::domain::ports::inbound::registration::{RegistrationData, RegistrationFlowData, RegistrationPort};

pub struct RegisterCommand {
    pub data: RegistrationData,
    pub cookie: Option<String>,
}

pub struct RegisterCommandResult {
    pub flow_id: String,
    pub session_cookie: String,
    pub user: UserProfile,
}

pub struct RegisterCommandHandler {
    registration_port: Arc<dyn RegistrationPort>,
}

impl RegisterCommandHandler {
    pub fn new(registration_port: Arc<dyn RegistrationPort>) -> Self {
        Self { registration_port }
    }
}

#[async_trait]
impl CommandHandler<RegisterCommand, RegisterCommandResult> for RegisterCommandHandler {
    #[instrument(skip(self, command), name = "command.register")]
    async fn handle(&self, command: RegisterCommand) -> Result<RegisterCommandResult, DomainError> {
        let flow = self.initiate(command.cookie.as_deref()).await?;
        let flow_id = flow.flow_id.clone();
        let result = self.complete(flow, command.data).await?;
        Ok(RegisterCommandResult {
            flow_id,
            session_cookie: result.session_cookie,
            user: result.user,
        })
    }
}

impl RegisterCommandHandler {
    #[instrument(skip(self), name = "registration.initiate")]
    async fn initiate(&self, cookie: Option<&str>) -> Result<RegistrationFlowData, DomainError> {
        self.registration_port.initiate_registration(cookie).await
    }

    #[instrument(skip(self, flow, data), name = "registration.complete")]
    async fn complete(
        &self,
        flow: RegistrationFlowData,
        data: RegistrationData,
    ) -> Result<crate::domain::ports::inbound::registration::RegistrationResult, DomainError> {
        self.registration_port.complete_registration(flow, data).await
    }
}
