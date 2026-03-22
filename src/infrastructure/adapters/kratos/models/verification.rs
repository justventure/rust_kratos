use crate::domain::value_objects::auth_method::AuthMethod;

#[derive(serde::Serialize)]
pub struct VerificationPayload {
    pub method: AuthMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub csrf_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transient_payload: Option<serde_json::Value>,
}

impl VerificationPayload {
    pub fn new(
        method: AuthMethod,
        email: Option<String>,
        code: Option<String>,
        csrf_token: String,
        transient_payload: Option<serde_json::Value>,
    ) -> Self {
        Self {
            method,
            email,
            code,
            csrf_token,
            transient_payload,
        }
    }
}
