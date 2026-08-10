use std::ffi::OsString;
use std::io;
use std::time::Duration;

use crate::protocol::ProtocolVersion;

/// Errors encountered while reading or writing an extension protocol frame.
#[derive(Debug)]
pub enum ProtocolError {
    /// The underlying stream failed.
    Io(io::Error),
    /// The frame did not begin with the Mago protocol magic bytes.
    InvalidMagic([u8; 4]),
    /// The peer is using an incompatible protocol major version.
    UnsupportedVersion(ProtocolVersion),
    /// The frame contains an unknown message kind.
    UnknownFrameKind(u8),
    /// The frame uses non-zero reserved header bytes.
    ReservedHeaderBits(u16),
    /// The payload exceeds the configured frame-size limit.
    FrameTooLarge { length: usize, maximum: usize },
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "extension protocol I/O failed: {error}"),
            Self::InvalidMagic(magic) => write!(
                formatter,
                "invalid extension frame magic: {:02x}{:02x}{:02x}{:02x}",
                magic[0], magic[1], magic[2], magic[3]
            ),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported extension protocol version {}.{}", version.major, version.minor)
            }
            Self::UnknownFrameKind(kind) => write!(formatter, "unknown extension frame kind {kind}"),
            Self::ReservedHeaderBits(bits) => {
                write!(formatter, "extension frame reserved header bits are non-zero: {bits:#06x}")
            }
            Self::FrameTooLarge { length, maximum } => {
                write!(formatter, "extension frame is {length} bytes, exceeding the {maximum}-byte limit")
            }
        }
    }
}

impl std::error::Error for ProtocolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for ProtocolError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// Errors encountered while managing or communicating with extension workers.
#[derive(Debug)]
pub enum WorkerError {
    /// A worker process could not be started.
    Spawn { program: OsString, source: io::Error },
    /// A spawned process did not expose all required standard streams.
    MissingPipe { worker: usize, pipe: &'static str },
    /// A thread responsible for one worker stream could not be started.
    ThreadSpawn { worker: usize, role: &'static str, source: io::Error },
    /// A temporary host thread coordinating a pool-wide request panicked.
    CoordinatorPanic { worker: usize },
    /// A frame could not be encoded or decoded.
    Protocol { worker: usize, source: ProtocolError },
    /// A request exceeded its configured deadline.
    Timeout { worker: usize, request: u64, duration: Duration },
    /// A worker returned an error response.
    Remote { worker: usize, request: u64, payload: Vec<u8> },
    /// A worker sent a request but no host-side handler was installed.
    UnexpectedRequest { worker: usize, request: u64 },
    /// A worker connection stopped before completing the request.
    Disconnected { worker: usize, message: String },
    /// No live worker was available.
    Unavailable,
    /// A failed worker could not be replaced.
    Restart { worker: usize, source: Box<Self> },
}

impl WorkerError {
    #[must_use]
    pub(crate) fn protocol(worker: usize, source: ProtocolError) -> Self {
        Self::Protocol { worker, source }
    }
}

impl std::fmt::Display for WorkerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spawn { program, source } => {
                write!(formatter, "failed to start extension worker {}: {source}", program.to_string_lossy())
            }
            Self::MissingPipe { worker, pipe } => {
                write!(formatter, "extension worker {worker} did not expose its {pipe} pipe")
            }
            Self::ThreadSpawn { worker, role, source } => {
                write!(formatter, "failed to start extension worker {worker} {role} thread: {source}")
            }
            Self::CoordinatorPanic { worker } => {
                write!(formatter, "extension worker {worker} request coordinator panicked")
            }
            Self::Protocol { worker, source } => {
                write!(formatter, "extension worker {worker} violated the protocol: {source}")
            }
            Self::Timeout { worker, request, duration } => {
                write!(
                    formatter,
                    "extension worker {worker} did not answer request {request} within {} ms",
                    duration.as_millis()
                )
            }
            Self::Remote { worker, request, payload } => write!(
                formatter,
                "extension worker {worker} rejected request {request}: {}",
                String::from_utf8_lossy(payload)
            ),
            Self::UnexpectedRequest { worker, request } => {
                write!(formatter, "extension worker {worker} sent unexpected nested request {request}")
            }
            Self::Disconnected { worker, message } => {
                write!(formatter, "extension worker {worker} disconnected: {message}")
            }
            Self::Unavailable => formatter.write_str("no extension worker is available"),
            Self::Restart { worker, source } => {
                write!(formatter, "failed to restart extension worker {worker}: {source}")
            }
        }
    }
}

impl std::error::Error for WorkerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spawn { source, .. } => Some(source),
            Self::ThreadSpawn { source, .. } => Some(source),
            Self::Protocol { source, .. } => Some(source),
            Self::Restart { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// A cloneable failure sent from a worker's reader thread to pending requests.
#[derive(Debug, Clone)]
pub(crate) struct WorkerFailure {
    pub message: String,
}

impl WorkerFailure {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}
