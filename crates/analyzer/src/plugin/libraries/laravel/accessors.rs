//! Accessor property type resolution for Laravel Eloquent models.
//!
//! Laravel supports two accessor patterns:
//!
//! - **Legacy accessors** (all versions): `getFullNameAttribute(): string` → `$full_name: string`
//! - **Modern accessors** (Laravel 9+): `fullName(): Attribute<string>` → `$full_name: string`
//!
//! This module provides functions to detect accessor methods on a model class
//! and resolve the corresponding virtual property type.
//!
//! Derived from phpantom_lsp's `LaravelModelProvider`:
//! `is_legacy_accessor()`, `is_modern_accessor()`,
//! `legacy_accessor_property_name()`, `extract_modern_accessor_type()`.

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;

use super::utils::ATTRIBUTE_CLASS;
use super::utils::accessor_method_candidates;
use super::utils::extract_short_name;
use super::utils::is_legacy_getter_accessor;

/// Try to resolve the type for a virtual property produced by an accessor.
///
/// Given a property name like `full_name` on a model class, looks for:
/// 1. A legacy accessor `getFullNameAttribute()` → uses its return type
/// 2. A modern accessor `fullName(): Attribute<string>` → extracts the
///    first generic argument from `Attribute`
///
/// Returns `Some(TUnion)` with the resolved type, or `None` if no
/// accessor matches.
pub fn resolve_accessor_property_type(
    property_name: &str,
    class_metadata: &ClassLikeMetadata,
    codebase: &CodebaseMetadata,
) -> Option<TUnion> {
    let candidates = accessor_method_candidates(property_name);

    for candidate in &candidates {
        if let Some(method_meta) = codebase.get_method(&class_metadata.name, candidate) {
            // Check if it's a legacy accessor (getXxxAttribute).
            if is_legacy_getter_accessor(candidate) {
                return Some(resolve_legacy_accessor_type(method_meta));
            }

            // Check if it's a modern accessor (returns Attribute).
            if is_modern_accessor_method(method_meta) {
                return Some(resolve_modern_accessor_type(method_meta));
            }
        }
    }

    None
}

/// Determine whether a method is a modern Eloquent accessor (Laravel 9+).
///
/// Modern accessors are methods that return
/// `Illuminate\Database\Eloquent\Casts\Attribute` (or just `Attribute`
/// as a short name).  The method name is in camelCase and the virtual
/// property name is the snake_case equivalent.
fn is_modern_accessor_method(method_meta: &FunctionLikeMetadata) -> bool {
    let return_type = match method_meta.return_type_metadata.as_ref().map(|tm| &tm.type_union) {
        Some(rt) => rt,
        None => return false,
    };

    for atomic in return_type.types.iter() {
        if let TAtomic::Object(TObject::Named(named)) = atomic {
            let name: &str = named.name.as_ref();
            if name.eq_ignore_ascii_case(ATTRIBUTE_CLASS) || extract_short_name(name).eq_ignore_ascii_case("Attribute")
            {
                return true;
            }
        }
    }

    false
}

/// Resolve the property type from a legacy accessor method.
///
/// Uses the method's return type directly.  Falls back to `mixed` if
/// no return type is declared.
fn resolve_legacy_accessor_type(method_meta: &FunctionLikeMetadata) -> TUnion {
    method_meta.return_type_metadata.as_ref().map(|tm| tm.type_union.clone()).unwrap_or_else(get_mixed)
}

