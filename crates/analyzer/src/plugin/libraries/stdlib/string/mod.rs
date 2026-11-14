//! PHP string function providers.

mod sprintf;
mod strlen;
mod strval;

pub use sprintf::SprintfProvider;
pub use sprintf::resolve_sprintf;
pub use strlen::StrlenProvider;
pub use strval::StrvalProvider;
