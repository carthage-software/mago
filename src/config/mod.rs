//! Configuration management for Mago CLI.
//!
//! This module provides a comprehensive configuration system that supports multiple
//! sources and precedence levels. Configuration can be loaded from TOML files,
//! environment variables, and command-line arguments, with each source overriding
//! the previous one.
//!
//! # Configuration Sources
//!
//! Configuration is loaded and merged from the following sources, in order of precedence
//! (highest to lowest):
//!
//! 1. **Environment Variables**: Prefixed with `MAGO_` (e.g., `MAGO_THREADS=4`)
//! 2. **Workspace Config**: `mago.toml` in the workspace directory
//! 3. **Global Config**: `mago.toml` in `$XDG_CONFIG_HOME` or `$HOME`
//! 4. **Explicit File**: Specified via `--config` flag (bypasses workspace/global)
//! 5. **Defaults**: Built-in defaults for all settings
//!
//! # Configuration Structure
//!
//! The configuration is organized into several sub-configurations:
//!
//! - **Source**: Defines workspace paths and file discovery patterns
//! - **Linter**: Controls linting behavior and rule sets
//! - **Formatter**: Defines code formatting style preferences
//! - **Analyzer**: Controls static type analysis settings
//! - **Guard**: Configures the guard service for continuous monitoring
//!
//! # Normalization and Validation
//!
//! After loading, configurations are normalized to ensure valid values:
//!
//! - Thread count defaults to logical CPU count if zero
//! - Stack size is clamped between minimum and maximum bounds
//! - PHP version compatibility is validated
//! - Source paths are resolved and validated
//!
//! # Environment Variables
//!
//! All configuration options can be set via environment variables using the
//! `MAGO_` prefix and kebab-case conversion. For nested options, use underscores:
//!
//! - `MAGO_THREADS` → `threads`
//! - `MAGO_PHP_VERSION` → `php_version`
//! - `MAGO_SOURCE_WORKSPACE` → `source.workspace`

use std::env::home_dir;
use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;

use config::Case;
use config::Config;
use config::Environment;
use config::File;
use config::FileFormat;
use config::FileStoredFormat;
use config::Value;
use config::ValueKind;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;

use crate::config::analyzer::AnalyzerConfiguration;
use crate::config::formatter::FormatterConfiguration;
use crate::config::guard::GuardConfiguration;
use crate::config::linter::LinterConfiguration;
use crate::config::source::SourceConfiguration;
use crate::consts::*;
use crate::error::Error;

pub mod analyzer;
pub mod formatter;
pub mod guard;
pub mod linter;
pub mod source;

/// Default value for threads configuration field.
fn default_threads() -> usize {
    *LOGICAL_CPUS
}

/// Default value for stack_size configuration field.
fn default_stack_size() -> usize {
    DEFAULT_STACK_SIZE
}

/// Default value for php_version configuration field.
fn default_php_version() -> PHPVersion {
    DEFAULT_PHP_VERSION
}

/// Default value for source configuration field.
fn default_source_configuration() -> SourceConfiguration {
    SourceConfiguration::from_workspace(CURRENT_DIR.clone())
}

