use std::sync::Arc;

use async_trait::async_trait;
use tracing::instrument;

use crate::application::commands::CommandHandler;
use crate::domain::errors::DomainError;
use crate::domain::ports::inbound::registration::{RegistrationData, RegistrationPort};

pub struct RegisterCommand {
    pub data: RegistrationData,
    pub cookie: Option<String>,
}

pub struct RegisterCommandResult {
    pub flow_id: String,
    pub session_cookie: String,
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
        let flow_id = self.initiate(command.cookie.as_deref()).await?;
        let session_cookie = self.complete(&flow_id, command.data).await?;
        Ok(RegisterCommandResult {
            flow_id,
            session_cookie,
        })
    }
}

impl RegisterCommandHandler {
    #[instrument(skip(self), name = "registration.initiate")]
    async fn initiate(&self, cookie: Option<&str>) -> Result<String, DomainError> {
        self.registration_port.initiate_registration(cookie).await
    }

    #[instrument(skip(self, data), name = "registration.complete")]
    async fn complete(&self, flow_id: &str, data: RegistrationData) -> Result<String, DomainError> {
        self.registration_port.complete_registration(flow_id, data).await
    }
}
