use std::error::Error as StdError;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum SwarmError {
    Config(String),
    Io(io::Error),
    Json(serde_json::Error),
    Parse(String),
    Validation(String),
    Other(String),
}

impl fmt::Display for SwarmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(message) => write!(f, "Configuration error: {}", message),
            Self::Io(error) => write!(f, "I/O error: {}", error),
            Self::Json(error) => write!(f, "JSON error: {}", error),
            Self::Parse(message) => write!(f, "Parsing error: {}", message),
            Self::Validation(message) => write!(f, "Validation error: {}", message),
            Self::Other(message) => write!(f, "Error: {}", message),
        }
    }
}

impl StdError for SwarmError {}

impl From<io::Error> for SwarmError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for SwarmError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<toml::de::Error> for SwarmError {
    fn from(error: toml::de::Error) -> Self {
        Self::Parse(error.to_string())
    }
}

impl From<toml::ser::Error> for SwarmError {
    fn from(error: toml::ser::Error) -> Self {
        Self::Parse(error.to_string())
    }
}

pub type SwarmResult<T> = Result<T, SwarmError>;
