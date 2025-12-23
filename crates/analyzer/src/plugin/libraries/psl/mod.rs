//! PSL (PHP Standard Library) providers.

pub mod regex;
pub mod str;
pub mod type_;

use crate::plugin::Plugin;
use crate::plugin::PluginMeta;
use crate::plugin::PluginRegistry;

/// Plugin providing type inference for azjezz/psl package.
pub struct PslPlugin;

static META: PluginMeta = PluginMeta::new(
    "psl",
    "PSL (PHP Standard Library)",
    "Type providers for azjezz/psl package",
    &["php-standard-library", "azjezz-psl"],
    false,
);

impl Plugin for PslPlugin {
    fn meta(&self) -> &'static PluginMeta {
        &META
    }

    fn register(&self, registry: &mut PluginRegistry) {
        registry.register_function_provider(type_::ShapeProvider);
        registry.register_function_provider(type_::OptionalProvider);
        registry.register_function_provider(str::StrProvider);
        registry.register_function_provider(regex::CaptureGroupsProvider);
    }
}
