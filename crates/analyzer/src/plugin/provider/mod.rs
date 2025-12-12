//! Provider traits for the analyzer plugin system.

pub mod assertion;
pub mod function;
pub mod method;
pub mod property;
pub mod throw;

pub use assertion::*;
pub use function::*;
pub use method::*;
pub use property::*;
pub use throw::*;

#[derive(Debug, Clone, Copy)]
pub struct ProviderMeta {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

impl ProviderMeta {
    pub const fn new(id: &'static str, name: &'static str, description: &'static str) -> Self {
        Self { id, name, description }
    }
}

pub trait Provider: Send + Sync {
    fn meta() -> &'static ProviderMeta
    where
        Self: Sized;
}
