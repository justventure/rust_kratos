use std::sync::Arc;

use thiserror::Error;

use crate::application::commands::account::recovery::RecoveryCommandHandler;
use crate::application::commands::account::settings::UpdateSettingsCommandHandler;
use crate::application::commands::account::verification::VerificationCommandHandler;
use crate::application::commands::auth::login::LoginCommandHandler;
use crate::application::commands::auth::logout::LogoutCommandHandler;
use crate::application::commands::identity::register::RegisterCommandHandler;
use crate::application::queries::get_current_user::GetCurrentUserQueryHandler;
use crate::infrastructure::adapters::cache::redis_cache::{RedisCache, RedisCacheConfig};
use crate::infrastructure::adapters::kratos::client::{KratosClient, KratosClientConfig};
use crate::infrastructure::di::adapter_factory::AdapterFactory;
use crate::infrastructure::di::factory::KratosAdapterFactory;

pub struct ContainerConfig {
    pub kratos: KratosClientConfig,
    pub redis: RedisCacheConfig,
}

pub struct Commands {
    pub login: LoginCommandHandler,
    pub logout: LogoutCommandHandler,
    pub register: RegisterCommandHandler,
    pub recovery: RecoveryCommandHandler,
    pub update_settings: UpdateSettingsCommandHandler,
    pub verification: VerificationCommandHandler,
}

pub struct Queries {
    pub get_current_user: GetCurrentUserQueryHandler,
}

pub struct UseCases {
    pub commands: Commands,
    pub queries: Queries,
}

impl UseCases {
    pub fn new(factory: &dyn AdapterFactory) -> Self {
        Self {
            commands: Commands {
                login: LoginCommandHandler::new(factory.create_authentication_adapter()),
                logout: LogoutCommandHandler::new(factory.create_session_adapter()),
                register: RegisterCommandHandler::new(factory.create_registration_adapter()),
                recovery: RecoveryCommandHandler::new(factory.create_recovery_adapter()),
                update_settings: UpdateSettingsCommandHandler::new(factory.create_settings_adapter()),
                verification: VerificationCommandHandler::new(factory.create_verification_adapter()),
            },
            queries: Queries {
                get_current_user: GetCurrentUserQueryHandler::new(factory.create_identity_adapter()),
            },
        }
    }
}

#[derive(Clone)]
pub struct AppContainer {
    pub use_cases: Arc<UseCases>,
    pub cache: RedisCache,
    kratos: Arc<KratosClient>,
}

impl AppContainer {
    pub async fn new(config: ContainerConfig) -> Result<Self, ContainerError> {
        let kratos = Arc::new(KratosClient::new(&config.kratos));
        kratos
            .wait_until_ready()
            .await
            .map_err(|e| ContainerError::Initialization(format!("Kratos unavailable: {e}")))?;

        let cache = RedisCache::new_with_retry(&config.redis)
            .await
            .map_err(|e| ContainerError::Initialization(format!("Redis unavailable: {e}")))?;

        let factory = KratosAdapterFactory::from_client(kratos.clone(), cache.clone(), config.redis.cache_ttl_secs);

        Ok(Self {
            use_cases: Arc::new(UseCases::new(&factory)),
            cache,
            kratos,
        })
    }

    pub fn use_cases(&self) -> Arc<UseCases> {
        self.use_cases.clone()
    }

    pub fn kratos_client(&self) -> Arc<KratosClient> {
        self.kratos.clone()
    }
}

#[derive(Debug, Error)]
pub enum ContainerError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Initialization failed: {0}")]
    Initialization(String),
    #[error("Factory creation failed: {0}")]
    FactoryCreation(String),
}
