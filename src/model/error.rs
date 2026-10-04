use core::fmt;
use std::io;

#[derive(Debug)]
pub enum SwarmError {
    Config(String),
    Io(io::Error),
    Json(serde_json::Error),
    Other(String),
    Parse(String),
    Validation(String),
}

impl fmt::Display for SwarmError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(message) => write!(formatter, "Configuration error: {message}"),
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Json(error) => write!(formatter, "JSON error: {error}"),
            Self::Other(message) => write!(formatter, "Error: {message}"),
            Self::Parse(message) => write!(formatter, "Parsing error: {message}"),
            Self::Validation(message) => write!(formatter, "Validation error: {message}"),
        }
    }
}

impl core::error::Error for SwarmError {}

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
