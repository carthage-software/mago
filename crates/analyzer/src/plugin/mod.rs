//! Plugin system for extending the analyzer with custom type inference and hooks.
#![allow(clippy::module_inception)]

pub mod context;
pub mod error;
pub mod hook;
pub mod libraries;
pub mod plugin;
pub mod provider;
pub mod registry;

pub use context::*;
pub use error::*;
pub use hook::*;
pub use plugin::*;
pub use provider::*;
pub use registry::PluginRegistry;

/// Returns a list of all available plugins with their metadata.
#[must_use]
pub fn available_plugins() -> Vec<&'static PluginMeta> {
    libraries::ALL_PLUGINS.iter().map(|p| p.meta()).collect()
}

/// Resolves a plugin name or alias to its canonical id.
///
/// Returns `None` if no plugin matches the given name.
#[must_use]
pub fn resolve_plugin_name(name: &str) -> Option<&'static str> {
    for plugin in libraries::ALL_PLUGINS.iter() {
        if plugin.meta().matches(name) {
            return Some(plugin.meta().id);
        }
    }
    None
}

/// Creates a plugin registry with the specified plugins enabled.
///
/// # Arguments
///
/// * `enabled_plugins` - List of plugin names/aliases to enable
/// * `disable_defaults` - If true, default plugins are not automatically enabled
///
/// # Returns
///
/// A new `PluginRegistry` with only the specified plugins registered.
#[must_use]
pub fn create_registry_with_plugins(enabled_plugins: &[String], disable_defaults: bool) -> PluginRegistry {
    let mut registry = PluginRegistry::new();

    for plugin in libraries::ALL_PLUGINS.iter() {
        let meta = plugin.meta();

        // Check if explicitly enabled
        let explicitly_enabled = enabled_plugins.iter().any(|name| meta.matches(name));

        // Check if should be enabled by default
        let default_enabled = !disable_defaults && meta.default_enabled;

        if explicitly_enabled || default_enabled {
            plugin.register(&mut registry);
        }
    }

    registry
}

/// Creates a plugin registry with all library providers enabled.
///
/// This is the legacy behavior for backwards compatibility.
#[must_use]
pub fn create_registry() -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    for plugin in libraries::ALL_PLUGINS.iter() {
        plugin.register(&mut registry);
    }

    registry
}
