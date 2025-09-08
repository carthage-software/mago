//! This crate provides functionality to pipe terminal output through a pager
//! program (like `less` or `more`) on Unix-like systems. It is designed to be
//! used in command-line applications that generate large amounts of output,
//! allowing users to scroll through the content conveniently.
//!
//! The primary entry point is the `Pager` builder, which allows for configuration
//! and ultimately spawns a pager process via the `spawn()` method. The returned
//! `PagerSession` struct handles the lifecycle of the pager process. When the
//! `PagerSession` is dropped, it gracefully terminates the pager and restores
//! the original standard output.
//!
//! This functionality is only active on Unix-like systems and when the output
//! is being sent to an interactive terminal. On Windows or in non-interactive
//! sessions, it gracefully becomes a no-op.
//!
//! ## Usage
//!
//! ```no_run
//! use mago_pager::Pager;
//!
//! # fn run() -> std::io::Result<()> {
//! // Spawn a pager if the environment is suitable.
//! let _session = Pager::new().spawn()?;
//!
//! // All subsequent writes to stdout will be piped to the pager.
//! println!("This line goes to the pager.");
//! println!("So does this one.");
//!
//! // When `_session` goes out of scope, the pager is closed and stdout is restored.
//! # Ok(())
//! # }
//! ```
//!
//! ## Environment Variables
//!
//! - `PAGER`: Overrides the default pager command.
//! - `NOPAGER`: If set to any value, disables the pager entirely.

use std::env;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::Result;

#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use std::io::Error;
#[cfg(unix)]
use std::io::IsTerminal;
#[cfg(unix)]
use std::io::Write;
#[cfg(unix)]
use std::io::stdout;
#[cfg(unix)]
use std::os::fd::AsRawFd;
#[cfg(unix)]
use std::os::fd::RawFd;
#[cfg(unix)]
use std::process::Child;
#[cfg(unix)]
use std::process::Command;
#[cfg(unix)]
use std::process::Stdio;

/// Environment variable to specify a custom pager command.
pub const DEFAULT_PAGER_ENV: &str = "PAGER";

/// Environment variable to disable the pager.
pub const NOPAGER_ENV: &str = "NOPAGER";

/// The fallback pager command if no other is specified.
pub const DEFAULT_PAGER: &str = "less";

/// A builder for configuring and spawning a terminal pager.
///
/// This struct follows the builder pattern to allow for flexible configuration
/// before spawning the pager process.
#[derive(Debug, Default)]
pub struct Pager {
    /// A pager command explicitly set via a builder method.
    /// This takes precedence over environment variables or defaults.
    command: Option<OsString>,
    /// A fallback command to use if `PAGER` is not set.
    default_command: Option<OsString>,
    /// Additional environment variables to pass to the pager process.
    envs: Vec<(OsString, OsString)>,
}

impl Pager {
    /// Creates a new `Pager` builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a specific command to use for the pager.
    ///
    /// This overrides any pager specified by environment variables or defaults. The command
    /// will be parsed by `shell_words`.
    ///
    /// # Arguments
    ///
    /// * `command` - The pager command to execute (e.g., "less -R").
    pub fn command<S: Into<OsString>>(mut self, command: S) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Sets a default pager command to use if no other pager is configured.
    ///
    /// This is used as a fallback if `PAGER` is not set and no explicit
    /// command was provided via `.command()`.
    ///
    /// # Arguments
    ///
    /// * `command` - The default pager command (e.g., "less").
    pub fn default_command<S: Into<OsString>>(mut self, command: S) -> Self {
        self.default_command = Some(command.into());
        self
    }

