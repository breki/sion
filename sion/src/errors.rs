use std::fmt;

#[derive(Debug)]
pub struct SionError {
    pub message: String,
}

impl SionError {
    pub fn new(message: &str) -> SionError {
        SionError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for SionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SionError {}