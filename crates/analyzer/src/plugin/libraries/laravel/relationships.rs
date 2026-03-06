//! Relationship property type resolution for Laravel Eloquent models.
//!
//! When a model has a method like `posts()` returning `HasMany<Post>`, accessing
//! `$user->posts` should resolve to `\Illuminate\Database\Eloquent\Collection<Post>`.
//!
//! This module provides functions to:
//! - Detect relationship methods on a model class
//! - Extract the related type from the method's return type
//! - Build the correct property type (collection, singular model, or MorphTo)
//! - Synthesize `*_count` property types as `int`
//!
//! Derived from phpantom_lsp's `LaravelModelProvider`:
//! `classify_relationship()`, `extract_related_type()`, `build_property_type()`,
//! `infer_relationship_from_body()`.

use mago_atom::atom;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::get_int;
use mago_codex::ttype::union::TUnion;

use super::utils::ELOQUENT_COLLECTION;
use super::utils::ELOQUENT_MODEL;
use super::utils::RelationshipKind;
use super::utils::classify_relationship_fqn;
use super::utils::snake_to_camel;

/// Try to resolve the type for a virtual property access on a model.
///
/// Given a property name like `posts` on a model class, looks for:
/// 1. A relationship method with the same name (e.g. `posts()` returning `HasMany<Post>`)
/// 2. A `*_count` property for a relationship (e.g. `posts_count`)
///
/// Returns `Some(TUnion)` with the resolved type, or `None` if no
/// relationship matches.
pub fn resolve_relationship_property_type(
    property_name: &str,
    class_metadata: &ClassLikeMetadata,
    codebase: &CodebaseMetadata,
) -> Option<TUnion> {
    // First check if the property name directly maps to a relationship method.
    let camel_name = snake_to_camel(property_name);
    let method_name = if property_name.contains('_') { &camel_name } else { property_name };

    if let Some(method_meta) = codebase.get_method(&class_metadata.name, method_name)
        && let Some(ty) = resolve_relationship_type_from_method(method_meta, class_metadata, codebase)
    {
        return Some(ty);
    }

    // Also check the property name directly as a method name (relationship methods
    // are typically camelCase, but the property might already be the method name).
    if method_name != property_name
        && let Some(method_meta) = codebase.get_method(&class_metadata.name, property_name)
        && let Some(ty) = resolve_relationship_type_from_method(method_meta, class_metadata, codebase)
    {
        return Some(ty);
    }

    None
}

/// Try to resolve a `*_count` property (e.g. `posts_count` → `int`).
///
/// Returns `Some(get_int())` if the property name ends with `_count` and
/// the base name maps to a relationship method.
pub fn resolve_count_property_type(
    property_name: &str,
    class_metadata: &ClassLikeMetadata,
    codebase: &CodebaseMetadata,
) -> Option<TUnion> {
    let base = property_name.strip_suffix("_count")?;
    if base.is_empty() {
        return None;
    }

    let method_name = snake_to_camel(base);
    let method_meta = codebase.get_method(&class_metadata.name, &method_name)?;

    // Verify it's actually a relationship method.
    let return_type = get_return_type_union(method_meta)?;
    let kind = classify_return_type_as_relationship(return_type)?;
    let _ = kind; // We just need to verify it's a relationship.

    Some(get_int())
}

/// Resolve the property type from a relationship method's return type.
///
/// Maps:
/// - Singular relationships (HasOne, BelongsTo, MorphOne, HasOneThrough) → `RelatedModel|null`
/// - Collection relationships (HasMany, BelongsToMany, etc.) → `Collection<RelatedModel>`
/// - MorphTo → `Model|null`
fn resolve_relationship_type_from_method(
    method_meta: &FunctionLikeMetadata,
    _owner_class: &ClassLikeMetadata,
    codebase: &CodebaseMetadata,
) -> Option<TUnion> {
    let return_type = get_return_type_union(method_meta)?;
    let kind = classify_return_type_as_relationship(return_type)?;
    let related_type = extract_related_type_from_union(return_type);

    build_property_type(kind, related_type.as_deref(), codebase)
}

