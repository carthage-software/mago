//! Factory return type support for Laravel Eloquent factories.
//!
//! This module implements Phase 5 of the Laravel migration:
//!
//! When a class extends `Illuminate\Database\Eloquent\Factories\Factory`
//! and does not already have `@extends Factory<Model>` generics, the
//! `create()` and `make()` methods should return the model type derived
//! from the naming convention (e.g. `UserFactory` → `User`).
//!
//! This is implemented as a `MethodReturnTypeProvider` targeting `create`
//! and `make` on any class.  The provider checks if the calling class
//! is a Factory subclass without explicit generics, derives the model
//! FQN from the naming convention, verifies the model exists in the
//! codebase, and returns its type.
//!
//! Derived from phpantom_lsp's `LaravelFactoryProvider` and
//! `build_factory_model_methods()`.

use mago_atom::atom;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::union::TUnion;

use crate::plugin::context::InvocationInfo;
use crate::plugin::context::ProviderContext;
use crate::plugin::provider::Provider;
use crate::plugin::provider::ProviderMeta;
use crate::plugin::provider::method::MethodReturnTypeProvider;
use crate::plugin::provider::method::MethodTarget;

use super::utils::ELOQUENT_FACTORY;
use super::utils::factory_to_model_fqn;
use super::utils::is_eloquent_factory_parent;

// ────────────────────────────────────────────────────────────────────────────────
// Constants
// ────────────────────────────────────────────────────────────────────────────────

/// The short name `Factory` used for checking `@extends` generics.
const FACTORY_SHORT_NAME: &str = "Factory";

// ────────────────────────────────────────────────────────────────────────────────
// MethodReturnTypeProvider — Factory create/make return types
// ────────────────────────────────────────────────────────────────────────────────

static META: ProviderMeta = ProviderMeta::new(
    "laravel-factory-return-type",
    "Laravel Factory Return Type",
    "Returns the model type for create() and make() on Factory subclasses",
);

/// Targets `create` and `make` on any class (we filter to Factory subclasses
/// in `get_return_type`).
static TARGETS: [MethodTarget; 2] = [MethodTarget::any_class("create"), MethodTarget::any_class("make")];

/// Method return type provider that makes `UserFactory::create()` and
/// `UserFactory::make()` return `User` (the model type derived from
/// naming convention) when the factory doesn't have explicit
/// `@extends Factory<Model>` generics.
pub struct FactoryReturnTypeProvider;

impl Provider for FactoryReturnTypeProvider {
    fn meta() -> &'static ProviderMeta {
        &META
    }
}

impl MethodReturnTypeProvider for FactoryReturnTypeProvider {
    fn targets() -> &'static [MethodTarget]
    where
        Self: Sized,
    {
        &TARGETS
    }

    fn get_return_type(
        &self,
        context: &ProviderContext<'_, '_, '_>,
        class_name: &str,
        _method_name: &str,
        invocation: &InvocationInfo<'_, '_, '_>,
    ) -> Option<TUnion> {
        // The `class_name` parameter is the *declaring* class (e.g. `Factory`),
        // but we need the *calling* class (e.g. `UserFactory`) to derive the
        // model from the naming convention.  Use `calling_class_name()` which
        // reads the method context's class_like_metadata, falling back to the
        // declaring class name.
        let effective_class: String = invocation
            .calling_class_name()
            .map(|atom| {
                let s: &str = atom.as_ref();
                s.to_string()
            })
            .unwrap_or_else(|| class_name.to_string());
        let effective_class_name: &str = &effective_class;

        // Skip the Factory base class itself.
        if is_factory_base_class(effective_class_name) {
            return None;
        }

        // Check if this class is a Factory subclass.
        let class_metadata = context.codebase().get_class_like(effective_class_name)?;
        if !is_eloquent_factory_parent(&class_metadata.all_parent_classes) {
            return None;
        }

        // Check if the factory already has `@extends Factory<Model>` generics.
        // If so, the type system can resolve TModel on its own — skip.
        if has_factory_extends_generic(class_metadata) {
            return None;
        }

        // Derive the model FQN from the factory's naming convention.
        let model_fqn = factory_to_model_fqn(effective_class_name)?;

        // Verify the model class actually exists in the codebase.
        context.codebase().get_class_like(&model_fqn)?;

        // Return the model type.
        Some(TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom(&model_fqn))))))
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Helpers
// ────────────────────────────────────────────────────────────────────────────────

/// Check whether a class name is the Factory base class itself.
fn is_factory_base_class(class_name: &str) -> bool {
    class_name.eq_ignore_ascii_case(ELOQUENT_FACTORY)
}

