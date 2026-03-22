use crate::domain::errors::DomainError;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Email(String);

impl Email {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();
        let value = value.trim().to_lowercase();

        if value.is_empty() {
            return Err(DomainError::InvalidData("Email cannot be empty".into()));
        }

        if !value.contains('@') || !value.contains('.') {
            return Err(DomainError::InvalidData("Invalid email format".into()));
        }

        if value.len() > 254 {
            return Err(DomainError::InvalidData("Email is too long".into()));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