/// The main configuration structure for Mago CLI.
///
/// This struct aggregates all configuration settings for Mago, including global options
/// like threading and PHP version, as well as specialized configurations for each service
/// (linter, analyzer, formatter, guard).
///
/// Configuration values are loaded from multiple sources with the following precedence
/// (from highest to lowest):
///
/// 1. Environment variables (`MAGO_*`)
/// 2. Workspace `mago.toml` file
/// 3. Global `mago.toml` file (`$XDG_CONFIG_HOME` or `$HOME`)
/// 4. Built-in defaults
///
/// The struct uses serde for deserialization from TOML files and environment variables,
/// with strict validation via `deny_unknown_fields` to catch configuration errors early.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Configuration {
    /// Number of worker threads for parallel processing.
    ///
    /// Controls the thread pool size used by Rayon for parallel operations.
    /// If set to 0, defaults to the number of logical CPUs available.
    /// Can be overridden via `MAGO_THREADS` environment variable or `--threads` CLI flag.
    #[serde(default = "default_threads")]
    pub threads: usize,

    /// Stack size for each worker thread in bytes.
    ///
    /// Determines the maximum stack size allocated for each thread in the thread pool.
    /// Must be between `MINIMUM_STACK_SIZE` and `MAXIMUM_STACK_SIZE`.
    /// If set to 0, uses `MAXIMUM_STACK_SIZE`. Values outside the valid range are
    /// automatically clamped during normalization.
    #[serde(default = "default_stack_size")]
    pub stack_size: usize,

    /// Target PHP version for parsing and analysis.
    ///
    /// Specifies which PHP version to assume when parsing code and performing analysis.
    /// This affects syntax parsing rules, available built-in functions, and type checking.
    /// Can be overridden via `MAGO_PHP_VERSION` environment variable or `--php-version` CLI flag.
    #[serde(default = "default_php_version")]
    pub php_version: PHPVersion,

    /// Whether to allow PHP versions not officially supported by Mago.
    ///
    /// When enabled, Mago will not fail if the specified PHP version is outside the
    /// officially supported range. Use with caution as behavior may be unpredictable.
    /// Can be enabled via `MAGO_ALLOW_UNSUPPORTED_PHP_VERSION` environment variable
    /// or `--allow-unsupported-php-version` CLI flag.
    #[serde(default)]
    pub allow_unsupported_php_version: bool,

    /// Source discovery and workspace configuration.
    ///
    /// Defines the workspace root, source paths to scan, and exclusion patterns.
    /// This configuration determines which PHP files are loaded into the database
    /// for analysis, linting, or formatting.
    #[serde(default = "default_source_configuration")]
    pub source: SourceConfiguration,

    /// Linter service configuration.
    ///
    /// Controls linting behavior, including enabled/disabled rules, rule-specific
    /// settings, and reporting preferences. Defaults to an empty configuration
    /// if not specified in the config file.
    #[serde(default)]
    pub linter: LinterConfiguration,

    /// Formatter service configuration.
    ///
    /// Defines code formatting style preferences such as indentation, line width,
    /// brace placement, and spacing rules. Defaults to standard formatting settings
    /// if not specified in the config file.
    #[serde(default)]
    pub formatter: FormatterConfiguration,

    /// Type analyzer service configuration.
    ///
    /// Controls static type analysis behavior, including strictness levels,
    /// inference settings, and type-related rules. Defaults to an empty configuration
    /// if not specified in the config file.
    #[serde(default)]
    pub analyzer: AnalyzerConfiguration,

    /// Guard service configuration for continuous monitoring.
    ///
    /// Defines settings for the guard/watch mode, including file watching behavior,
    /// debouncing, and incremental analysis. Defaults to an empty configuration
    /// if not specified in the config file.
    #[serde(default)]
    pub guard: GuardConfiguration,

    /// Log filter for tracing output.
    ///
    /// This field exists solely to prevent errors when `MAGO_LOG` environment
    /// variable is set. Due to `deny_unknown_fields`, without this field serde
    /// would reject the configuration when `MAGO_LOG` is present in the environment.
    ///
    /// This is not a user-facing configuration option and is never serialized.
    #[serde(default, skip_serializing)]
    #[schemars(skip)]
    #[allow(dead_code)]
    log: Value,
}

