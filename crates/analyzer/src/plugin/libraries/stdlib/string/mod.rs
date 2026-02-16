//! PHP string function providers.

mod sprintf;
mod strlen;

pub use sprintf::SprintfProvider;
pub use sprintf::resolve_sprintf;
pub use strlen::StrlenProvider;
