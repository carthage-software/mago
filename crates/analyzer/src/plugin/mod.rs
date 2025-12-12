//! Plugin system for extending the analyzer with custom type inference and hooks.

pub mod context;
pub mod error;
pub mod hook;
pub mod libraries;
pub mod provider;
pub mod registry;

pub use context::*;
pub use error::*;
pub use hook::*;
pub use provider::*;
pub use registry::PluginRegistry;

pub fn create_registry() -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    libraries::register_library_providers(&mut registry);
    registry
}
