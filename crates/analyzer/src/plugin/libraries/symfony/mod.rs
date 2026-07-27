//! Symfony providers.
//!
//! STATUS: WIP scaffold — not yet wired into `libraries/mod.rs`.
//! See `PLUGINS-RFC.md` at the repository root.
//!
//! Scope note, measured before designing: the shipped `psr-container` plugin
//! already resolves `get(X::class)` for any PSR-11 container — which covers
//! the dominant Symfony test pattern (`static::getContainer()->get(X::class)`).
//! What remains, and what this plugin adds:
//!
//! 1. String-id lookups: `get('some.service.id')` resolved through the
//!    compiled container XML (path given in configuration), mapping service
//!    ids to classes — the same source phpstan-symfony uses.
//! 2. `getParameter('...')` typed from the same XML.
//! 3. (later) `#[Autowire]`-style constructs where types are config-driven.

mod container_id;

pub use container_id::ContainerIdProvider;

use crate::plugin::Plugin;
use crate::plugin::PluginMeta;
use crate::plugin::PluginRegistry;
use crate::plugin::PluginSettings;

/// Plugin providing container-aware type inference for Symfony applications.
pub struct SymfonyPlugin;

static META: PluginMeta = PluginMeta::new(
    "symfony",
    "Symfony",
    "Container XML-driven type providers for Symfony applications",
    &["symfony-container"],
    false,
);

impl Plugin for SymfonyPlugin {
    fn meta(&self) -> &'static PluginMeta {
        &META
    }

    fn register(&self, registry: &mut PluginRegistry, _settings: &PluginSettings) {
        let _ = registry; // TODO: registry.register_method_provider(ContainerIdProvider::from_config(...));
    }
}
