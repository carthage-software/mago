//! Laravel framework providers.
//!
//! This plugin provides analysis support for Laravel applications,
//! helping the analyzer understand Eloquent conventions that rely on
//! runtime behavior (`__call`, `__callStatic`, `__get`, `__set`).
//!
//! The functionality here is derived from PHPantom's Laravel
//! virtual member providers (`src/virtual_members/laravel.rs`).

pub mod accessors;
pub mod builder;
pub mod casts;
pub mod factory;
pub mod issue_filter;
pub mod model;
pub mod property_init;
pub mod relationships;
pub mod utils;

use crate::plugin::Plugin;
use crate::plugin::PluginMeta;
use crate::plugin::PluginRegistry;

use builder::BuilderForwardingHook;
use builder::BuilderReturnTypeProvider;
use builder::BuilderScopeHook;
use factory::FactoryReturnTypeProvider;
use issue_filter::LaravelIssueFilter;
use model::LaravelModelPropertyHook;
use property_init::LaravelPropertyInit;

/// Plugin providing analysis support for the Laravel framework.
pub struct LaravelPlugin;

static META: PluginMeta =
    PluginMeta::new("laravel", "Laravel", "Analysis support for the Laravel framework", &["illuminate"], false);

impl Plugin for LaravelPlugin {
    fn meta(&self) -> &'static PluginMeta {
        &META
    }

    fn register(&self, registry: &mut PluginRegistry) {
        // Phase 2: Mark Eloquent model/builder/factory properties as initialized
        registry.register_property_initialization_provider(LaravelPropertyInit);
        // Phase 1: Suppress false-positive diagnostics on Eloquent classes
        registry.register_issue_filter_hook(LaravelIssueFilter::new());
        // Phase 3: Provide precise types for Eloquent model virtual properties
        registry.register_expression_hook(LaravelModelPropertyHook);
        // Phase 4: Forward static method calls on Models to Builder
        registry.register_static_method_call_hook(BuilderForwardingHook);
        // Phase 4: Map Builder method return types for query chain resolution
        registry.register_method_provider(BuilderReturnTypeProvider);
        // Phase 5: Provide precise return types for Factory create/make methods
        registry.register_method_provider(FactoryReturnTypeProvider);
        // Phase 6: Resolve scope method calls on Builder instances
        registry.register_method_call_hook(BuilderScopeHook);
    }
}
