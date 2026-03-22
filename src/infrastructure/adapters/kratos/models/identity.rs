use crate::domain::entities::user_profile::UserProfile;
use chrono::{DateTime, Utc};

#[derive(serde::Deserialize)]
pub struct SessionResponse {
    pub identity: Identity,
    pub active: bool,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize)]
pub struct Identity {
    pub id: String,
    pub traits: Traits,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub state: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct Traits {
    pub email: String,
    pub username: String,
    pub geo_location: Option<String>,
}

impl From<(Identity, bool, Option<DateTime<Utc>>)> for UserProfile {
    fn from((identity, active, expires_at): (Identity, bool, Option<DateTime<Utc>>)) -> Self {
        Self {
            id: identity.id,
            email: identity.traits.email,
            username: identity.traits.username,
            geo_location: identity.traits.geo_location,
            created_at: identity.created_at,
            updated_at: identity.updated_at,
            state: identity.state,
            active,
            expires_at,
        }
    }
}
