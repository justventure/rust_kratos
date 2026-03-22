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
