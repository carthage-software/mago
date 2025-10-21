//! Error types for the Mago CLI application.
//!
//! This module defines the [`Error`] enum which encompasses all possible errors that can occur
//! during the execution of Mago commands. The error type provides detailed error messages,
//! proper error chaining via [`std::error::Error::source`], and automatic conversion from
//! various underlying error types.
//!
//! # Error Handling Strategy
//!
//! The CLI uses a centralized error handling approach where all errors are:
//!
//! 1. **Converted**: Underlying errors from dependencies are converted into [`Error`] variants
//!    using [`From`] implementations for ergonomic error propagation with the `?` operator
//! 2. **Logged**: All errors are logged via the tracing framework in [`main()`](crate::main)
//!    before the application exits
//! 3. **Displayed**: User-friendly error messages are provided via the [`Display`](std::fmt::Display)
//!    implementation, with technical details available through [`source()`](std::error::Error::source)
//! 4. **Mapped to Exit Codes**: All errors result in [`ExitCode::FAILURE`](std::process::ExitCode::FAILURE)
//!
//! # Error Categories
//!
//! Errors are organized into several categories:
//!
//! - **I/O Errors**: File reading, writing, and path canonicalization failures
//! - **Configuration Errors**: TOML/JSON parsing and configuration building errors
//! - **Database Errors**: File system scanning and database loading errors
//! - **Analysis Errors**: Static analysis, linting, and formatting errors
//! - **Runtime Errors**: Thread pool initialization and runtime building errors
//! - **User Interaction Errors**: Terminal interaction and dialoguer errors
//! - **Version Errors**: PHP version validation and parsing errors

use dialoguer::Error as DialoguerError;

use mago_analyzer::error::AnalysisError;
use mago_database::error::DatabaseError;
use mago_orchestrator::OrchestratorError;
use mago_php_version::PHPVersion;
use mago_php_version::error::ParsingError;
use mago_reporting::error::ReportingError;
use rayon::ThreadPoolBuildError;

/// The main error type for all Mago CLI operations.
///
/// This enum encompasses all possible errors that can occur during command execution,
/// from configuration loading to analysis and reporting. Each variant wraps the underlying
/// error type and provides context-specific error messages.
///
/// # Error Propagation
///
/// The error type implements [`From`] for common underlying error types, enabling
/// ergonomic error propagation using the `?` operator:
///
/// ```ignore
/// fn load_config() -> Result<Config, Error> {
///     let content = std::fs::read_to_string("mago.toml")?; // io::Error -> Error
///     let config: Config = toml::from_str(&content)?;      // toml::de::Error -> Error
///     Ok(config)
/// }
/// ```
///
/// # Error Chain
///
/// All variants that wrap underlying errors preserve the error chain via the
/// [`source()`](std::error::Error::source) method, allowing full error context
/// to be accessed programmatically or logged for debugging.
#[derive(Debug)]
pub enum Error {
    /// Failed to load or scan the file database.
    ///
    /// This error occurs during file system scanning when the database loader encounters
    /// issues such as permission errors, invalid paths, or I/O failures while discovering
    /// PHP files in the workspace.
    Database(DatabaseError),

    /// Failed to generate or output analysis reports.
    ///
    /// This error occurs when the reporting system cannot format or write analysis results,
    /// such as when writing to stdout/stderr fails or when generating JSON/SARIF output
    /// encounters serialization errors.
    Reporting(ReportingError),

    /// Failed to build the async runtime.
    ///
    /// This error occurs if the Tokio runtime cannot be initialized, typically due to
    /// system resource constraints or incompatible runtime configurations.
    BuildingRuntime(std::io::Error),

    /// Failed to build or merge configuration from multiple sources.
    ///
    /// This error occurs when loading configuration from environment variables, TOML files,
    /// or command-line arguments fails due to invalid settings, missing required fields,
    /// or incompatible configuration values.
    BuildingConfiguration(config::ConfigError),

    /// Failed to deserialize TOML configuration.
    ///
    /// This error occurs when parsing `mago.toml` or global configuration files fails due to
    /// syntax errors, type mismatches, or invalid TOML structure.
    DeserializingToml(toml::de::Error),

    /// Failed to serialize configuration to TOML format.
    ///
    /// This error occurs during the `mago init` command when the default configuration
    /// cannot be serialized to TOML, typically indicating a programming error rather than
    /// user error.
    SerializingToml(toml::ser::Error),