    /// Adds environment variables to be set for the pager process.
    ///
    /// # Arguments
    ///
    /// * `envs` - An iterator of key-value pairs representing the environment variables.
    pub fn envs<I, K, V>(mut self, envs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<OsString>,
        V: Into<OsString>,
    {
        self.envs = envs.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        self
    }

    /// Determines the appropriate pager command based on configuration and environment.
    ///
    /// The selection process follows this order of precedence:
    /// 1. The command set via `Pager::command()`.
    /// 2. The `PAGER` environment variable.
    /// 3. The command set via `Pager::default_command()`.
    /// 4. The hardcoded `DEFAULT_PAGER` ("more").
    ///
    /// If the `NOPAGER` environment variable is set, this method returns `None`.
    #[cfg(unix)]
    fn determine_command(&self) -> Option<OsString> {
        if env::var_os(NOPAGER_ENV).is_some() {
            return None;
        }

        self.command
            .clone()
            .or_else(|| env::var_os(DEFAULT_PAGER_ENV))
            .or_else(|| self.default_command.clone())
            .or_else(|| Some(OsStr::new(DEFAULT_PAGER).into()))
    }

    /// Spawns the pager process if the conditions are met.
    ///
    /// This method will only attempt to spawn a pager if:
    /// - The target OS is Unix-like.
    /// - The current process is running in an interactive terminal.
    /// - A pager command is available (i.e., not disabled by `NOPAGER`).
    ///
    /// On Windows or in non-interactive sessions, it returns a no-op `PagerSession`
    /// that does nothing and has no overhead.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `PagerSession` on success, or an `std::io::Error`
    /// if the pager process fails to spawn or redirect output.
    pub fn spawn(self) -> Result<PagerSession> {
        #[cfg(not(unix))]
        {
            Ok(PagerSession {})
        }

        #[cfg(unix)]
        {
            if !stdout().is_terminal() {
                // Not a TTY, so no pager is needed.
                return Ok(PagerSession::new_inactive());
            }

            let Some(pager_command) = self.determine_command() else {
                // Pager is disabled via NOPAGER env var.
                return Ok(PagerSession::new_inactive());
            };

            // The command might contain arguments, so we need to parse it.
            let pager_str =
                pager_command.to_str().ok_or_else(|| Error::other("pager command contains non-UTF8 characters"))?;

            let args = shell_words::split(pager_str).map_err(Error::other)?;
            let Some((program, args)) = args.split_first() else {
                return Err(Error::other("pager command is empty"));
            };

            // Spawn the pager process with its stdin piped.
            let pager_process = Command::new(program).args(args).envs(self.envs).stdin(Stdio::piped()).spawn()?;

            // This is the file descriptor for the pager's stdin.
            let pager_stdin_fd = pager_process.stdin.as_ref().expect("stdin must be piped as configured").as_raw_fd();

            // The following block redirects the process's standard output to the
            // pager's standard input. We must be careful to restore the original
            // stdout when the PagerSession is dropped.
            let original_stdout_fd;
            unsafe {
                // 1. Save the original stdout file descriptor. `dup` creates a copy.
                original_stdout_fd = libc::dup(libc::STDOUT_FILENO);
                if original_stdout_fd < 0 {
                    return Err(Error::last_os_error());
                }

                // 2. Redirect stdout to the pager's stdin. `dup2` closes the
                //    current STDOUT_FILENO and makes it a copy of `pager_stdin_fd`.
                if libc::dup2(pager_stdin_fd, libc::STDOUT_FILENO) != libc::STDOUT_FILENO {
                    // In case of failure, close the saved fd and return the error.
                    libc::close(original_stdout_fd);
                    return Err(Error::last_os_error());
                }
            }

            Ok(PagerSession::new_active(pager_process, original_stdout_fd))
        }
    }

    /// Executes a closure within a pager session, handling setup and teardown automatically.
    ///
    /// This is a convenience method that simplifies using the pager. It spawns a pager
    /// session, executes the provided `action` closure, and then ensures the session
    /// is closed, restoring the original standard output. This avoids the need to
    /// manually manage the `PagerSession` guard variable.
    ///
    /// The closure receives a boolean argument, `is_active`, which is `true` if
    /// the output is being piped to a pager, and `false` otherwise. This allows for
    /// conditional logic within your application, such as printing a different header
    /// or footer when paging is active.
    ///
    /// # Arguments
    ///
    /// * `action`: A closure to execute. It receives a boolean indicating if the
    ///   pager is active and should return a `Result`.
    ///
    /// # Returns
    ///
    /// This method returns the `Result<T>` where `T` is the return type of the closure.
    /// If spawning the pager fails, it returns an `std::io::Error`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use mago_pager::Pager;
    /// use std::io::Result;
    ///
    /// fn main() -> Result<()> {
    ///     Pager::new().page(|is_active| {
    ///         if is_active {
    ///             println!("--- Pager is active. Showing 1000 lines. ---");
    ///         }
    ///
    ///         for i in 0..1000 {
    ///             println!("This is line number {}", i);
    ///         }
    ///     })?;
    ///
    ///     // Pager is automatically closed here.
    ///     Ok(())
    /// }
    /// ```
    pub fn page<T, F>(self, action: F) -> Result<T>
    where
        F: FnOnce(bool) -> T,
    {
        let session = self.spawn()?;
        let result = action(session.is_active());
        drop(session);
        Ok(result)
    }
}

/// Represents an active pager session.
///
/// This struct's main responsibility is to clean up resources when it is dropped (RAII).
/// On Unix, it holds the pager's child process and the original stdout file descriptor.
/// On other platforms (like Windows), it is a zero-sized, no-op struct.
#[must_use = "if the pager session is not bound, it will be dropped immediately, closing the pager"]
pub struct PagerSession {
    /// The spawned pager process. Only present on Unix.
    #[cfg(unix)]
    process: Option<Child>,
    /// The saved file descriptor for the original standard output. Only present on Unix.
    #[cfg(unix)]
    original_stdout_fd: Option<RawFd>,
}

#[cfg(unix)]
impl PagerSession {
    /// Creates a new, active pager session that will manage the child process.
    fn new_active(process: Child, original_stdout_fd: RawFd) -> Self {
        Self { process: Some(process), original_stdout_fd: Some(original_stdout_fd) }
    }