/// Get the return type TUnion from a method's metadata.
///
/// Prefers the docblock/annotated return type over the declaration type.
fn get_return_type_union(method_meta: &FunctionLikeMetadata) -> Option<&TUnion> {
    method_meta.return_type_metadata.as_ref().map(|tm| &tm.type_union)
}

/// Classify a method's return type union as a relationship kind.
///
/// Looks through the atomic types in the union for a named object that
/// is a known Eloquent relationship class.
fn classify_return_type_as_relationship(return_type: &TUnion) -> Option<RelationshipKind> {
    for atomic in return_type.types.iter() {
        if let TAtomic::Object(TObject::Named(named)) = atomic {
            let name_str: &str = named.name.as_ref();
            if let Some(kind) = classify_relationship_fqn(name_str) {
                return Some(kind);
            }
        }
    }
    None
}

/// Extract the related model type from a relationship return type's
/// generic parameters.
///
/// For `HasMany<Post>`, extracts `Post`.
/// For `HasMany<\App\Models\Post, $this>`, extracts `App\Models\Post`.
fn extract_related_type_from_union(return_type: &TUnion) -> Option<String> {
    for atomic in return_type.types.iter() {
        if let TAtomic::Object(TObject::Named(named)) = atomic {
            let name_str: &str = named.name.as_ref();
            if classify_relationship_fqn(name_str).is_some() {
                if let Some(related_atomic) = named
                    .type_parameters
                    .as_ref()
                    .and_then(|tp| tp.first())
                    .and_then(|first_param| first_param.types.first())
                {
                    return Some(get_fqn_from_atomic(related_atomic));
                }
                // No generic parameters — return None to indicate unknown related type.
                return None;
            }
        }
    }
    None
}

/// Extract a fully-qualified name string from an atomic type.
fn get_fqn_from_atomic(atomic: &TAtomic) -> String {
    match atomic {
        TAtomic::Object(TObject::Named(named)) => {
            let name: &str = named.name.as_ref();
            name.to_string()
        }
        TAtomic::Object(TObject::Enum(e)) => {
            let name: &str = e.name.as_ref();
            name.to_string()
        }
        _ => {
            // For things like `static`, `$this`, generic params, etc.
            let id = atomic.get_id();
            let s: &str = id.as_ref();
            s.to_string()
        }
    }
}

/// Build the TUnion property type for a relationship.
///
/// - Singular → `RelatedModel|null` (nullable because relationship may not be loaded;
///   this is an intentional divergence from phpantom which omits null)
/// - Collection → `Collection<RelatedModel>` (using custom collection if available)
/// - MorphTo → `Model|null`
fn build_property_type(
    kind: RelationshipKind,
    related_type: Option<&str>,
    codebase: &CodebaseMetadata,
) -> Option<TUnion> {
    match kind {
        RelationshipKind::Singular => {
            let related = related_type?;
            // Build nullable type: RelatedModel|null
            let obj = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom(related)))));
            Some(obj.as_nullable())
        }
        RelationshipKind::Collection => {
            let related = related_type.unwrap_or(ELOQUENT_MODEL);

            // Check if the related model has a custom collection via #[CollectedBy].
            let collection_class =
                get_custom_collection_class(related, codebase).unwrap_or_else(|| ELOQUENT_COLLECTION.to_string());

            let related_union = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom(related)))));

            let collection_obj =
                TNamedObject::new_with_type_parameters(atom(&collection_class), Some(vec![related_union]));

            Some(TUnion::from_atomic(TAtomic::Object(TObject::Named(collection_obj))))
        }
        RelationshipKind::MorphTo => {
            // MorphTo resolves to the generic Model base class, nullable.
            let obj = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom(ELOQUENT_MODEL)))));
            Some(obj.as_nullable())
        }
    }
}

