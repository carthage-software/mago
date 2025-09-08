use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

/// The format to use when writing the report.
pub use termcolor::*;

use crate::reporter::ReportingTarget;

/// A thread-safe wrapper around `StandardStream`, enabling colorized and styled output to
/// either `stdout` or `stderr`.
#[derive(Debug, Clone)]
pub(crate) struct ReportWriter {
    /// Inner `StandardStream` wrapped in an `Arc<Mutex>` to ensure thread-safe access.
    inner: Arc<Mutex<StandardStream>>,
}

impl ReportWriter {
    /// Creates a new `ReportWriter` for the specified target (`stdout` or `stderr`).
    ///
    /// # Parameters
    ///
    /// - `target`: The output target, either `Target::Stdout` or `Target::Stderr`.
    /// - `with_colors`: A boolean indicating whether to enable colored output.
    ///
    /// # Returns
    ///
    /// A new `ReportWriter` instance configured for the specified target.
    pub fn new(target: ReportingTarget, with_colors: bool) -> Self {
        let stream = match target {
            ReportingTarget::Stdout => {
                StandardStream::stdout(if with_colors { ColorChoice::Auto } else { ColorChoice::Never })
            }
            ReportingTarget::Stderr => {
                StandardStream::stderr(if with_colors { ColorChoice::Auto } else { ColorChoice::Never })
            }
        };

        Self { inner: Arc::new(Mutex::new(stream)) }
    }

    /// Acquires a lock on the internal `StandardStream`, returning a `Gaurd` for performing write operations.
    ///
    /// # Returns
    ///
    /// A `Gaurd` object, which implements `Write` and `WriteColor` traits for text and styled output.
    ///
    /// # Panics
    ///
    /// Panics if the internal `Mutex` is poisoned.
    pub fn lock(&self) -> Gaurd<'_> {
        Gaurd(self.inner.lock().expect("writer lock poisoned, this should never happen"))
    }
}

/// A guard object for safely accessing and writing to the `StandardStream`.
///
/// This struct is created by the `lock` method of `ReportWriter`.
pub(crate) struct Gaurd<'a>(MutexGuard<'a, StandardStream>);

impl WriteColor for Gaurd<'_> {
    /// Sets the color for subsequent output written through this guard.
    ///
    /// # Parameters
    /// - `spec`: A `ColorSpec` describing the desired text styling.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn set_color(&mut self, spec: &ColorSpec) -> std::io::Result<()> {
        self.0.set_color(spec)
    }

    /// Resets the text styling to default.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn reset(&mut self) -> std::io::Result<()> {
        self.0.reset()
    }

    /// Checks whether the underlying stream supports color output.
    ///
    /// # Returns
    ///
    /// `true` if the stream supports color, `false` otherwise.
    fn supports_color(&self) -> bool {
        self.0.supports_color()
    }
}

impl std::io::Write for Gaurd<'_> {
    /// Writes a buffer to the stream.
    ///
    /// # Parameters
    ///
    /// - `buf`: A byte slice containing the data to write.
    ///
    /// # Returns
    ///
    /// The number of bytes written or an error.
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    /// Flushes the stream, ensuring all buffered data is written out.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}
