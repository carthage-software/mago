//! Plugin trait and metadata for analyzer plugins.

use crate::plugin::PluginRegistry;

/// Metadata describing an analyzer plugin.
#[derive(Debug, Clone)]
pub struct PluginMeta {
    /// Canonical plugin identifier (e.g., "stdlib").
    pub id: &'static str,
    /// Human-readable name (e.g., "PHP Standard Library").
    pub name: &'static str,
    /// Description of what the plugin provides.
    pub description: &'static str,
    /// Alternative names that resolve to this plugin.
    pub aliases: &'static [&'static str],
    /// Whether this plugin is enabled by default.
    pub default_enabled: bool,
}

impl PluginMeta {
    /// Creates a new plugin metadata.
    #[must_use]
    pub const fn new(
        id: &'static str,
        name: &'static str,
        description: &'static str,
        aliases: &'static [&'static str],
        default_enabled: bool,
    ) -> Self {
        Self { id, name, description, aliases, default_enabled }
    }

    /// Checks if the given name matches this plugin (either id or alias).
    #[must_use]
    pub fn matches(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        if self.id.to_lowercase() == name_lower {
            return true;
        }
        self.aliases.iter().any(|alias| alias.to_lowercase() == name_lower)
    }
}

/// Trait for analyzer plugins that provide type inference and analysis hooks.
pub trait Plugin: Send + Sync {
    /// Returns the metadata for this plugin.
    fn meta(&self) -> &'static PluginMeta;

    /// Registers all providers and hooks from this plugin into the registry.
    fn register(&self, registry: &mut PluginRegistry);
}
