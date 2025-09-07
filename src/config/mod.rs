use std::env::home_dir;
use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;

use config::Case;
use config::Config;
use config::Environment;
use config::File;
use config::Value;
use config::ValueKind;
use serde::Deserialize;
use serde::Serialize;

use mago_php_version::PHPVersion;

use crate::config::analyzer::AnalyzerConfiguration;
use crate::config::formatter::FormatterConfiguration;
use crate::config::linter::LinterConfiguration;
use crate::config::source::SourceConfiguration;
use crate::consts::*;
use crate::error::Error;

pub mod analyzer;
pub mod formatter;
pub mod linter;
pub mod source;

/// Configuration options for mago.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct Configuration {
    /// The number of threads to use.
    pub threads: usize,

    /// The size of the stack for each thread.
    pub stack_size: usize,

    /// The version of PHP to use.
    pub php_version: PHPVersion,

    /// Whether to allow unsupported PHP versions.
    pub allow_unsupported_php_version: bool,

    /// Configuration options for source discovery.
    pub source: SourceConfiguration,

    /// Configuration options for the linter.
    #[serde(default)]
    pub linter: LinterConfiguration,

    /// Configuration options for the formatter.
    #[serde(default)]
    pub formatter: FormatterConfiguration,

    /// Configuration options for the analyzer.
    #[serde(default)]
    pub analyzer: AnalyzerConfiguration,

    /// The log filter.
    ///
    /// This is not a configuration option, but it is included here to allow specifying the log filter
    /// in the environment using `MAGO_LOG`.
    ///
    /// If this field is to be removed, serde will complain about an unknown field in the configuration
    /// when `MAGO_LOG` is set due to the `deny_unknown_fields` attribute and the use of `Environment` source.
    #[serde(default, skip_serializing)]
    #[allow(dead_code)]
    log: Value,
}

impl Configuration {
    /// Loads the configuration from a file or environment variables.
    ///
    /// This function attempts to load the configuration from the following sources, in order of precedence:
    ///
    /// 1. Environment variables with the prefix `MAGO_`.
    /// 2. A TOML file specified by the `file` argument.
    /// 3. A TOML file named `mago.toml` in the current directory.
    /// 4. A TOML file named `mago.toml` in the `$XDG_CONFIG_HOME` directory.
    /// 5. A TOML file named `mago.toml` in the `$HOME` directory.
    ///
    /// When the `file` argument is set, 3 and 4 are not used at all.
    ///
    /// The loaded configuration is then normalized and validated.
    ///
    /// # Arguments
    ///
    /// * `workspace` - An optional path to the workspace directory.
    /// * `file` - An optional path to a TOML configuration file.
    /// * `php_version` - An optional PHP version to use for the configuration.
    /// * `threads` - An optional number of threads to use for linting and formatting.
    /// * `allow_unsupported_php_version` - Whether to allow unsupported PHP versions.
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded `Configuration`, or an `Error` if the configuration could not be loaded or validated.
    pub fn load(
        workspace: Option<PathBuf>,
        file: Option<&Path>,
        php_version: Option<PHPVersion>,
        threads: Option<usize>,
        allow_unsupported_php_version: bool,
    ) -> Result<Configuration, Error> {
        let workspace_dir = workspace.clone().unwrap_or_else(|| CURRENT_DIR.to_path_buf());
        let workspace_config_path = workspace_dir.join(CONFIGURATION_FILE_NAME);

        let mut configuration = Configuration::from_workspace(workspace_dir);
        let mut builder = Config::builder().add_source(Config::try_from(&configuration)?);

        if let Some(file) = file {
            tracing::debug!("Sourcing configuration from {}.", file.display());

            builder = builder.add_source(File::from(file).required(true));
        } else {
            let global_config_roots = [std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from), home_dir()];
            for global_config_root in global_config_roots {
                let Some(global_config_root) = global_config_root else {
                    continue;
                };

                let global_config_path = global_config_root.join(CONFIGURATION_FILE_NAME);

                tracing::debug!("Sourcing global configuration from {}.", global_config_path.display());

                builder = builder.add_source(File::from(global_config_path).required(false));
            }

            tracing::debug!("Sourcing workspace configuration from {}.", workspace_config_path.display());

            builder = builder.add_source(File::from(workspace_config_path).required(false));
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

    /// Creates a new `Configuration` with the given workspace directory.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The workspace directory from which to start scanning.
    ///
    /// # Returns
    ///
    /// A new `Configuration` with the given workspace directory.
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
            log: Value::new(None, ValueKind::Nil),
        }
    }
}

impl Configuration {
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
    fn test_merge_workspace_override_global() {
        let home_path = temp_dir().join("home-3");
        let xdg_config_home_path = temp_dir().join("xdg-config-home-3");
        let workspace_path = temp_dir().join("workspace-3");

        std::fs::create_dir_all(&home_path).unwrap();
        std::fs::create_dir_all(&xdg_config_home_path).unwrap();
        std::fs::create_dir_all(&workspace_path).unwrap();

        create_tmp_file("threads = 3\nphp-version = \"7.4.0\"", &home_path, "toml");
        create_tmp_file("threads: 2", &workspace_path, "yaml");
        create_tmp_file("source.excludes = [\"yes\"]", &xdg_config_home_path, "toml");

        let config = temp_env::with_vars(
            [
                ("HOME", Some(home_path)),
                ("XDG_CONFIG_HOME", Some(xdg_config_home_path)),
                ("MAGO_THREADS", None),
                ("MAGO_PHP_VERSION", None),
                ("MAGO_ALLOW_UNSUPPORTED_PHP_VERSION", None),
            ],
            || Configuration::load(Some(workspace_path), None, None, None, false).unwrap(),
        );

        assert_eq!(config.threads, 2);
        assert_eq!(config.php_version.to_string(), "7.4.0".to_string());
        assert_eq!(config.source.excludes, vec!["yes".to_string()]);
    }

    fn create_tmp_file(config_content: &str, folder: &PathBuf, extension: &str) -> PathBuf {
        fs::create_dir_all(folder).unwrap();
        let config_path = folder.join(CONFIGURATION_FILE_NAME).with_extension(extension);
        fs::write(&config_path, config_content).unwrap();
        config_path
    }
}
