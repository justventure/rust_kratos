use std::sync::Arc;

use crate::domain::ports::inbound::login::AuthenticationPort;
use crate::domain::ports::inbound::recovery::RecoveryPort;
use crate::domain::ports::inbound::registration::RegistrationPort;
use crate::domain::ports::inbound::settings::SettingsPort;
use crate::domain::ports::inbound::verification::VerificationPort;
use crate::domain::ports::outbound::identity::IdentityPort;
use crate::domain::ports::outbound::session::SessionPort;

pub trait AdapterFactory: Send + Sync {
    fn create_registration_adapter(&self) -> Arc<dyn RegistrationPort>;
    fn create_authentication_adapter(&self) -> Arc<dyn AuthenticationPort>;
    fn create_session_adapter(&self) -> Arc<dyn SessionPort>;
    fn create_recovery_adapter(&self) -> Arc<dyn RecoveryPort>;
    fn create_verification_adapter(&self) -> Arc<dyn VerificationPort>;
    fn create_identity_adapter(&self) -> Arc<dyn IdentityPort>;
    fn create_settings_adapter(&self) -> Arc<dyn SettingsPort>;
}
