use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;

pub type Result<T> = std::result::Result<T, CrawlerError>;

#[derive(Debug)]
pub enum CrawlerError {
    Config(String),
    Git(String),
    InvalidRoot(String),
    Io(io::Error),
    Json(serde_json::Error),
    Notify(notify::Error),
    Sql(rusqlite::Error),
    TomlDe(toml::de::Error),
    TomlSer(toml::ser::Error),
    Utf8(std::string::FromUtf8Error),
    Yaml(serde_yaml::Error),
}

impl Display for CrawlerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(message) => write!(formatter, "configuration error: {message}"),
            Self::Git(message) => write!(formatter, "git error: {message}"),
            Self::InvalidRoot(message) => write!(formatter, "invalid repository root: {message}"),
            Self::Io(error) => write!(formatter, "io error: {error}"),
            Self::Json(error) => write!(formatter, "json error: {error}"),
            Self::Notify(error) => write!(formatter, "watch error: {error}"),
            Self::Sql(error) => write!(formatter, "sqlite error: {error}"),
            Self::TomlDe(error) => write!(formatter, "toml decode error: {error}"),
            Self::TomlSer(error) => write!(formatter, "toml encode error: {error}"),
            Self::Utf8(error) => write!(formatter, "utf-8 error: {error}"),
            Self::Yaml(error) => write!(formatter, "yaml error: {error}"),
        }
    }
}

impl Error for CrawlerError {}

impl From<io::Error> for CrawlerError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<rusqlite::Error> for CrawlerError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sql(value)
    }
}

impl From<serde_json::Error> for CrawlerError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<notify::Error> for CrawlerError {
    fn from(value: notify::Error) -> Self {
        Self::Notify(value)
    }
}

impl From<toml::de::Error> for CrawlerError {
    fn from(value: toml::de::Error) -> Self {
        Self::TomlDe(value)
    }
}

impl From<toml::ser::Error> for CrawlerError {
    fn from(value: toml::ser::Error) -> Self {
        Self::TomlSer(value)
    }
}

impl From<std::string::FromUtf8Error> for CrawlerError {
    fn from(value: std::string::FromUtf8Error) -> Self {
        Self::Utf8(value)
    }
}

impl From<serde_yaml::Error> for CrawlerError {
    fn from(value: serde_yaml::Error) -> Self {
        Self::Yaml(value)
    }
}
