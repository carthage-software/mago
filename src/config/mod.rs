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
use serde::Serialize;

use crate::config::formatter::FormatterConfiguration;
use crate::config::linter::LinterConfiguration;
use crate::config::source::SourceConfiguration;
use crate::consts::*;
use crate::error::Error;

pub mod formatter;
pub mod linter;
pub mod source;

/// Configuration options for mago.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Configuration {
    /// The number of threads to use.
    pub threads: usize,

    /// The size of the stack for each thread.
    pub stack_size: usize,

    /// Configuration options for source discovery.
    pub source: SourceConfiguration,

    /// Configuration options for the linter.
    #[serde(default)]
    pub linter: LinterConfiguration,

    /// Configuration options for the formatter.
    #[serde(default)]
    pub format: FormatterConfiguration,
}

impl Configuration {
    pub fn load() -> Result<Configuration, Error> {
        let builder = Config::builder()
            .add_source(File::with_name(CONFIGURATION_FILE).required(false).format(FileFormat::Toml))
            .add_source(Environment::with_prefix(ENVIRONMENT_PREFIX).try_parsing(true).list_separator(","));

        tracing::debug!("loading configuration from sources");

        let mut this = Configuration::from_root(CURRENT_DIR.to_path_buf())
            .configure(builder)?
            .build()?
            .try_deserialize::<Configuration>()?;

        this.normalize()?;

        Ok(this)
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
            source: SourceConfiguration::from_root(root),
            threads: *LOGICAL_CPUS,
            stack_size: DEFAULT_STACK_SIZE,
            linter: LinterConfiguration::default(),
            format: FormatterConfiguration::default(),
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
        tracing::trace!("configuring configuration entry");

        let mut builder = builder
            .set_default("threads", Value::new(None, ValueKind::U64(self.threads as u64)))?
            .set_default("stack_size", Value::new(None, ValueKind::U64(self.stack_size as u64)))?;

        tracing::trace!("configuring source entry");
        builder = self.source.configure(builder)?;

        tracing::trace!("configuring linter entry");
        builder = self.linter.configure(builder)?;

        tracing::trace!("configuring formatter entry");
        builder = self.format.configure(builder)?;

        Ok(builder)
    }

    fn normalize(&mut self) -> Result<(), Error> {
        if self.threads == 0 {
            tracing::info!("thread configuration is zero, using the number of logical CPUs: {}", *LOGICAL_CPUS);

            self.threads = *LOGICAL_CPUS;
        } else {
            tracing::debug!("configuration specifies {} threads", self.threads);
        }

        if self.stack_size == 0 {
            tracing::info!("stack size configuration is zero, using the maximum size of {} bytes", MAXIMUM_STACK_SIZE);

            self.stack_size = MAXIMUM_STACK_SIZE;
        } else if self.stack_size > MAXIMUM_STACK_SIZE {
            tracing::warn!(
                "stack size configuration is too large, reducing to maximum size of {} bytes",
                MAXIMUM_STACK_SIZE
            );

            self.stack_size = MAXIMUM_STACK_SIZE;
        } else if self.stack_size < MINIMUM_STACK_SIZE {
            tracing::warn!(
                "stack size configuration is too small, increasing to minimum size of {} bytes",
                MINIMUM_STACK_SIZE
            );

            self.stack_size = MINIMUM_STACK_SIZE;
        } else {
            tracing::debug!("configuration specifies a stack size of {} bytes", self.stack_size);
        }

        self.source.normalize()?;
        self.linter.normalize()?;

        Ok(())
    }
}
