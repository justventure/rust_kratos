use serde::Serialize;

use crate::domain::ports::inbound::login::LoginCredentials;
use crate::domain::value_objects::auth_method::AuthMethod;

#[derive(serde::Serialize)]
pub struct LoginPayload {
    pub method: AuthMethod,
    pub identifier: String,
    pub password: String,
    pub csrf_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resend: Option<String>,
}

impl LoginPayload {
    pub fn from_credentials(credentials: LoginCredentials, csrf_token: String) -> Self {
        Self {
            method: if credentials.code.is_some() {
                AuthMethod::Code
            } else {
                AuthMethod::Password
            },
            identifier: credentials.identifier.as_str().to_string(),
            password: credentials.password.as_str().to_string(),
            csrf_token,
            address: credentials.address,
            code: credentials.code,
            resend: credentials.resend,
        }
    }
}

#[derive(Serialize)]
pub struct OidcLoginPayload {
    pub method: String,
    pub provider: String,
    pub csrf_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token_nonce: Option<String>,
}

impl OidcLoginPayload {
    pub fn new(provider: &str, csrf_token: String) -> Self {
        Self {
            method: "oidc".to_string(),
            provider: provider.to_string(),
            csrf_token,
            id_token: None,
            id_token_nonce: None,
        }
    }
}
