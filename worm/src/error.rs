use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;

pub type Result<T> = std::result::Result<T, WormError>;

#[derive(Debug)]
pub enum WormError {
    Config(String),
    Io(io::Error),
    Json(serde_json::Error),
    Sql(rusqlite::Error),
}

impl Display for WormError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(message) => write!(formatter, "configuration error: {message}"),
            Self::Io(error) => write!(formatter, "io error: {error}"),
            Self::Json(error) => write!(formatter, "json error: {error}"),
            Self::Sql(error) => write!(formatter, "sqlite error: {error}"),
        }
    }
}

impl Error for WormError {}

impl From<io::Error> for WormError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for WormError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<rusqlite::Error> for WormError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sql(value)
    }
}
