use crate::domain::ports::inbound::settings::SettingsData;
use crate::domain::value_objects::auth_method::AuthMethod;

#[derive(serde::Serialize)]
pub struct SettingsPayload {
    pub method: AuthMethod,
    pub csrf_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traits: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lookup_secret_confirm: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lookup_secret_disable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lookup_secret_regenerate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lookup_secret_reveal: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transient_payload: Option<serde_json::Value>,
}

impl SettingsPayload {
    pub fn from_data(data: SettingsData, csrf_token: String) -> Self {
        let method = match data.method.as_str() {
            "password" => AuthMethod::Password,
            "code" => AuthMethod::Code,
            _ => AuthMethod::Link,
        };
        Self {
            method,
            csrf_token,
            password: data.password.map(|p| p.as_str().to_string()),
            traits: data.traits,
            lookup_secret_confirm: data.lookup_secret_confirm,
            lookup_secret_disable: data.lookup_secret_disable,
            lookup_secret_regenerate: data.lookup_secret_regenerate,
            lookup_secret_reveal: data.lookup_secret_reveal,
            transient_payload: data.transient_payload,
        }
    }
}
