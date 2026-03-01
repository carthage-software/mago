//! Laravel framework providers.
//!
//! This plugin provides analysis support for Laravel applications,
//! helping the analyzer understand Eloquent conventions that rely on
//! runtime behavior (`__call`, `__callStatic`, `__get`, `__set`).
//!
//! The functionality here is derived from PHPantom's Laravel
//! virtual member providers (`src/virtual_members/laravel.rs`).

pub mod issue_filter;
pub mod property_init;
pub mod utils;

use crate::plugin::Plugin;
use crate::plugin::PluginMeta;
use crate::plugin::PluginRegistry;

use issue_filter::LaravelIssueFilter;
use property_init::LaravelPropertyInit;

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

    fn register(&self, registry: &mut PluginRegistry) {
        // Phase 2: Mark Eloquent model/builder/factory properties as initialized
        registry.register_property_initialization_provider(LaravelPropertyInit);
        // Phase 1: Suppress false-positive diagnostics on Eloquent classes
        registry.register_issue_filter_hook(LaravelIssueFilter::new());
    }
}