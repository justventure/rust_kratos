use crate::domain::ports::inbound::registration::RegistrationData;
use crate::domain::value_objects::auth_method::AuthMethod;

#[derive(serde::Serialize)]
pub struct RegistrationTraits {
    pub email: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geo_location: Option<String>,
}

#[derive(serde::Serialize)]
pub struct RegistrationPayload {
    pub method: AuthMethod,
    pub password: String,
    pub traits: RegistrationTraits,
    pub csrf_token: String,
}

impl RegistrationPayload {
    pub fn from_data(data: RegistrationData, csrf_token: String) -> Self {
        Self {
            method: AuthMethod::Password,
            password: data.password.as_str().to_string(),
            traits: RegistrationTraits {
                email: data.email.as_str().to_string(),
                username: data.username,
                geo_location: data.geo_location,
            },
            csrf_token,
        }
    }
}
