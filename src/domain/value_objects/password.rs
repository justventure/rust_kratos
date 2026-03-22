use crate::domain::errors::DomainError;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Password(String);

impl Password {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();

        if value.len() < 8 {
            return Err(DomainError::InvalidData(
                "Password must be at least 8 characters".into(),
            ));
        }

        if value.len() > 72 {
            return Err(DomainError::InvalidData(
                "Password must be at most 72 characters".into(),
            ));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}
