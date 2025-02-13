use std::path::PathBuf;

use config::builder::BuilderState;
use config::Config;
use config::ConfigBuilder;
use config::Environment;
use config::File;
use config::FileFormat;
use config::Value;
use config::ValueKind;
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
    pub fn load_from(root: PathBuf) -> Result<Configuration, Error> {
        let configuration_file = root.join(CONFIGURATION_FILE);

        let builder = Config::builder()
            .add_source(File::from(configuration_file).required(false).format(FileFormat::Toml))
            .add_source(Environment::with_prefix(ENVIRONMENT_PREFIX));

        let mut configuration =
            Configuration::from_root(root).configure(builder)?.build()?.try_deserialize::<Configuration>()?;

        configuration.normalize()?;

        Ok(configuration)
    }

    pub fn load() -> Result<Configuration, Error> {
        Self::load_from(CURRENT_DIR.to_path_buf())
    }

    /// Creates a new `Configuration` with the given root directory.
    ///
    /// # Arguments
    ///
    /// * `root` - The root directory from which to start scanning.
    ///
    /// # Returns
    ///
    /// A new `Configuration` with the given root directory.
    pub fn from_root(root: PathBuf) -> Self {
        Self {
            threads: *LOGICAL_CPUS,
            stack_size: DEFAULT_STACK_SIZE,
            php_version: DEFAULT_PHP_VERSION,
            source: SourceConfiguration::from_root(root),
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