impl Configuration {
    /// Loads and merges configuration from multiple sources.
    ///
    /// This method orchestrates the complete configuration loading process, combining
    /// settings from files, environment variables, and CLI arguments with proper
    /// precedence handling. The configuration is then normalized and validated.
    ///
    /// # Loading Process
    ///
    /// The method follows this workflow:
    ///
    /// 1. **Create Base Configuration**: Initialize with workspace-specific defaults
    /// 2. **Load Global Configs**: Merge `$XDG_CONFIG_HOME/mago.toml` and `$HOME/mago.toml`
    ///    (skipped if explicit config file is provided)
    /// 3. **Load Workspace Config**: Merge `<workspace>/mago.toml`
    ///    (skipped if explicit config file is provided)
    /// 4. **Load Explicit Config**: If `file` is provided, load it as the sole config file
    /// 5. **Apply Environment Variables**: Override with `MAGO_*` environment variables
    /// 6. **Apply CLI Overrides**: Apply specific CLI argument overrides
    /// 7. **Normalize and Validate**: Ensure all values are within valid ranges
    ///
    /// # Precedence Order
    ///
    /// Settings are merged with the following precedence (highest to lowest):
    ///
    /// 1. CLI arguments (`php_version`, `threads`, `allow_unsupported_php_version`)
    /// 2. Environment variables (`MAGO_*`)
    /// 3. Workspace config file (`<workspace>/mago.toml`)
    /// 4. Global config files (`$XDG_CONFIG_HOME/mago.toml`, `$HOME/mago.toml`)
    /// 5. Explicit config file (if provided via `file` parameter, bypasses 3 and 4)
    /// 6. Default values
    ///
    /// # Arguments
    ///
    /// * `workspace` - Optional workspace directory path. If not provided, uses current directory.
    ///   This directory is scanned for source files and may contain a `mago.toml` file.
    /// * `file` - Optional explicit path to a configuration file. When provided, global and
    ///   workspace config files are not loaded. The specified file must exist.
    /// * `php_version` - Optional PHP version override. Takes precedence over all config sources.
    /// * `threads` - Optional thread count override. Takes precedence over all config sources.
    /// * `allow_unsupported_php_version` - If `true`, enables support for PHP versions outside
    ///   the officially supported range. Only overrides the config if `true`.
    ///
    /// # Returns
    ///
    /// - `Ok(Configuration)` - Successfully loaded and validated configuration
    /// - `Err(Error::Configuration)` - Failed to parse TOML or deserialize configuration
    /// - `Err(Error::IOError)` - Failed to read configuration file (when `file` is provided)
    ///
    /// # Errors
    ///
    /// This method may fail if:
    /// - The specified config file does not exist or cannot be read (when `file` is provided)
    /// - Configuration files contain invalid TOML syntax
    /// - Configuration values fail validation (e.g., unknown field names due to `deny_unknown_fields`)
    /// - Required configuration fields are missing
    /// - Normalization fails (e.g., invalid source paths)
    pub fn load(
        workspace: Option<PathBuf>,
        file: Option<&Path>,
        php_version: Option<PHPVersion>,
        threads: Option<usize>,
        allow_unsupported_php_version: bool,
    ) -> Result<Configuration, Error> {
        let workspace_dir = workspace.clone().unwrap_or_else(|| CURRENT_DIR.to_path_buf());

        let mut configuration = Configuration::from_workspace(workspace_dir.clone());
        let mut builder = Config::builder().add_source(Config::try_from(&configuration)?);

        if let Some(file) = file {
            tracing::debug!("Sourcing configuration from {}.", file.display());

            builder = builder.add_source(File::from(file).required(true));
        } else {
            let formats = [FileFormat::Toml, FileFormat::Yaml, FileFormat::Json];
            // Check workspace first, then XDG, then home (workspace has highest precedence)
            let fallback_roots = [std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from), home_dir()];
            if let Some((config_file, format)) = Self::find_config_files(&workspace_dir, &fallback_roots, &formats) {
                tracing::debug!("Sourcing configuration from {}.", config_file.display());
                builder = builder.add_source(File::from(config_file).format(format).required(false));
            } else {
                tracing::debug!("No configuration file found, using defaults and environment variables.");
            }
        }

        configuration = builder
            .add_source(Environment::with_prefix(ENVIRONMENT_PREFIX).convert_case(Case::Kebab))
            .build()?
            .try_deserialize::<Configuration>()?;

        if allow_unsupported_php_version && !configuration.allow_unsupported_php_version {
            tracing::warn!("Allowing unsupported PHP versions.");

            configuration.allow_unsupported_php_version = true;
        }

        if let Some(php_version) = php_version {
            tracing::info!("Overriding PHP version with {}.", php_version);

            configuration.php_version = php_version;
        }

        if let Some(threads) = threads {
            tracing::info!("Overriding thread count with {}.", threads);

            configuration.threads = threads;
        }

        if let Some(workspace) = workspace {
            tracing::info!("Overriding workspace directory with {}.", workspace.display());

            configuration.source.workspace = workspace;
        }

        configuration.normalize()?;