/// Look up the custom collection class for a model via `#[CollectedBy]` attribute.
///
/// Returns the FQN of the custom collection class if the model declares
/// `#[CollectedBy(CustomCollection::class)]`, or `None` to use the default
/// `Illuminate\Database\Eloquent\Collection`.
///
/// **Note:** Currently `AttributeMetadata` does not store arguments, so
/// custom collection support is not yet functional.  This function is
/// a placeholder that always returns `None` until the metadata is
/// extended to include attribute arguments.
fn get_custom_collection_class(_model_fqn: &str, _codebase: &CodebaseMetadata) -> Option<String> {
    // TODO: Implement once AttributeMetadata stores argument values.
    // See migrate-laravel.md § "Note on custom collection classes".
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_singular_property_type() {
        let codebase = CodebaseMetadata::default();
        let ty = build_property_type(RelationshipKind::Singular, Some("App\\Models\\Post"), &codebase);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        // Should be nullable (Post|null).
        assert!(ty.is_nullable());
    }

    #[test]
    fn build_singular_without_related_returns_none() {
        let codebase = CodebaseMetadata::default();
        let ty = build_property_type(RelationshipKind::Singular, None, &codebase);
        assert!(ty.is_none());
    }

    #[test]
    fn build_collection_property_type() {
        let codebase = CodebaseMetadata::default();
        let ty = build_property_type(RelationshipKind::Collection, Some("App\\Models\\Post"), &codebase);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        // Should be a named object (Collection<Post>).
        assert!(!ty.is_nullable());
    }

    #[test]
    fn build_collection_without_related_uses_model_fallback() {
        let codebase = CodebaseMetadata::default();
        let ty = build_property_type(RelationshipKind::Collection, None, &codebase);
        assert!(ty.is_some());
    }

    #[test]
    fn build_morph_to_type() {
        let codebase = CodebaseMetadata::default();
        let ty = build_property_type(RelationshipKind::MorphTo, None, &codebase);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        // MorphTo is always nullable Model|null.
        assert!(ty.is_nullable());
    }

    #[test]
    fn build_morph_to_ignores_related_type() {
        let codebase = CodebaseMetadata::default();
        let ty = build_property_type(RelationshipKind::MorphTo, Some("App\\Models\\Specific"), &codebase);
        assert!(ty.is_some());
        let ty = ty.unwrap();
        assert!(ty.is_nullable());
    }

    #[test]
    fn related_type_without_leading_backslash() {
        let codebase = CodebaseMetadata::default();
        let ty = build_property_type(RelationshipKind::Singular, Some("App\\Models\\Post"), &codebase);
        assert!(ty.is_some());
    }

    #[test]
    fn classify_return_type_has_many() {
        let union = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
            atom("Illuminate\\Database\\Eloquent\\Relations\\HasMany"),
            Some(vec![TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom(
                "App\\Models\\Post",
            )))))]),
        ))));
        let kind = classify_return_type_as_relationship(&union);
        assert_eq!(kind, Some(RelationshipKind::Collection));
    }

    #[test]
    fn classify_return_type_has_one() {
        let union = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom(
            "Illuminate\\Database\\Eloquent\\Relations\\HasOne",
        )))));
        let kind = classify_return_type_as_relationship(&union);
        assert_eq!(kind, Some(RelationshipKind::Singular));
    }

    #[test]
    fn classify_return_type_morph_to() {
        let union = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom(
            "Illuminate\\Database\\Eloquent\\Relations\\MorphTo",
        )))));
        let kind = classify_return_type_as_relationship(&union);
        assert_eq!(kind, Some(RelationshipKind::MorphTo));
    }

    #[test]
    fn classify_non_relationship_returns_none() {
        let union = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom("App\\Models\\Post")))));
        let kind = classify_return_type_as_relationship(&union);
        assert!(kind.is_none());
    }

    #[test]
    fn extract_related_type_from_has_many() {
        let union = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
            atom("Illuminate\\Database\\Eloquent\\Relations\\HasMany"),
            Some(vec![TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom(
                "App\\Models\\Post",
            )))))]),
        ))));
        let related = extract_related_type_from_union(&union);
        assert_eq!(related, Some("App\\Models\\Post".to_string()));
    }

    #[test]
    fn extract_related_type_no_generics() {
        let union = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom(
            "Illuminate\\Database\\Eloquent\\Relations\\HasMany",
        )))));
        let related = extract_related_type_from_union(&union);
        assert!(related.is_none());
    }
}