/// Check whether a factory class already has `@extends Factory<Model>`
/// that would let the generics system resolve `TModel`.
///
/// Looks through `template_extended_offsets` for a parent named `Factory`
/// (or the full FQN) with non-empty type parameters.
fn has_factory_extends_generic(class_metadata: &ClassLikeMetadata) -> bool {
    for (parent_name, type_params) in class_metadata.template_extended_offsets.iter() {
        let name_str: &str = parent_name.as_ref();

        // Check by short name or full FQN.
        let is_factory = name_str.eq_ignore_ascii_case(ELOQUENT_FACTORY)
            || name_str.rsplit('\\').next().unwrap_or(name_str).eq_ignore_ascii_case(FACTORY_SHORT_NAME);

        if is_factory
            && let Some(first) = type_params.first()
            && !first.is_mixed()
            && !first.is_never()
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use mago_codex::metadata::class_like::ClassLikeMetadata;
    use mago_codex::metadata::flags::MetadataFlags;
    use mago_codex::ttype::get_mixed;
    use mago_codex::ttype::get_never;
    use mago_span::Span;

    // ── is_factory_base_class ───────────────────────────────────────

    #[test]
    fn factory_base_class_fqn() {
        assert!(is_factory_base_class(ELOQUENT_FACTORY));
    }

    #[test]
    fn factory_base_class_case_insensitive() {
        assert!(is_factory_base_class("illuminate\\database\\eloquent\\factories\\factory"));
    }

    #[test]
    fn non_factory_class() {
        assert!(!is_factory_base_class("App\\Models\\User"));
    }

    #[test]
    fn user_factory_is_not_base_class() {
        assert!(!is_factory_base_class("Database\\Factories\\UserFactory"));
    }

    // ── has_factory_extends_generic ─────────────────────────────────

    #[test]
    fn no_extends_generics() {
        let metadata = ClassLikeMetadata::new(
            atom("Database\\Factories\\UserFactory"),
            atom("Database\\Factories\\UserFactory"),
            Span::dummy(0, 0),
            None,
            MetadataFlags::empty(),
        );
        assert!(!has_factory_extends_generic(&metadata));
    }

    #[test]
    fn has_extends_generic_with_model_type() {
        let mut metadata = ClassLikeMetadata::new(
            atom("Database\\Factories\\UserFactory"),
            atom("Database\\Factories\\UserFactory"),
            Span::dummy(0, 0),
            None,
            MetadataFlags::empty(),
        );

        let model_type =
            TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom("App\\Models\\User")))));
        metadata.add_template_extended_offset(atom(ELOQUENT_FACTORY), vec![model_type]);

        assert!(has_factory_extends_generic(&metadata));
    }

    #[test]
    fn has_extends_generic_with_short_name() {
        let mut metadata = ClassLikeMetadata::new(
            atom("Database\\Factories\\UserFactory"),
            atom("Database\\Factories\\UserFactory"),
            Span::dummy(0, 0),
            None,
            MetadataFlags::empty(),
        );

        let model_type =
            TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom("App\\Models\\User")))));
        metadata.add_template_extended_offset(atom("Factory"), vec![model_type]);

        assert!(has_factory_extends_generic(&metadata));
    }

    #[test]
    fn extends_generic_with_mixed_is_not_useful() {
        let mut metadata = ClassLikeMetadata::new(
            atom("Database\\Factories\\UserFactory"),
            atom("Database\\Factories\\UserFactory"),
            Span::dummy(0, 0),
            None,
            MetadataFlags::empty(),
        );

        metadata.add_template_extended_offset(atom(ELOQUENT_FACTORY), vec![get_mixed()]);

        assert!(!has_factory_extends_generic(&metadata));
    }

    #[test]
    fn extends_generic_with_never_is_not_useful() {
        let mut metadata = ClassLikeMetadata::new(
            atom("Database\\Factories\\UserFactory"),
            atom("Database\\Factories\\UserFactory"),
            Span::dummy(0, 0),
            None,
            MetadataFlags::empty(),
        );

        metadata.add_template_extended_offset(atom(ELOQUENT_FACTORY), vec![get_never()]);

        assert!(!has_factory_extends_generic(&metadata));
    }

    #[test]
    fn empty_type_params_is_not_useful() {
        let mut metadata = ClassLikeMetadata::new(
            atom("Database\\Factories\\UserFactory"),
            atom("Database\\Factories\\UserFactory"),
            Span::dummy(0, 0),
            None,
            MetadataFlags::empty(),
        );

        metadata.add_template_extended_offset(atom(ELOQUENT_FACTORY), vec![]);

        assert!(!has_factory_extends_generic(&metadata));
    }

    // ── factory_to_model_fqn (integration with utils) ───────────────

    #[test]
    fn factory_to_model_standard() {
        let result = factory_to_model_fqn("Database\\Factories\\UserFactory");
        assert_eq!(result, Some("App\\Models\\User".to_string()));
    }

    #[test]
    fn factory_to_model_subdirectory() {
        let result = factory_to_model_fqn("Database\\Factories\\Admin\\UserFactory");
        assert_eq!(result, Some("App\\Models\\Admin\\User".to_string()));
    }

    #[test]
    fn factory_to_model_leading_backslash() {
        let result = factory_to_model_fqn("\\Database\\Factories\\UserFactory");
        assert_eq!(result, Some("App\\Models\\User".to_string()));
    }

    #[test]
    fn factory_to_model_no_factory_suffix() {
        let result = factory_to_model_fqn("Database\\Factories\\User");
        assert!(result.is_none());
    }

    #[test]
    fn factory_to_model_bare_factory() {
        // "Factory" alone → short name is "Factory", strip "Factory" → empty → None
        let result = factory_to_model_fqn("Factory");
        assert!(result.is_none());
    }
}