/// Resolve the property type from a modern accessor method.
///
/// Extracts the first generic argument from `Attribute<TGet>` or
/// `Attribute<TGet, TSet>`.  Falls back to `mixed` if no generic
/// parameter is present.
fn resolve_modern_accessor_type(method_meta: &FunctionLikeMetadata) -> TUnion {
    let return_type = match method_meta.return_type_metadata.as_ref().map(|tm| &tm.type_union) {
        Some(rt) => rt,
        None => return get_mixed(),
    };

    for atomic in return_type.types.iter() {
        if let TAtomic::Object(TObject::Named(named)) = atomic {
            let name: &str = named.name.as_ref();
            if name.eq_ignore_ascii_case(ATTRIBUTE_CLASS) || extract_short_name(name).eq_ignore_ascii_case("Attribute")
            {
                // Extract the first generic argument (TGet).
                if let Some(first_param) = named.type_parameters.as_ref().and_then(|tp| tp.first()) {
                    return first_param.clone();
                }
                // No generic parameters — fall back to mixed.
                return get_mixed();
            }
        }
    }

    get_mixed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mago_atom::atom;
    use mago_codex::metadata::flags::MetadataFlags;
    use mago_codex::metadata::function_like::{FunctionLikeKind, FunctionLikeMetadata};
    use mago_codex::metadata::ttype::TypeMetadata;
    use mago_codex::ttype::atomic::object::named::TNamedObject;
    use mago_codex::ttype::get_string;
    use mago_span::Span;

    fn test_span() -> Span {
        Span::zero()
    }

    #[test]
    fn is_modern_accessor_with_attribute_return_type() {
        let return_type = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
            atom(ATTRIBUTE_CLASS),
            Some(vec![get_string()]),
        ))));

        let mut meta = FunctionLikeMetadata::new(FunctionLikeKind::Method, test_span(), MetadataFlags::empty());
        meta.return_type_metadata = Some(TypeMetadata::new(return_type, test_span()));

        assert!(is_modern_accessor_method(&meta));
    }

    #[test]
    fn is_not_modern_accessor_without_attribute_return() {
        let return_type = get_string();

        let mut meta = FunctionLikeMetadata::new(FunctionLikeKind::Method, test_span(), MetadataFlags::empty());
        meta.return_type_metadata = Some(TypeMetadata::new(return_type, test_span()));

        assert!(!is_modern_accessor_method(&meta));
    }

    #[test]
    fn is_not_modern_accessor_without_return_type() {
        let meta = FunctionLikeMetadata::new(FunctionLikeKind::Method, test_span(), MetadataFlags::empty());

        assert!(!is_modern_accessor_method(&meta));
    }

    #[test]
    fn resolve_modern_type_extracts_first_generic() {
        let return_type = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
            atom(ATTRIBUTE_CLASS),
            Some(vec![get_string()]),
        ))));

        let mut meta = FunctionLikeMetadata::new(FunctionLikeKind::Method, test_span(), MetadataFlags::empty());
        meta.return_type_metadata = Some(TypeMetadata::new(return_type, test_span()));

        let resolved = resolve_modern_accessor_type(&meta);
        assert!(resolved.is_string());
    }

    #[test]
    fn resolve_modern_type_without_generics_returns_mixed() {
        let return_type =
            TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom(ATTRIBUTE_CLASS)))));

        let mut meta = FunctionLikeMetadata::new(FunctionLikeKind::Method, test_span(), MetadataFlags::empty());
        meta.return_type_metadata = Some(TypeMetadata::new(return_type, test_span()));

        let resolved = resolve_modern_accessor_type(&meta);
        assert!(resolved.is_mixed());
    }

    #[test]
    fn resolve_legacy_type_uses_return_type() {
        let mut meta = FunctionLikeMetadata::new(FunctionLikeKind::Method, test_span(), MetadataFlags::empty());
        meta.return_type_metadata = Some(TypeMetadata::new(get_string(), test_span()));

        let resolved = resolve_legacy_accessor_type(&meta);
        assert!(resolved.is_string());
    }

    #[test]
    fn resolve_legacy_type_without_return_type_is_mixed() {
        let meta = FunctionLikeMetadata::new(FunctionLikeKind::Method, test_span(), MetadataFlags::empty());

        let resolved = resolve_legacy_accessor_type(&meta);
        assert!(resolved.is_mixed());
    }
}