    /// Failed to canonicalize a file path.
    ///
    /// This error occurs when attempting to resolve a path to its absolute canonical form
    /// fails, typically because the path does not exist, involves broken symlinks, or
    /// permission issues prevent accessing parent directories.
    ///
    /// The first field contains the path that failed to canonicalize, and the second
    /// field contains the underlying I/O error.
    CanonicalizingPath(std::path::PathBuf, std::io::Error),

    /// Failed to parse or serialize JSON data.
    ///
    /// This error occurs when working with JSON files such as `composer.json` or when
    /// generating JSON output for analysis results. Common causes include malformed JSON,
    /// unexpected data types, or serialization failures.
    Json(serde_json::Error),

    /// Failed to perform self-update operation.
    ///
    /// This error occurs during the `mago self-update` command when the update process
    /// fails due to network issues, missing release assets, permission errors when
    /// replacing the binary, or checksum verification failures.
    SelfUpdate(self_update::errors::Error),

    /// The configured PHP version is too old and not supported.
    ///
    /// This error occurs when the configured PHP version is below the minimum supported
    /// version. The first field contains the minimum supported version, and the second
    /// field contains the actual configured version.
    ///
    /// This check can be bypassed using the `--allow-unsupported-php-version` flag.
    PHPVersionIsTooOld(PHPVersion, PHPVersion),

    /// The configured PHP version is too new and not supported.
    ///
    /// This error occurs when the configured PHP version exceeds the maximum supported
    /// version. The first field contains the maximum supported version, and the second
    /// field contains the actual configured version.
    ///
    /// This check can be bypassed using the `--allow-unsupported-php-version` flag.
    PHPVersionIsTooNew(PHPVersion, PHPVersion),

    /// Failed to parse the PHP version string.
    ///
    /// This error occurs when a PHP version string (from CLI arguments, environment
    /// variables, or configuration files) cannot be parsed into a valid [`PHPVersion`].
    /// The first field contains the invalid version string, and the second field
    /// contains the parsing error details.
    InvalidPHPVersion(String, ParsingError),

    /// Failed to interact with the user via the terminal.
    ///
    /// This error occurs during interactive prompts (such as `mago init`) when dialoguer
    /// cannot read user input due to terminal I/O errors, interrupted input, or when
    /// running in a non-interactive environment.
    Dialoguer(DialoguerError),

    /// Failed to write the configuration file to disk.
    ///
    /// This error occurs during the `mago init` command when writing the new `mago.toml`
    /// file fails due to permission errors, disk full, or I/O failures.
    WritingConfiguration(std::io::Error),

    /// Failed to read the `composer.json` file.
    ///
    /// This error occurs when attempting to load `composer.json` for PHP version detection
    /// or autoload path resolution fails due to missing file, permission errors, or I/O
    /// failures.
    ReadingComposerJson(std::io::Error),

    /// Failed to read the baseline file.
    ///
    /// This error occurs when loading an existing baseline file for incremental issue
    /// tracking fails due to missing file, permission errors, or I/O failures.
    ReadingBaselineFile(std::io::Error),

    /// Failed to create or write the baseline file.
    ///
    /// This error occurs when generating a new baseline file fails due to permission
    /// errors, disk full, or I/O failures.
    CreatingBaselineFile(std::io::Error),

    /// Failed to parse the `composer.json` file.
    ///
    /// This error occurs when `composer.json` contains invalid JSON syntax or unexpected
    /// structure that prevents extracting PHP version or autoload configuration.
    ParsingComposerJson(serde_json::Error),

    /// Failed to initialize the thread pool for parallel processing.
    ///
    /// This error occurs when Rayon cannot build the global thread pool with the
    /// configured number of threads and stack size, typically due to system resource
    /// constraints or invalid configuration values.
    ThreadPoolBuildError(ThreadPoolBuildError),

    /// Failed to perform static analysis.
    ///
    /// This error occurs during the `mago analyze` command when the static analyzer
    /// encounters internal errors, such as type system failures, control flow analysis
    /// errors, or unrecoverable analysis state.
    Analysis(AnalysisError),

    /// Orchestrator operation failed.
    ///
    /// This error occurs when the orchestrator encounters errors while coordinating
    /// analysis tools, such as database loading failures, service initialization errors,
    /// or failures during parallel file processing.
    Orchestrator(OrchestratorError),
}

/// Formats the error for user-friendly display.
///
/// This implementation provides context-specific error messages for each variant,
/// making errors readable in terminal output and logs. For technical details and
/// the error chain, use [`source()`](std::error::Error::source).
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
            Self::Analysis(error) => write!(f, "Failed to analyze the source code: {error}"),
            Self::ThreadPoolBuildError(error) => {
                write!(f, "Failed to build the thread pool: {error}")
            }
            Self::Orchestrator(error) => write!(f, "Orchestrator error: {error}"),
        }
    }
}

