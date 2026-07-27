//! Doctrine ORM providers.
//!
//! STATUS: WIP scaffold — not yet wired into `libraries/mod.rs`.
//! See `PLUGINS-RFC.md` at the repository root for the full design and the
//! acceptance fixtures measured against a production Symfony/Doctrine codebase.

mod find_by;

pub use find_by::FindByFieldValidator;

use crate::plugin::Plugin;
use crate::plugin::PluginMeta;
use crate::plugin::PluginRegistry;

/// Plugin providing type inference and argument validation for Doctrine ORM.
///
/// What it covers, in order of measured value:
/// 1. `findBy` / `findOneBy` / `count` criteria keys validated against the
///    entity's persisted fields, read from `#[ORM\Column]`, `#[ORM\Id]` and
///    association attributes already parsed by the codex. An unknown field is
///    reported instead of being silently accepted.
/// 2. (later) precise return types for `EntityRepository` methods where the
///    docblock generics of doctrine/orm are not enough.
pub struct DoctrinePlugin;

static META: PluginMeta = PluginMeta::new(
    "doctrine",
    "Doctrine ORM",
    "Criteria validation and type providers for Doctrine ORM entities",
    &["doctrine-orm"],
    false,
);

impl Plugin for DoctrinePlugin {
    fn meta(&self) -> &'static PluginMeta {
        &META
    }

    fn register(&self, registry: &mut PluginRegistry) {
        let _ = registry; // TODO: registry.register_method_hook(FindByFieldValidator);
    }
}
