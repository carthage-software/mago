//! Laravel framework providers.
//!
//! This plugin provides analysis support for Laravel applications,
//! helping the analyzer understand Eloquent conventions that rely on
//! runtime behavior (`__call`, `__callStatic`, `__get`, `__set`).
//!
//! The functionality here is derived from PHPantom's Laravel
//! virtual member providers (`src/virtual_members/laravel.rs`).

pub mod utils;

use crate::plugin::Plugin;
use crate::plugin::PluginMeta;
use crate::plugin::PluginRegistry;

/// Plugin providing analysis support for the Laravel framework.
pub struct LaravelPlugin;

static META: PluginMeta = PluginMeta::new(
    "laravel",
    "Laravel",
    "Analysis support for the Laravel framework",
    &["illuminate"],
    false,
);

impl Plugin for LaravelPlugin {
    fn meta(&self) -> &'static PluginMeta {
        &META
    }

    fn register(&self, _registry: &mut PluginRegistry) {
        // TODO: Register providers derived from PHPantom's
        // LaravelModelProvider and LaravelFactoryProvider.
    }
}
