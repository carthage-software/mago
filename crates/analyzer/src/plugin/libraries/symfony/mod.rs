//! Symfony providers.
//!
//! See `PLUGINS-RFC.md` at the repository root for the full design and the
//! acceptance probes measured against a production Symfony codebase.
//!
//! Scope note, measured before designing: the shipped `psr-container` plugin
//! already resolves `get(X::class)` for any PSR-11 container — which covers
//! the dominant Symfony test pattern (`static::getContainer()->get(X::class)`).
//! This plugin adds what `::class` cannot express: literal **string** service
//! ids, resolved through the compiled container XML dump — the same source
//! phpstan-symfony uses. `getParameter('...')` typing from the same dump is a
//! natural follow-up, kept out of this iteration to keep the provider small.

mod container_id;
pub mod container_xml;

pub use container_id::ContainerIdProvider;
pub use container_xml::ServiceMap;

use crate::plugin::Plugin;
use crate::plugin::PluginMeta;
use crate::plugin::PluginRegistry;
use crate::plugin::PluginSettings;

/// Plugin providing container-aware type inference for Symfony applications.
///
/// Requires the `symfony_container_xml_path` plugin setting to point at a
/// compiled container dump (for an application under test, typically
/// `var/cache/test/App_KernelTestDebugContainer.xml`). Without it — or when
/// the dump cannot be read — the plugin registers nothing.
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

    fn register(&self, registry: &mut PluginRegistry, settings: &PluginSettings) {
        let Some(path) = &settings.symfony_container_xml_path else {
            tracing::debug!(
                "The `symfony` plugin is enabled without `symfony_container_xml_path`; nothing to register."
            );

            return;
        };

        let Some(services) = ServiceMap::load(path) else {
            // `ServiceMap::load` already warned about the unreadable dump.
            return;
        };

        registry.register_method_provider(ContainerIdProvider::new(services));
    }
}