        Ok(configuration)
    }

    /// Searches for configuration files in a project directory, falling back to global config locations.
    ///
    /// This function attempts to load at most one configuration file per supported config type.
    /// It first searches the given `root_dir` (typically the workspace/project directory).  
    /// If any configuration file is found there, those results are returned immediately and no
    /// fallback locations are checked.
    ///
    /// If no config files are found in `root_dir`, the function searches each directory in
    /// `fallback_roots` in order. The first matching format for each config file name is returned.
    ///
    /// # Arguments
    ///
    /// * `root_dir` - The primary directory to search (project root)
    /// * `fallback_roots` - A list of additional directories to search if `root_dir` contains no matches
    /// * `file_formats` - Supported configuration formats (`toml`, `yaml`, etc.), each with possible extensions
    ///
    /// # Returns
    ///
    /// A vector of `(PathBuf, FileFormat)` pairs, where:
    /// * The path is the resolved config file
    /// * The format indicates which file format identified it
    ///
    /// # Behavior Summary
    ///
    /// 1. Try to resolve `<root_dir>/<name>.<ext>` for each supported format
    /// 2. Stop and return immediately if any matches are found
    /// 3. Otherwise, search each directory in `fallback_roots` in order
    /// 4. The first match (by name and directory order) wins
    ///
    /// This prevents global configuration files from overriding project-local configuration.
    fn find_config_files(
        root_dir: &Path,
        fallback_roots: &[Option<PathBuf>],
        file_formats: &[FileFormat],
    ) -> Option<(PathBuf, FileFormat)> {
        let config_files = [CONFIGURATION_FILE_NAME, CONFIGURATION_DIST_FILE_NAME];

        for name in config_files.iter() {
            let base = root_dir.join(name);

            for format in file_formats.iter() {
                if let Some(ext) = format.file_extensions().iter().find(|ext| base.with_added_extension(ext).exists()) {
                    return Some((base.with_added_extension(ext), *format));
                }
            }
        }

        for root in fallback_roots.iter().flatten() {
            let base = root.join(CONFIGURATION_FILE_NAME);

            for format in file_formats.iter() {
                if let Some(ext) = format.file_extensions().iter().find(|ext| base.with_added_extension(ext).exists()) {
                    return Some((base.with_added_extension(ext), *format));
                }
            }
        }

        None
    }

    /// Creates a default configuration anchored to a specific workspace directory.
    ///
    /// This constructor initializes a configuration with sensible defaults suitable
    /// for immediate use. All service-specific configurations (linter, formatter,
    /// analyzer, guard) use their default settings.
    ///
    /// # Default Values
    ///
    /// - **threads**: Number of logical CPUs available on the system
    /// - **stack_size**: `DEFAULT_STACK_SIZE` (typically 8MB)
    /// - **php_version**: `DEFAULT_PHP_VERSION` (latest stable PHP version)
    /// - **allow_unsupported_php_version**: `false`
    /// - **source**: Workspace-specific source configuration
    /// - **linter**: Default linter configuration
    /// - **formatter**: Default formatter configuration
    /// - **analyzer**: Default analyzer configuration
    /// - **guard**: Default guard configuration
    ///
    /// This method is primarily used as the starting point for configuration loading,
    /// with values subsequently overridden by config files and environment variables.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The root directory of the workspace to analyze. This directory
    ///   serves as the base path for relative source paths and is where the database
    ///   will look for PHP files.
    ///
    /// # Returns
    ///
    /// A new `Configuration` instance with default values and the specified workspace.
    pub fn from_workspace(workspace: PathBuf) -> Self {
        Self {
            threads: *LOGICAL_CPUS,
            stack_size: DEFAULT_STACK_SIZE,
            php_version: DEFAULT_PHP_VERSION,
            allow_unsupported_php_version: false,
            source: SourceConfiguration::from_workspace(workspace),
            linter: LinterConfiguration::default(),
            formatter: FormatterConfiguration::default(),
            analyzer: AnalyzerConfiguration::default(),
            guard: GuardConfiguration::default(),
            log: Value::new(None, ValueKind::Nil),
        }
    }
}

impl Configuration {
    /// Returns a filtered version of the configuration suitable for display.
    ///
    /// This method excludes linter rules that don't match the configured integrations,
    /// so that only applicable rules are shown in the output.
    #[must_use]
    pub fn to_filtered_value(&self) -> serde_json::Value {
        serde_json::json!({
            "threads": self.threads,
            "stack-size": self.stack_size,
            "php-version": self.php_version,
            "allow-unsupported-php-version": self.allow_unsupported_php_version,
            "source": self.source,
            "linter": self.linter.to_filtered_value(self.php_version),
            "formatter": self.formatter,
            "analyzer": self.analyzer,
            "guard": self.guard,
        })
    }

