use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelError {
    message: String,
}

impl ModelError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ModelError {}
