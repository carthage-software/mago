use dialoguer::Error as DialoguerError;

use mago_analyzer::error::AnalysisError;
use mago_database::error::DatabaseError;
use mago_php_version::PHPVersion;
use mago_php_version::error::ParsingError;
use mago_reporting::error::ReportingError;
use rayon::ThreadPoolBuildError;

#[derive(Debug)]
pub enum Error {
    Database(DatabaseError),
    Reporting(ReportingError),
    BuildingRuntime(std::io::Error),
    BuildingConfiguration(config::ConfigError),
    DeserializingToml(toml::de::Error),
    SerializingToml(toml::ser::Error),
    CanonicalizingPath(std::path::PathBuf, std::io::Error),
    Json(serde_json::Error),
    SelfUpdate(self_update::errors::Error),
    PHPVersionIsTooOld(PHPVersion, PHPVersion),
    PHPVersionIsTooNew(PHPVersion, PHPVersion),
    InvalidPHPVersion(String, ParsingError),
    Dialoguer(DialoguerError),
    WritingConfiguration(std::io::Error),
    ReadingComposerJson(std::io::Error),
    ReadingBaselineFile(std::io::Error),
    CreatingBaselineFile(std::io::Error),
    ParsingComposerJson(serde_json::Error),
    ThreadPoolBuildError(ThreadPoolBuildError),
    Pager(std::io::Error),
    Analysis(AnalysisError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(error) => write!(f, "Failed to load database: {error}"),
            Self::Reporting(error) => write!(f, "Failed to report results: {error}"),
            Self::BuildingRuntime(error) => write!(f, "Failed to build the runtime: {error}"),
            Self::BuildingConfiguration(error) => write!(f, "Failed to build the configuration: {error}"),
            Self::DeserializingToml(error) => write!(f, "Failed to deserialize TOML: {error}"),
            Self::SerializingToml(error) => write!(f, "Failed to serialize TOML: {error}"),
            Self::CanonicalizingPath(path, error) => write!(f, "Failed to canonicalize path `{path:?}`: {error}"),
            Self::Json(error) => write!(f, "Failed to parse JSON: {error}"),
            Self::SelfUpdate(error) => write!(f, "Failed to self update: {error}"),
            Self::PHPVersionIsTooOld(minimum, actual) => {
                write!(f, "PHP version {actual} is not supported, minimum supported version is {minimum}")
            }
            Self::PHPVersionIsTooNew(maximum, actual) => {
                write!(f, "PHP version {actual} is not supported, maximum supported version is {maximum}")
            }
            Self::InvalidPHPVersion(version, error) => {
                write!(f, "Invalid PHP version `{version}`: {error}")
            }
            Self::Dialoguer(error) => write!(f, "Failed to interact with the user: {error}"),
            Self::WritingConfiguration(error) => write!(f, "Failed to write the configuration file: {error}"),
            Self::ReadingComposerJson(error) => write!(f, "Failed to read the `composer.json` file: {error}"),
            Self::ParsingComposerJson(error) => write!(f, "Failed to parse the `composer.json` file: {error}"),
            Self::ReadingBaselineFile(error) => write!(f, "Failed to read the baseline file: {error}"),
            Self::CreatingBaselineFile(error) => write!(f, "Failed to create the baseline file: {error}"),
            Self::Pager(error) => write!(f, "Failed to launch the pager: {error}"),
            Self::Analysis(error) => write!(f, "Failed to analyze the source code: {error}"),
            Self::ThreadPoolBuildError(error) => {
                write!(f, "Failed to build the thread pool: {error}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::Reporting(error) => Some(error),
            Self::BuildingConfiguration(error) => Some(error),
            Self::BuildingRuntime(error) => Some(error),
            Self::DeserializingToml(error) => Some(error),
            Self::SerializingToml(error) => Some(error),
            Self::CanonicalizingPath(_, error) => Some(error),
            Self::Json(error) => Some(error),
            Self::SelfUpdate(error) => Some(error),
            Self::InvalidPHPVersion(_, error) => Some(error),
            Self::Dialoguer(error) => Some(error),
            Self::WritingConfiguration(error) => Some(error),
            Self::ReadingComposerJson(error) => Some(error),
            Self::ParsingComposerJson(error) => Some(error),
            Self::ReadingBaselineFile(error) => Some(error),
            Self::CreatingBaselineFile(error) => Some(error),
            Self::Pager(error) => Some(error),
            Self::Analysis(error) => Some(error),
            Self::ThreadPoolBuildError(error) => Some(error),
            _ => None,
        }
    }
}

impl From<DatabaseError> for Error {
    fn from(error: DatabaseError) -> Self {
        Self::Database(error)
    }
}

impl From<ReportingError> for Error {
    fn from(error: ReportingError) -> Self {
        Self::Reporting(error)
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

impl From<DialoguerError> for Error {
    fn from(error: DialoguerError) -> Self {
        Self::Dialoguer(error)
    }
}

impl From<AnalysisError> for Error {
    fn from(error: AnalysisError) -> Self {
        Self::Analysis(error)
    }
}

impl From<ThreadPoolBuildError> for Error {
    fn from(error: ThreadPoolBuildError) -> Self {
        Self::ThreadPoolBuildError(error)
    }
}