    /// Normalizes and validates configuration values.
    ///
    /// This method ensures that all configuration values are within acceptable ranges
    /// and makes sensible adjustments where needed. It is called automatically by
    /// [`load`](Self::load) after merging all configuration sources.
    ///
    /// # Normalization Rules
    ///
    /// ## Thread Count
    ///
    /// - If set to `0`: Defaults to the number of logical CPUs
    /// - Otherwise: Uses the configured value as-is
    ///
    /// ## Stack Size
    ///
    /// - If set to `0`: Uses `MAXIMUM_STACK_SIZE`
    /// - If greater than `MAXIMUM_STACK_SIZE`: Clamped to `MAXIMUM_STACK_SIZE`
    /// - If less than `MINIMUM_STACK_SIZE`: Clamped to `MINIMUM_STACK_SIZE`
    /// - Otherwise: Uses the configured value as-is
    ///
    /// ## Source Configuration
    ///
    /// Delegates to [`SourceConfiguration::normalize`] to validate workspace paths
    /// and resolve relative paths.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if normalization succeeded
    /// - `Err(Error)` if source normalization failed (e.g., invalid workspace path)
    ///
    /// # Side Effects
    ///
    /// This method logs warnings and informational messages when values are adjusted,
    /// helping users understand how their configuration was interpreted.
    fn normalize(&mut self) -> Result<(), Error> {
        match self.threads {
            0 => {
                tracing::info!("Thread configuration is zero, using the number of logical CPUs: {}.", *LOGICAL_CPUS);

                self.threads = *LOGICAL_CPUS;
            }
            _ => {
                tracing::debug!("Configuration specifies {} threads.", self.threads);
            }
        }

        match self.stack_size {
            0 => {
                tracing::info!(
                    "Stack size configuration is zero, using the maximum size of {} bytes.",
                    MAXIMUM_STACK_SIZE
                );

                self.stack_size = MAXIMUM_STACK_SIZE;
            }
            s if s > MAXIMUM_STACK_SIZE => {
                tracing::warn!(
                    "Stack size configuration is too large, reducing to maximum size of {} bytes.",
                    MAXIMUM_STACK_SIZE
                );

                self.stack_size = MAXIMUM_STACK_SIZE;
            }
            s if s < MINIMUM_STACK_SIZE => {
                tracing::warn!(
                    "Stack size configuration is too small, increasing to minimum size of {} bytes.",
                    MINIMUM_STACK_SIZE
                );

                self.stack_size = MINIMUM_STACK_SIZE;
            }
            _ => {
                tracing::debug!("Configuration specifies a stack size of {} bytes.", self.stack_size);
            }
        }

        self.source.normalize()?;

        Ok(())
    }
}

#[cfg(all(test, not(target_os = "windows")))]
mod tests {
    use core::str;
    use std::fs;

    use pretty_assertions::assert_eq;
    use tempfile::env::temp_dir;

    use super::*;

    #[test]
    fn test_take_defaults() {
        let workspace_path = temp_dir().join("workspace-0");
        std::fs::create_dir_all(&workspace_path).unwrap();

        let config = temp_env::with_vars(
            [
                ("HOME", temp_dir().to_str()),
                ("MAGO_THREADS", None),
                ("MAGO_PHP_VERSION", None),
                ("MAGO_ALLOW_UNSUPPORTED_PHP_VERSION", None),
            ],
            || Configuration::load(Some(workspace_path), None, None, None, false).unwrap(),
        );

        assert_eq!(config.threads, *LOGICAL_CPUS)
    }

    #[test]
    fn test_toml_has_precedence_when_multiple_configs_present() {
        let workspace_path = temp_dir().join("workspace-with-multiple-configs");
        std::fs::create_dir_all(&workspace_path).unwrap();

        create_tmp_file("threads = 3", &workspace_path, "toml");
        create_tmp_file("threads: 2\nphp-version: \"7.4.0\"", &workspace_path, "yaml");
        create_tmp_file("{\"threads\": 1,\"php-version\":\"8.1.0\"}", &workspace_path, "json");

        let config = Configuration::load(Some(workspace_path), None, None, None, false).unwrap();

        assert_eq!(config.threads, 3);
        assert_eq!(config.php_version.to_string(), DEFAULT_PHP_VERSION.to_string())
    }