/// Implements the standard error trait, providing access to the error chain.
///
/// The [`source()`](std::error::Error::source) method returns the underlying error
/// for variants that wrap other errors, allowing full error context to be accessed
/// programmatically. This is used by error reporting libraries and the tracing
/// framework to display complete error chains.
impl std::error::Error for Error {
    /// Returns the underlying error that caused this error, if any.
    ///
    /// This method enables error chain traversal, allowing callers to access the
    /// full context of why an operation failed. Most variants return `Some`,
    /// except for variants like `PHPVersionIsTooOld` and `PHPVersionIsTooNew`
    /// which represent validation errors without an underlying cause.
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
            Self::Analysis(error) => Some(error),
            Self::ThreadPoolBuildError(error) => Some(error),
            Self::Orchestrator(error) => Some(error),
            _ => None,
        }
    }
}

/// Converts database errors into CLI errors.
///
/// This enables the `?` operator to automatically convert [`DatabaseError`] into
/// [`Error`] when propagating errors from database operations.
impl From<DatabaseError> for Error {
    fn from(error: DatabaseError) -> Self {
        Self::Database(error)
    }
}

/// Converts reporting errors into CLI errors.
///
/// This enables the `?` operator to automatically convert [`ReportingError`] into
/// [`Error`] when propagating errors from report generation operations.
impl From<ReportingError> for Error {
    fn from(error: ReportingError) -> Self {
        Self::Reporting(error)
    }
}

/// Converts configuration building errors into CLI errors.
///
/// This enables the `?` operator to automatically convert [`config::ConfigError`]
/// into [`Error`] when propagating errors from configuration loading operations.
impl From<config::ConfigError> for Error {
    fn from(error: config::ConfigError) -> Self {
        Self::BuildingConfiguration(error)
    }
}

/// Converts TOML deserialization errors into CLI errors.
///
/// This enables the `?` operator to automatically convert [`toml::de::Error`]
/// into [`Error`] when propagating errors from TOML parsing operations.
impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self {
        Self::DeserializingToml(error)
    }
}

/// Converts TOML serialization errors into CLI errors.
///
/// This enables the `?` operator to automatically convert [`toml::ser::Error`]
/// into [`Error`] when propagating errors from TOML generation operations.
impl From<toml::ser::Error> for Error {
    fn from(error: toml::ser::Error) -> Self {
        Self::SerializingToml(error)
    }
}

/// Converts JSON serialization/deserialization errors into CLI errors.
///
/// This enables the `?` operator to automatically convert [`serde_json::Error`]
/// into [`Error`] when propagating errors from JSON operations.
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

/// Converts self-update errors into CLI errors.
///
/// This enables the `?` operator to automatically convert self-update crate errors
/// into [`Error`] when propagating errors from the `mago self-update` command.
impl From<self_update::errors::Error> for Error {
    fn from(error: self_update::errors::Error) -> Self {
        Self::SelfUpdate(error)
    }
}

/// Converts dialoguer interaction errors into CLI errors.
///
/// This enables the `?` operator to automatically convert [`DialoguerError`]
/// into [`Error`] when propagating errors from interactive terminal prompts.
impl From<DialoguerError> for Error {
    fn from(error: DialoguerError) -> Self {
        Self::Dialoguer(error)
    }
}

/// Converts analysis errors into CLI errors.
///
/// This enables the `?` operator to automatically convert [`AnalysisError`]
/// into [`Error`] when propagating errors from static analysis operations.
impl From<AnalysisError> for Error {
    fn from(error: AnalysisError) -> Self {
        Self::Analysis(error)
    }
}

/// Converts thread pool build errors into CLI errors.
///
/// This enables the `?` operator to automatically convert [`ThreadPoolBuildError`]
/// into [`Error`] when propagating errors from Rayon thread pool initialization.
impl From<ThreadPoolBuildError> for Error {
    fn from(error: ThreadPoolBuildError) -> Self {
        Self::ThreadPoolBuildError(error)
    }
}

/// Converts orchestrator errors into CLI errors.
///
/// This enables the `?` operator to automatically convert [`OrchestratorError`]
/// into [`Error`] when propagating errors from orchestrator operations.
impl From<OrchestratorError> for Error {
    fn from(error: OrchestratorError) -> Self {
        Self::Orchestrator(error)
    }
}
