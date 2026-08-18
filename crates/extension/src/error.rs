use std::ffi::OsString;
use std::io;
use std::time::Duration;

use crate::protocol::ProtocolVersion;

/// Errors encountered while encoding or decoding a domain payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadError {
    /// A collection or byte string cannot be represented by the wire `u32` length.
    LengthOverflow { length: usize },
    /// Advancing the reader would overflow its byte offset.
    OffsetOverflow { field: &'static str },
    /// The payload ended before a field was fully decoded.
    UnexpectedEnd { field: &'static str, needed: usize, remaining: usize },
    /// A byte string expected to contain text is not valid UTF-8.
    InvalidUtf8 { field: &'static str },
    /// A boolean was encoded with a value other than zero or one.
    InvalidBoolean { field: &'static str, value: u8 },
    /// A decoded collection count exceeds its domain-specific limit.
    CountExceeded { field: &'static str, count: usize, maximum: usize },
    /// A message contained bytes after its last expected field.
    TrailingBytes { remaining: usize },
}

impl std::fmt::Display for PayloadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LengthOverflow { length } => {
                write!(formatter, "payload length {length} exceeds u32::MAX")
            }
            Self::OffsetOverflow { field } => write!(formatter, "{field} length overflowed the payload offset"),
            Self::UnexpectedEnd { field, needed, remaining } => {
                write!(formatter, "payload ended while reading {field}: needed {needed} bytes, found {remaining}")
            }
            Self::InvalidUtf8 { field } => write!(formatter, "{field} is not valid UTF-8"),
            Self::InvalidBoolean { field, value } => write!(formatter, "{field} has invalid boolean value {value}"),
            Self::CountExceeded { field, count, maximum } => {
                write!(formatter, "{field} count {count} exceeds limit {maximum}")
            }
            Self::TrailingBytes { remaining } => write!(formatter, "payload contains {remaining} trailing bytes"),
        }
    }
}

impl std::error::Error for PayloadError {}

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
    /// A worker explicitly cancelled an in-flight request.
    Cancelled { worker: usize, request: u64 },
    /// A worker returned an error response.
    Remote { worker: usize, request: u64, payload: Vec<u8> },
    /// A worker sent a request but no host-side handler was installed.
    UnexpectedRequest { worker: usize, request: u64 },
    /// A worker connection stopped before completing the request.
    Disconnected { worker: usize, message: String },
    /// A lazily started worker advertised different initialization state.
    InconsistentInitialization { worker: usize },
    /// Terminal state reduction could not complete for a worker.
    Reduction { worker: usize, message: String },
    /// No live worker was available.
    Unavailable,
    /// A failed worker could not be replaced.
    Restart { worker: usize, source: Box<Self> },
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
            Self::Cancelled { worker, request } => {
                write!(formatter, "extension worker {worker} cancelled request {request}")
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
            Self::InconsistentInitialization { worker } => {
                write!(formatter, "extension worker {worker} returned inconsistent initialization state")
            }
            Self::Reduction { worker, message } => {
                write!(formatter, "extension worker {worker} reduction failed: {message}")
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
