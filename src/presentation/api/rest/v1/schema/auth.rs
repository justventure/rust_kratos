use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct RegisterSchema {
    pub identifier: String,
    pub username: Option<String>,
    pub password: String,
    pub geo_location: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct LoginSchema {
    pub identifier: String,
    pub password: String,
    pub address: Option<String>,
    pub code: Option<String>,
    pub resend: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct RecoverySchema {
    pub email: String,
}

#[derive(Deserialize, ToSchema)]
pub struct VerifyByLinkSchema {
    pub email: String,
    pub transient_payload: Option<Value>,
}

#[derive(Deserialize, ToSchema)]
pub struct SendVerificationCodeSchema {
    pub email: String,
    pub transient_payload: Option<Value>,
}

#[derive(Deserialize, ToSchema)]
pub struct SubmitVerificationCodeSchema {
    pub code: String,
    pub transient_payload: Option<Value>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateSettingsSchema {
    pub method: String,
    pub password: Option<String>,
    pub traits: Option<Value>,
    pub lookup_secret_confirm: Option<bool>,
    pub lookup_secret_disable: Option<bool>,
    pub lookup_secret_regenerate: Option<bool>,
    pub lookup_secret_reveal: Option<bool>,
    pub transient_payload: Option<Value>,
}

#[derive(Serialize, ToSchema)]
pub struct UserProfileSchema {
    pub id: String,
    pub email: String,
    pub username: String,
    pub geo_location: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub state: Option<String>,
    pub active: bool,
    pub expires_at: Option<DateTime<Utc>>,
}
