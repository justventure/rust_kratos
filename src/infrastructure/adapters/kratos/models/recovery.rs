use crate::domain::value_objects::auth_method::AuthMethod;

#[derive(serde::Serialize)]
pub struct RecoveryPayload {
    pub method: AuthMethod,
    pub email: String,
    pub csrf_token: String,
}

impl RecoveryPayload {
    pub fn new(email: &str, csrf_token: String) -> Self {
        Self {
            method: AuthMethod::Link,
            email: email.to_string(),
            csrf_token,
        }
    }
}
