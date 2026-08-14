use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use mago_extension::WorkerCommand;
use mago_extension::WorkerPoolOptions;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;

const DEFAULT_MAXIMUM_PAYLOAD_SIZE: usize = 64 * 1024 * 1024;
const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_SHUTDOWN_TIMEOUT_MS: u64 = 250;
const DEFAULT_STDERR_TAIL_SIZE: usize = 64 * 1024;

/// Configuration for one external extension host pool.
///
/// One command is replicated into a pool of identical worker processes. Every
/// worker in that pool advertises the same set of logical extensions, and each
/// extension may expose both linter rules and analyzer capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
pub struct ExtensionHostConfiguration {
    /// Whether this extension host is active.
    pub enabled: bool,

    /// Program and arguments used to start each worker, without shell parsing.
    ///
    /// The first element is the executable. Remaining elements are passed as
    /// literal arguments. A bare executable name is resolved through `PATH`;
    /// a relative path with a directory component is resolved from the
    /// effective configuration file's directory.
    pub command: Vec<String>,

    /// Number of worker processes.
    ///
    /// Zero uses an adaptive pool that can grow to Mago's global thread count.
    /// A non-zero value starts exactly that many workers.
    pub workers: usize,

    /// Working directory for workers.
    ///
    /// Relative paths resolve from the effective configuration file's
    /// directory. When omitted, that directory itself is used.
    pub working_directory: Option<PathBuf>,

    /// Environment variables added to or replacing inherited values.
    pub environment: BTreeMap<String, String>,

    /// Whether workers inherit Mago's process environment.
    pub inherit_environment: bool,

    /// Maximum bytes accepted in one extension protocol frame.
    pub maximum_payload_size: usize,

    /// Maximum duration of one request, in milliseconds.
    pub request_timeout_ms: u64,

    /// Grace period for worker shutdown, in milliseconds.
    pub shutdown_timeout_ms: u64,

    /// Number of trailing worker stderr bytes retained for diagnostics.
    pub stderr_tail_size: usize,

    #[serde(skip)]
    #[schemars(skip)]
    base_directory: PathBuf,
}

impl Default for ExtensionHostConfiguration {
    fn default() -> Self {
        Self {
            enabled: true,
            command: Vec::new(),
            workers: 0,
            working_directory: None,
            environment: BTreeMap::new(),
            inherit_environment: true,
            maximum_payload_size: DEFAULT_MAXIMUM_PAYLOAD_SIZE,
            request_timeout_ms: DEFAULT_REQUEST_TIMEOUT_MS,
            shutdown_timeout_ms: DEFAULT_SHUTDOWN_TIMEOUT_MS,
            stderr_tail_size: DEFAULT_STDERR_TAIL_SIZE,
            base_directory: PathBuf::new(),
        }
    }
}

impl ExtensionHostConfiguration {
    pub(super) fn normalize(&mut self, name: &str, base_directory: &Path) -> Result<(), String> {
        self.base_directory = base_directory.to_path_buf();

        if !self.enabled {
            return Ok(());
        }

        if self.command.first().is_none_or(String::is_empty) {
            return Err(format!("extension host `{name}` must define a non-empty command"));
        }

        if self.maximum_payload_size == 0 || self.maximum_payload_size > u32::MAX as usize {
            return Err(format!("extension host `{name}` maximum-payload-size must be between 1 and {}", u32::MAX));
        }

        if self.request_timeout_ms == 0 {
            return Err(format!("extension host `{name}` request-timeout-ms must be greater than zero"));
        }

        Ok(())
    }

    /// Returns the fixed pool size or the adaptive pool's worker ceiling.
    #[must_use]
    pub fn worker_count(&self, mago_threads: usize) -> NonZeroUsize {
        NonZeroUsize::new(if self.workers == 0 { mago_threads } else { self.workers }).unwrap_or(NonZeroUsize::MIN)
    }

    /// Builds the shell-free command used for every worker process.
    ///
    /// Returns `None` only for a disabled host without a command.
    #[must_use]
    pub fn worker_command(&self) -> Option<WorkerCommand> {
        let program = self.command.first()?;
        let program_path = Path::new(program);
        let program = if program_path.is_relative() && program_path.components().count() > 1 {
            self.base_directory.join(program_path).into_os_string()
        } else {
            program.as_str().into()
        };

        let working_directory = self
            .working_directory
            .as_deref()
            .map_or_else(|| self.base_directory.clone(), |directory| self.resolve_path(directory));
        let mut command = WorkerCommand::new(program)
            .with_arguments(self.command.iter().skip(1).map(String::as_str))
            .with_current_directory(working_directory)
            .with_clear_environment(!self.inherit_environment);
        for (key, value) in &self.environment {
            command = command.with_environment(key, value);
        }

        Some(command)
    }

    /// Builds the process and protocol limits for this host's worker pool.
    #[must_use]
    pub fn worker_pool_options(&self) -> WorkerPoolOptions {
        WorkerPoolOptions {
            maximum_payload_size: self.maximum_payload_size,
            request_timeout: Duration::from_millis(self.request_timeout_ms),
            shutdown_timeout: Duration::from_millis(self.shutdown_timeout_ms),
            stderr_tail_size: self.stderr_tail_size,
        }
    }

    fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_relative() { self.base_directory.join(path) } else { path.to_path_buf() }
    }
}
