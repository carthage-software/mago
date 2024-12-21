use mago_reporting::error::ReportingError;
use mago_source::error::SourceError;

#[derive(Debug)]
pub enum Error {
    Source(SourceError),
    Reporting(ReportingError),
    BuildingRuntime(std::io::Error),
    Walking(async_walkdir::Error),
    BuildingConfiguration(config::ConfigError),
    DeserializingToml(toml::de::Error),
    SerializingToml(toml::ser::Error),
    CanonicalizingPath(std::path::PathBuf, std::io::Error),
    Join(tokio::task::JoinError),
    Json(serde_json::Error),
    SelfUpdate(self_update::errors::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Source(error) => write!(f, "{}", error),
            Self::Reporting(error) => write!(f, "{}", error),
            Self::Walking(error) => write!(f, "{}", error),
            Self::BuildingRuntime(error) => write!(f, "{}", error),
            Self::BuildingConfiguration(error) => write!(f, "{}", error),
            Self::DeserializingToml(error) => write!(f, "{}", error),
            Self::SerializingToml(error) => write!(f, "{}", error),
            Self::CanonicalizingPath(_, error) => write!(f, "{}", error),
            Self::Join(error) => write!(f, "{}", error),
            Self::Json(error) => write!(f, "{}", error),
            Self::SelfUpdate(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Source(error) => Some(error),
            Self::Reporting(error) => Some(error),
            Self::Walking(error) => Some(error),
            Self::BuildingConfiguration(error) => Some(error),
            Self::BuildingRuntime(error) => Some(error),
            Self::DeserializingToml(error) => Some(error),
            Self::SerializingToml(error) => Some(error),
            Self::CanonicalizingPath(_, error) => Some(error),
            Self::Join(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::SelfUpdate(error) => Some(error),
        }
    }
}

impl From<SourceError> for Error {
    fn from(error: SourceError) -> Self {
        Self::Source(error)
    }
}

impl From<ReportingError> for Error {
    fn from(error: ReportingError) -> Self {
        Self::Reporting(error)
    }
}

impl From<async_walkdir::Error> for Error {
    fn from(error: async_walkdir::Error) -> Self {
        Self::Walking(error)
    }
}

impl From<config::ConfigError> for Error {
    fn from(error: config::ConfigError) -> Self {
        Self::BuildingConfiguration(error)
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self {
        Self::DeserializingToml(error)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(error: toml::ser::Error) -> Self {
        Self::SerializingToml(error)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(error: tokio::task::JoinError) -> Self {
        Self::Join(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl From<self_update::errors::Error> for Error {
    fn from(error: self_update::errors::Error) -> Self {
        Self::SelfUpdate(error)
    }
}