    /// Creates a new, inactive pager session (a no-op).
    fn new_inactive() -> Self {
        Self { process: None, original_stdout_fd: None }
    }
}

impl PagerSession {
    /// Returns `true` if a pager process was successfully spawned.
    ///
    /// On non-Unix platforms, this will always return `false`.
    pub fn is_active(&self) -> bool {
        #[cfg(unix)]
        {
            self.process.is_some()
        }
        #[cfg(not(unix))]
        {
            false
        }
    }
}

/// The Drop implementation is crucial for restoring the terminal's state on Unix.
#[cfg(unix)]
impl Drop for PagerSession {
    fn drop(&mut self) {
        let Some(mut process) = self.process.take() else {
            // Nothing to do if the process was never started.
            return;
        };

        let Some(original_stdout_fd) = self.original_stdout_fd.take() else {
            // Should not happen if process is Some, but we guard anyway.
            return;
        };

        unsafe {
            // Before restoring stdout, we must flush any buffered output from
            // Rust's `stdout` handle. If we don't, this buffered content might get
            // printed to the terminal *after* the pager has already exited.
            //
            // To prevent this, we temporarily redirect stdout to `/dev/null`,
            // flush the buffer (sending its contents to oblivion), and then
            // finally restore the original stdout.
            if let Ok(null_device) = File::create("/dev/null") {
                // Redirect stdout to /dev/null
                libc::dup2(null_device.as_raw_fd(), libc::STDOUT_FILENO);
            }

            // Now, flush the buffer. We ignore potential errors here, as there's
            // little we can do to recover.
            let _ = stdout().flush();

            // Finally, restore the original stdout file descriptor.
            libc::dup2(original_stdout_fd, libc::STDOUT_FILENO);
            // And close the duplicated file descriptor we created.
            libc::close(original_stdout_fd);
        }

        // Wait for the pager process to exit. We ignore the result because
        // we can't do anything about an error at this point. The user might
        // have quit the pager (e.g., with 'q'), which can result in a non-zero exit status.
        let _ = process.wait();
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Environment variables are a global resource, so tests that modify them
    // must be serialized to prevent race conditions. A static mutex is a
    // common way to achieve this.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// A RAII guard to temporarily modify an environment variable for a test.
    struct EnvGuard {
        key: OsString,
        old_value: Option<OsString>,
    }

    impl EnvGuard {
        fn set(key: &str, value: &str) -> Self {
            let key = OsString::from(key);
            let old_value = env::var_os(&key);
            unsafe {
                env::set_var(&key, value);
            }

            Self { key, old_value }
        }

        fn remove(key: &str) -> Self {
            let key = OsString::from(key);
            let old_value = env::var_os(&key);
            unsafe {
                env::remove_var(&key);
            }

            Self { key, old_value }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            unsafe {
                if let Some(old_value) = &self.old_value {
                    env::set_var(&self.key, old_value);
                } else {
                    env::remove_var(&self.key);
                }
            }
        }
    }

    fn assert_command(pager: &Pager, expected: &str) {
        assert_eq!(pager.determine_command(), Some(OsStr::new(expected).into()));
    }

    #[test]
    fn test_pager_disabled_by_nopager_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set(NOPAGER_ENV, "1");

        let pager = Pager::new();
        assert!(pager.determine_command().is_none());
    }

    #[test]
    fn test_default_fallback_pager_is_more() {
        let _lock = ENV_LOCK.lock().unwrap();
        // Ensure relevant env vars are not set
        let _guard1 = EnvGuard::remove(NOPAGER_ENV);
        let _guard2 = EnvGuard::remove(DEFAULT_PAGER_ENV);

        let pager = Pager::new();
        assert_command(&pager, DEFAULT_PAGER); // "more"
    }

    #[test]
    fn test_pager_reads_from_mago_pager_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set(DEFAULT_PAGER_ENV, "less -R");

        let pager = Pager::new();
        assert_command(&pager, "less -R");
    }

    #[test]
    fn test_explicit_command_overrides_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set(DEFAULT_PAGER_ENV, "less");

        let pager = Pager::new().command("bat");
        assert_command(&pager, "bat");
    }

    #[test]
    fn test_explicit_default_command_is_used_as_fallback() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::remove(DEFAULT_PAGER_ENV);

        let pager = Pager::new().default_command("less");
        assert_command(&pager, "less");
    }

    #[test]
    fn test_env_overrides_explicit_default_command() {
        let _lock = ENV_LOCK.lock().unwrap();
        let _guard = EnvGuard::set(DEFAULT_PAGER_ENV, "bat");

        let pager = Pager::new().default_command("less");
        assert_command(&pager, "bat");
    }
}
