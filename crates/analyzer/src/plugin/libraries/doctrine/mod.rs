//! Doctrine ORM providers.
//!
//! See `PLUGINS-RFC.md` at the repository root for the full design and the
//! acceptance fixtures measured against a production Symfony/Doctrine codebase.

mod find_by;

pub use find_by::FindByFieldValidator;

use crate::plugin::Plugin;
use crate::plugin::PluginMeta;
use crate::plugin::PluginRegistry;
use crate::plugin::PluginSettings;

/// Plugin providing argument validation for Doctrine ORM.
///
/// Covers `findBy` / `findOneBy` / `count` criteria keys, validated against
/// the entity's persisted fields as declared through `#[ORM\Column]`,
/// `#[ORM\Id]` and association attributes. An unknown field is reported as
/// `doctrine-unknown-field` instead of being silently accepted.
pub struct DoctrinePlugin;

static META: PluginMeta = PluginMeta::new(
    "doctrine",
    "Doctrine ORM",
    "Criteria validation for Doctrine ORM repositories",
    &["doctrine-orm"],
    false,
);

impl Plugin for DoctrinePlugin {
    fn meta(&self) -> &'static PluginMeta {
        &META
    }

    fn register(&self, registry: &mut PluginRegistry, _settings: &PluginSettings) {
        registry.register_method_provider(FindByFieldValidator);
    }
}