    #[test]
    fn test_env_config_override_all_others() {
        let workspace_path = temp_dir().join("workspace-1");
        let config_path = temp_dir().join("config-1");

        std::fs::create_dir_all(&workspace_path).unwrap();
        std::fs::create_dir_all(&config_path).unwrap();

        let config_file_path = create_tmp_file("threads = 1", &config_path, "toml");
        create_tmp_file("threads = 2", &workspace_path, "toml");

        let config = temp_env::with_vars(
            [
                ("HOME", None),
                ("MAGO_THREADS", Some("3")),
                ("MAGO_PHP_VERSION", None),
                ("MAGO_ALLOW_UNSUPPORTED_PHP_VERSION", None),
            ],
            || Configuration::load(Some(workspace_path), Some(&config_file_path), None, None, false).unwrap(),
        );

        assert_eq!(config.threads, 3);
    }

    #[test]
    fn test_config_cancel_workspace() {
        let workspace_path = temp_dir().join("workspace-2");
        let config_path = temp_dir().join("config-2");

        std::fs::create_dir_all(&workspace_path).unwrap();
        std::fs::create_dir_all(&config_path).unwrap();

        create_tmp_file("threads = 2\nphp-version = \"7.4.0\"", &workspace_path, "toml");

        let config_file_path = create_tmp_file("threads = 1", &config_path, "toml");
        let config = temp_env::with_vars(
            [
                ("HOME", None::<&str>),
                ("MAGO_THREADS", None),
                ("MAGO_PHP_VERSION", None),
                ("MAGO_ALLOW_UNSUPPORTED_PHP_VERSION", None),
            ],
            || Configuration::load(Some(workspace_path), Some(&config_file_path), None, None, false).unwrap(),
        );

        assert_eq!(config.threads, 1);
        assert_eq!(config.php_version.to_string(), DEFAULT_PHP_VERSION.to_string());
    }

    #[test]
    fn test_workspace_has_precedence_over_global() {
        let home_path = temp_dir().join("home-3");
        let xdg_config_home_path = temp_dir().join("xdg-config-home-3");
        let workspace_path = temp_dir().join("workspace-3");

        // Clean up any existing directories from previous test runs
        let _ = std::fs::remove_dir_all(&home_path);
        let _ = std::fs::remove_dir_all(&xdg_config_home_path);
        let _ = std::fs::remove_dir_all(&workspace_path);

        std::fs::create_dir_all(&home_path).unwrap();
        std::fs::create_dir_all(&xdg_config_home_path).unwrap();
        std::fs::create_dir_all(&workspace_path).unwrap();

        create_tmp_file("threads: 2\nphp-version: \"8.1.0\"", &workspace_path.to_owned(), "yaml");
        create_tmp_file("threads = 3\nphp-version = \"7.4.0\"", &home_path, "toml");
        create_tmp_file("source.excludes = [\"yes\"]", &xdg_config_home_path, "toml");

        let config = temp_env::with_vars(
            [
                ("HOME", Some(home_path)),
                ("XDG_CONFIG_HOME", Some(xdg_config_home_path)),
                ("MAGO_THREADS", None),
                ("MAGO_PHP_VERSION", None),
                ("MAGO_ALLOW_UNSUPPORTED_PHP_VERSION", None),
            ],
            || Configuration::load(Some(workspace_path.clone()), None, None, None, false).unwrap(),
        );

        assert_eq!(config.threads, 2);
        assert_eq!(config.php_version.to_string(), "8.1.0".to_string());
        assert_eq!(config.source.excludes, Vec::<String>::new());
    }

    fn create_tmp_file(config_content: &str, folder: &PathBuf, extension: &str) -> PathBuf {
        fs::create_dir_all(folder).unwrap();
        let config_path = folder.join(CONFIGURATION_FILE_NAME).with_extension(extension);
        fs::write(&config_path, config_content).unwrap();
        config_path
    }
}
