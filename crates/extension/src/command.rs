use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

/// The command used to start every process in an extension worker pool.
#[derive(Clone)]
pub struct WorkerCommand {
    program: OsString,
    arguments: Vec<OsString>,
    current_directory: Option<PathBuf>,
    environment: Vec<(OsString, OsString)>,
    clear_environment: bool,
}

impl std::fmt::Debug for WorkerCommand {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorkerCommand")
            .field("program", &self.program)
            .field("argument_count", &self.arguments.len())
            .field("current_directory", &self.current_directory)
            .field("environment_keys", &self.environment.iter().map(|(key, _)| key).collect::<Vec<_>>())
            .field("clear_environment", &self.clear_environment)
            .finish()
    }
}

impl WorkerCommand {
    /// Creates a worker command for `program`.
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            arguments: Vec::new(),
            current_directory: None,
            environment: Vec::new(),
            clear_environment: false,
        }
    }

    /// Adds one argument to the worker command.
    #[must_use]
    pub fn with_argument(mut self, argument: impl Into<OsString>) -> Self {
        self.arguments.push(argument.into());
        self
    }

    /// Adds arguments to the worker command.
    #[must_use]
    pub fn with_arguments<I, S>(mut self, arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.arguments.extend(arguments.into_iter().map(Into::into));
        self
    }

    /// Sets the working directory inherited by worker processes.
    #[must_use]
    pub fn with_current_directory(mut self, directory: impl Into<PathBuf>) -> Self {
        self.current_directory = Some(directory.into());
        self
    }

    /// Adds or replaces one environment variable for worker processes.
    #[must_use]
    pub fn with_environment(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.environment.push((key.into(), value.into()));
        self
    }

    /// Prevents worker processes from inheriting Mago's environment.
    #[must_use]
    pub fn with_clear_environment(mut self, clear: bool) -> Self {
        self.clear_environment = clear;
        self
    }

    /// Returns the configured program.
    #[must_use]
    pub fn program(&self) -> &OsStr {
        &self.program
    }

    /// Returns the configured arguments.
    #[must_use]
    pub fn arguments(&self) -> &[OsString] {
        &self.arguments
    }

    /// Returns the configured working directory, if any.
    #[must_use]
    pub fn current_directory(&self) -> Option<&Path> {
        self.current_directory.as_deref()
    }

    pub(crate) fn build(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.arguments);
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        if self.clear_environment {
            command.env_clear();
        }

        if let Some(directory) = &self.current_directory {
            command.current_dir(directory);
        }

        for (key, value) in &self.environment {
            command.env(key, value);
        }

        command
    }
}
