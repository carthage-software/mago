//! Infrastructure for running external Mago extensions.
//!
//! This crate deliberately knows nothing about linter rules or analyzer
//! providers. It owns the process lifecycle, worker pool, multiplexed request
//! routing, and the stable outer frame used by all extension capabilities.

pub mod command;
pub mod error;
pub mod payload;
pub mod pool;
pub mod protocol;
pub mod worker;

mod reduction;

pub use command::WorkerCommand;
pub use error::PayloadError;
pub use error::ProtocolError;
pub use error::WorkerError;
pub use payload::PayloadReader;
pub use payload::PayloadWriter;
pub use pool::WorkerPool;
pub use pool::WorkerPoolOptions;
pub use protocol::FRAME_HEADER_LENGTH;
pub use protocol::FRAME_MAGIC;
pub use protocol::Frame;
pub use protocol::FrameFlags;
pub use protocol::FrameKind;
pub use protocol::PROTOCOL_VERSION;
pub use protocol::ProtocolVersion;
pub use worker::WorkerRequestHandler;
