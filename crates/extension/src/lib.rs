//! Infrastructure for running external Mago extensions.
//!
//! This crate deliberately knows nothing about linter rules or analyzer
//! providers. It owns the process lifecycle, worker pool, multiplexed request
//! routing, capability-neutral source snapshots, and the stable outer frame
//! used by all extension capabilities.

pub mod command;
pub mod error;
pub mod payload;
pub mod pool;
pub mod protocol;
pub mod source;
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
pub use protocol::Frame;
pub use worker::WorkerRequestHandler;
