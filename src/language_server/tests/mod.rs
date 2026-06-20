//! In-process LSP tests. Driven via [`tokio::io::duplex`] paired with
//! [`super::serve`] so each test runs the full backend without the
//! overhead of spawning a child `mago` binary.

mod client;
mod harness;

mod capabilities;
mod initialize;
mod lifecycle;
