use std::sync::Arc;

use crate::domain::ports::inbound::login::AuthenticationPort;
use crate::domain::ports::inbound::recovery::RecoveryPort;
use crate::domain::ports::inbound::registration::RegistrationPort;
use crate::domain::ports::inbound::settings::SettingsPort;
use crate::domain::ports::inbound::verification::VerificationPort;
use crate::domain::ports::outbound::identity::IdentityPort;
use crate::domain::ports::outbound::session::SessionPort;
use crate::infrastructure::adapters::cache::redis_cache::RedisCache;
use crate::infrastructure::adapters::kratos::client::KratosClient;
use crate::infrastructure::adapters::kratos::http::identity::KratosIdentityAdapter;
use crate::infrastructure::adapters::kratos::http::login::KratosAuthenticationAdapter;
use crate::infrastructure::adapters::kratos::http::logout::KratosSessionAdapter;
use crate::infrastructure::adapters::kratos::http::recovery::KratosRecoveryAdapter;
use crate::infrastructure::adapters::kratos::http::register::KratosRegistrationAdapter;
use crate::infrastructure::adapters::kratos::http::settings::KratosSettingsAdapter;
use crate::infrastructure::adapters::kratos::http::verification::KratosVerificationAdapter;
use crate::infrastructure::di::adapter_factory::AdapterFactory;

pub struct KratosAdapterFactory {
    client: Arc<KratosClient>,
    cache: RedisCache,
    cache_ttl_secs: u64,
}

impl KratosAdapterFactory {
    pub fn from_client(client: Arc<KratosClient>, cache: RedisCache, cache_ttl_secs: u64) -> Self {
        Self {
            client,
            cache,
            cache_ttl_secs,
        }
    }
}

impl AdapterFactory for KratosAdapterFactory {
    fn create_registration_adapter(&self) -> Arc<dyn RegistrationPort> {
        Arc::new(KratosRegistrationAdapter::new(
            self.client.clone(),
            self.create_session_adapter(),
        ))
    }

    fn create_authentication_adapter(&self) -> Arc<dyn AuthenticationPort> {
        Arc::new(KratosAuthenticationAdapter::new(
            self.client.clone(),
            self.create_session_adapter(),
        ))
    }

    fn create_session_adapter(&self) -> Arc<dyn SessionPort> {
        Arc::new(KratosSessionAdapter::new(self.client.clone(), Some(self.cache.clone())))
    }

    fn create_recovery_adapter(&self) -> Arc<dyn RecoveryPort> {
        Arc::new(KratosRecoveryAdapter::new(self.client.clone()))
    }

    fn create_verification_adapter(&self) -> Arc<dyn VerificationPort> {
        Arc::new(KratosVerificationAdapter::new(self.client.clone()))
    }

    fn create_identity_adapter(&self) -> Arc<dyn IdentityPort> {
        Arc::new(KratosIdentityAdapter::new(
            self.client.clone(),
            Some(self.cache.clone()),
            self.cache_ttl_secs,
        ))
    }

    fn create_settings_adapter(&self) -> Arc<dyn SettingsPort> {
        Arc::new(KratosSettingsAdapter::new(self.client.clone()))
    }
}
