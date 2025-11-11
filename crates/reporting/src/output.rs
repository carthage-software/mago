use std::io::Write;
use std::io::{self};
use std::sync::Arc;
use std::sync::Mutex;

/// Target for report output.
#[derive(strum::Display, strum::EnumString, strum::VariantNames)]
#[strum(serialize_all = "kebab-case")]
#[derive(Default)]
pub enum ReportingTarget {
    /// Write to standard output.
    #[strum(serialize = "stdout", serialize = "out")]
    Stdout,
    /// Write to standard error.
    #[strum(serialize = "stderr", serialize = "err")]
    #[default]
    Stderr,
    /// Write to a custom writer.
    #[strum(disabled)]
    Writer(Arc<Mutex<Box<dyn Write + Send>>>),
}

impl Clone for ReportingTarget {
    fn clone(&self) -> Self {
        match self {
            ReportingTarget::Stdout => ReportingTarget::Stdout,
            ReportingTarget::Stderr => ReportingTarget::Stderr,
            ReportingTarget::Writer(writer) => ReportingTarget::Writer(writer.clone()),
        }
    }
}

impl std::fmt::Debug for ReportingTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportingTarget::Stdout => write!(f, "ReportingTarget::Stdout"),
            ReportingTarget::Stderr => write!(f, "ReportingTarget::Stderr"),
            ReportingTarget::Writer(_) => write!(f, "ReportingTarget::Writer(..)"),
        }
    }
}

impl ReportingTarget {
    /// Create a target that writes to a shared buffer.
    ///
    /// This is useful for testing where you want to capture the output.
    ///
    /// # Returns
    ///
    /// A tuple of (target, buffer) where the buffer can be read after reporting.
    pub fn buffer() -> (Self, Arc<Mutex<Vec<u8>>>) {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer_buffer = buffer.clone();
        let target = ReportingTarget::Writer(Arc::new(Mutex::new(Box::new(BufferWriter { buffer: writer_buffer }))));
        (target, buffer)
    }

    /// Resolve this target to an actual writer.
    ///
    /// # Returns
    ///
    /// A boxed writer that can be written to.
    pub(crate) fn resolve(&self) -> Box<dyn Write + '_> {
        match self {
            ReportingTarget::Stdout => Box::new(io::stdout()),
            ReportingTarget::Stderr => Box::new(io::stderr()),
            ReportingTarget::Writer(writer) => Box::new(LockedWriter { writer: writer.clone() }),
        }
    }
}

/// A wrapper that provides Write implementation for Arc<Mutex<Box<dyn Write>>>.
struct LockedWriter {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl Write for LockedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.lock().unwrap().flush()
    }
}

/// A writer that writes to a shared buffer.
struct BufferWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl Write for BufferWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buffer.lock().unwrap().flush()
    }
}
