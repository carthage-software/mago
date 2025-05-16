use std::env::home_dir;
use std::path::PathBuf;

use config::Config;
use config::ConfigBuilder;
use config::Environment;
use config::File;
use config::FileFormat;
use config::Value;
use config::ValueKind;
use config::builder::BuilderState;
use serde::Deserialize;

use mago_php_version::PHPVersion;

use crate::config::formatter::FormatterConfiguration;
use crate::config::linter::LinterConfiguration;
use crate::config::source::SourceConfiguration;
use crate::consts::*;
use crate::error::Error;

pub mod formatter;
pub mod linter;
pub mod source;

/// Configuration options for mago.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
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
    pub format: FormatterConfiguration,

    /// The log filter.
    ///
    /// This is not a configuration option, but it is included here to allow specifying the log filter
    /// in the environment using `MAGO_LOG`.
    ///
    /// If this field is to be removed, serde will complain about an unknown field in the configuration
    /// when `MAGO_LOG` is set due to the `deny_unknown_fields` attribute and the use of `Environment` source.
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
    /// 4. A TOML file named `mago.toml` in the `$HOME` directory.
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
        file: Option<PathBuf>,
        php_version: Option<PHPVersion>,
        threads: Option<usize>,
        allow_unsupported_php_version: bool,
    ) -> Result<Configuration, Error> {
        let workspace_dir = workspace.clone().unwrap_or_else(|| CURRENT_DIR.to_path_buf());

        let mut builder = Config::builder();
        if let Some(file) = file {
            builder = builder.add_source(File::from(file).required(true).format(FileFormat::Toml));
        } else {
            if let Some(home_dir) = home_dir() {
                builder = builder
                    .add_source(File::from(home_dir.join(CONFIGURATION_FILE)).required(false).format(FileFormat::Toml));
            }

            builder = builder
                .add_source(File::from(workspace_dir.join(CONFIGURATION_FILE)).required(false).format(FileFormat::Toml))
        }

        builder = builder.add_source(Environment::with_prefix(ENVIRONMENT_PREFIX));

        let mut configuration = Configuration::from_workspace(workspace_dir);

        configuration = configuration.configure(builder)?.build()?.try_deserialize::<Configuration>()?;

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
            format: FormatterConfiguration::default(),
            log: Value::new(None, ValueKind::Nil),
        }
    }
}

trait ConfigurationEntry {
    /// Configures the builder with the entry.
    fn configure<St: BuilderState>(self, builder: ConfigBuilder<St>) -> Result<ConfigBuilder<St>, Error>;

    fn normalize(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl ConfigurationEntry for Configuration {
    fn configure<St: BuilderState>(self, builder: ConfigBuilder<St>) -> Result<ConfigBuilder<St>, Error> {
        let mut builder = builder
            .set_default("threads", Value::new(None, ValueKind::U64(self.threads as u64)))?
            .set_default("stack_size", Value::new(None, ValueKind::U64(self.stack_size as u64)))?
            .set_default("php_version", Value::new(None, ValueKind::String(self.php_version.to_string())))?
            .set_default("allow_unsupported_php_version", self.allow_unsupported_php_version)?
            .set_default("log", self.log)?;

        builder = self.source.configure(builder)?;
        builder = self.linter.configure(builder)?;
        builder = self.format.configure(builder)?;

        Ok(builder)
    }

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
        self.linter.normalize()?;

        Ok(())
    }
}

#[cfg(all(test, not(target_os = "windows")))]
mod tests {
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

        let config_file_path = create_tmp_file("threads = 1", &config_path);
        create_tmp_file("threads = 2", &workspace_path);

        let config = temp_env::with_vars(
            [
                ("HOME", None),
                ("MAGO_THREADS", Some("3")),
                ("MAGO_PHP_VERSION", None),
                ("MAGO_ALLOW_UNSUPPORTED_PHP_VERSION", None),
            ],
            || Configuration::load(Some(workspace_path), Some(config_file_path), None, None, false).unwrap(),
        );

        assert_eq!(config.threads, 3);
    }

    #[test]
    fn test_config_cancel_workspace() {
        let workspace_path = temp_dir().join("workspace-2");
        let config_path = temp_dir().join("config-2");

        std::fs::create_dir_all(&workspace_path).unwrap();
        std::fs::create_dir_all(&config_path).unwrap();

        create_tmp_file("threads = 2\nphp_version = \"7.4.0\"", &workspace_path);

        let config_file_path = create_tmp_file("threads = 1", &config_path);
        let config = temp_env::with_vars(
            [
                ("HOME", None::<&str>),
                ("MAGO_THREADS", None),
                ("MAGO_PHP_VERSION", None),
                ("MAGO_ALLOW_UNSUPPORTED_PHP_VERSION", None),
            ],
            || Configuration::load(Some(workspace_path), Some(config_file_path), None, None, false).unwrap(),
        );

        assert_eq!(config.threads, 1);
        assert_eq!(config.php_version.to_string(), DEFAULT_PHP_VERSION.to_string());
    }

    #[test]
    fn test_merge_workspace_override_global() {
        let home_path = temp_dir().join("home-3");
        let workspace_path = temp_dir().join("workspace-3");

        std::fs::create_dir_all(&home_path).unwrap();
        std::fs::create_dir_all(&workspace_path).unwrap();

        create_tmp_file("threads = 3\nphp_version = \"7.4.0\"", &home_path);
        create_tmp_file("threads = 2", &workspace_path);

        let config = temp_env::with_vars(
            [
                ("HOME", Some(home_path)),
                ("MAGO_THREADS", None),
                ("MAGO_PHP_VERSION", None),
                ("MAGO_ALLOW_UNSUPPORTED_PHP_VERSION", None),
            ],
            || Configuration::load(Some(workspace_path), None, None, None, false).unwrap(),
        );

        assert_eq!(config.threads, 2);
        assert_eq!(config.php_version.to_string(), "7.4.0".to_string());
    }

    fn create_tmp_file(config_content: &str, folder: &PathBuf) -> PathBuf {
        fs::create_dir_all(folder).unwrap();
        let config_path = folder.join(CONFIGURATION_FILE);
        fs::write(&config_path, config_content).unwrap();
        config_path
    }
}
