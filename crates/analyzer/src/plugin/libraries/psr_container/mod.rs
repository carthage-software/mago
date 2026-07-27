//! PSR-11 Container providers.

mod get;

pub use get::ContainerGetProvider;

use crate::plugin::Plugin;
use crate::plugin::PluginMeta;
use crate::plugin::PluginRegistry;
use crate::plugin::PluginSettings;

/// Plugin providing type inference for psr/container interface.
pub struct PsrContainerPlugin;

static META: PluginMeta = PluginMeta::new(
    "psr-container",
    "PSR-11 Container",
    "Type providers for PSR-11 container interface",
    &["psr-11"],
    false,
);

impl Plugin for PsrContainerPlugin {
    fn meta(&self) -> &'static PluginMeta {
        &META
    }

    fn register(&self, registry: &mut PluginRegistry, _settings: &PluginSettings) {
        registry.register_method_provider(ContainerGetProvider);
    }
}
