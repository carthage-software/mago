//! Flow-PHP providers.

mod type_structure;

pub use type_structure::TypeStructureProvider;

use crate::plugin::Plugin;
use crate::plugin::PluginMeta;
use crate::plugin::PluginRegistry;

/// Plugin providing type inference for flow-php/etl package.
pub struct FlowPhpPlugin;

static META: PluginMeta = PluginMeta::new(
    "flow-php",
    "Flow-PHP",
    "Type providers for flow-php/etl package",
    &["flow", "flow-etl"],
    false, // not default enabled
);

impl Plugin for FlowPhpPlugin {
    fn meta(&self) -> &'static PluginMeta {
        &META
    }

    fn register(&self, registry: &mut PluginRegistry) {
        registry.register_function_provider(TypeStructureProvider);
    }
}
