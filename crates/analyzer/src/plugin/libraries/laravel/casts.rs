//! Cast property type resolution for Laravel Eloquent models.
//!
//! When a model declares `protected $casts = ['status' => 'string', ...]`,
//! accessing `$model->status` should resolve to `string`.
//!
//! This module provides functions to:
//! - Extract cast definitions from a model class's `$casts` property metadata
//! - Map cast type strings to Mago `TUnion` types
//! - Handle custom cast classes (enums, Castable, CastsAttributes)
//!
//! Derived from phpantom_lsp's `LaravelModelProvider`:
//! `cast_type_to_php_type()`, `extract_tget_from_implements_generics()`,
//! `is_castable()`.

use mago_atom::atom;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;

use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::get_bool;
use mago_codex::ttype::get_float;
use mago_codex::ttype::get_int;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_object;
use mago_codex::ttype::get_string;
use mago_codex::ttype::union::TUnion;

use super::utils::CASTABLE;
use super::utils::CASTS_ATTRIBUTES;
use super::utils::CASTS_ATTRIBUTES_SHORT;
use super::utils::cast_type_to_php_type;
use super::utils::extract_short_name;
use super::utils::strip_leading_backslash;

/// Try to resolve the type for a cast property on a model.
///
/// Looks up the `$casts` property on the class, extracts the array entries,
/// and maps the cast type string for the given column name to a `TUnion`.
///
/// Returns `Some(TUnion)` with the resolved type, or `None` if the
/// property is not in the casts array.
pub fn resolve_cast_property_type(
    property_name: &str,
    class_metadata: &ClassLikeMetadata,
    codebase: &CodebaseMetadata,
) -> Option<TUnion> {
    let casts = extract_casts_from_class(class_metadata)?;

    let cast_type =
        casts.iter().find_map(|(col, cast_str)| if col == property_name { Some(cast_str.as_str()) } else { None })?;

    Some(resolve_cast_type(cast_type, codebase))
}

/// Try to resolve the type for an attribute default property on a model.
///
/// Looks up the `$attributes` property on the class, extracts the array entries,
/// and infers the type from the default value literal.
///
/// Returns `Some(TUnion)` for columns that have defaults but are NOT already
/// covered by `$casts`.  The caller should check casts first.
pub fn resolve_attribute_default_type(property_name: &str, class_metadata: &ClassLikeMetadata) -> Option<TUnion> {
    let defaults = extract_attribute_defaults_from_class(class_metadata)?;

    defaults.iter().find_map(|(col, ty)| if col == property_name { Some(ty.clone()) } else { None })
}

/// Check if a property name appears in the `$fillable`, `$guarded`, or `$hidden`
/// arrays on the model.  These are column name properties that get a `mixed`
/// fallback type.
///
/// Returns `true` if the property appears in any of these arrays.
pub fn is_column_name_property(property_name: &str, class_metadata: &ClassLikeMetadata) -> bool {
    for prop_key in &[atom("$fillable"), atom("$guarded"), atom("$hidden")] {
        if let Some(prop_meta) = class_metadata.properties.get(prop_key)
            && let Some(ref default_type) = prop_meta.default_type_metadata
            && array_type_contains_string_value(&default_type.type_union, property_name)
        {
            return true;
        }
    }
    false
}

/// Check if a property name appears in the `$casts` array.
pub fn is_cast_property(property_name: &str, class_metadata: &ClassLikeMetadata) -> bool {
    if let Some(casts) = extract_casts_from_class(class_metadata) {
        return casts.iter().any(|(col, _)| col == property_name);
    }
    false
}

/// Check if a property name appears in the `$attributes` defaults array.
pub fn is_attribute_default_property(property_name: &str, class_metadata: &ClassLikeMetadata) -> bool {
    if let Some(defaults) = extract_attribute_defaults_from_class(class_metadata) {
        return defaults.iter().any(|(col, _)| col == property_name);
    }
    false
}

// ────────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ────────────────────────────────────────────────────────────────────────────────

/// Extract cast definitions from the `$casts` property on a class.
///
/// Returns a list of `(column_name, cast_type_string)` pairs extracted
/// from the keyed array in the property's default value type metadata.
fn extract_casts_from_class(class_metadata: &ClassLikeMetadata) -> Option<Vec<(String, String)>> {
    let casts_prop = class_metadata.properties.get(&atom("$casts"))?;
    let default_type = casts_prop.default_type_metadata.as_ref()?;

    extract_string_pairs_from_array(&default_type.type_union)
}

/// Extract attribute defaults from the `$attributes` property on a class.
///
/// Returns a list of `(column_name, inferred_type)` pairs.
fn extract_attribute_defaults_from_class(class_metadata: &ClassLikeMetadata) -> Option<Vec<(String, TUnion)>> {
    let attrs_prop = class_metadata.properties.get(&atom("$attributes"))?;
    let default_type = attrs_prop.default_type_metadata.as_ref()?;

    extract_key_value_types_from_array(&default_type.type_union)
}

/// Extract string key-value pairs from an array type.
///
/// Given a type like `array{status: 'string', age: 'integer'}`,
/// returns `[("status", "string"), ("age", "integer")]`.
fn extract_string_pairs_from_array(type_union: &TUnion) -> Option<Vec<(String, String)>> {
    let mut pairs = Vec::new();

    for atomic in type_union.types.iter() {
        if let TAtomic::Array(TArray::Keyed(keyed)) = atomic
            && let Some(ref known_items) = keyed.known_items
        {
            for (key, (_optional, value)) in known_items {
                if let ArrayKey::String(key_atom) = key {
                    let key_str: &str = key_atom.as_ref();
                    if let Some(val_str) = value.get_single_literal_string_value() {
                        pairs.push((key_str.to_string(), val_str.to_string()));
                    }
                }
            }
        }
    }

    if pairs.is_empty() { None } else { Some(pairs) }
}

/// Extract key→type pairs from an array type.
///
/// Given a type like `array{name: 'John', age: 0}`,
/// returns `[("name", string), ("age", int)]` with the inferred types
/// of the default values.
fn extract_key_value_types_from_array(type_union: &TUnion) -> Option<Vec<(String, TUnion)>> {
    let mut pairs = Vec::new();

    for atomic in type_union.types.iter() {
        if let TAtomic::Array(TArray::Keyed(keyed)) = atomic
            && let Some(ref known_items) = keyed.known_items
        {
            for (key, (_optional, value)) in known_items {
                if let ArrayKey::String(key_atom) = key {
                    let key_str: &str = key_atom.as_ref();
                    pairs.push((key_str.to_string(), value.clone()));
                }
            }
        }
    }

    if pairs.is_empty() { None } else { Some(pairs) }
}

/// Check if an array type contains a specific string literal value.
///
/// Used to check if a column name appears in `$fillable`, `$guarded`, or `$hidden`.
fn array_type_contains_string_value(type_union: &TUnion, value: &str) -> bool {
    for atomic in type_union.types.iter() {
        if let TAtomic::Array(TArray::Keyed(keyed)) = atomic {
            if let Some(ref known_items) = keyed.known_items {
                for (_optional, item_type) in known_items.values() {
                    if let Some(literal) = item_type.get_single_literal_string_value()
                        && literal == value
                    {
                        return true;
                    }
                }
            }
            // Also check the general value parameter.
            if let Some((_key_type, value_type)) = &keyed.parameters
                && let Some(literal) = value_type.get_single_literal_string_value()
                && literal == value
            {
                return true;
            }
        }
    }
    false
}

/// Map a cast type string to a Mago `TUnion`.
///
/// Handles:
/// 1. Built-in cast strings via `cast_type_to_php_type()` (int, string, datetime, etc.)
/// 2. Custom cast classes with a `get()` method return type
/// 3. Enum casts (the property type is the enum itself)
/// 4. `Castable` implementations (the property type is the class itself)
/// 5. Fallback to `@implements CastsAttributes<TGet, TSet>` generics
/// 6. Strip `:argument` suffixes (e.g. `Address::class.':nullable'`)
///
/// Falls back to `mixed` for unrecognized types.
fn resolve_cast_type(cast_type: &str, codebase: &CodebaseMetadata) -> TUnion {
    // 1. Try the built-in cast type mapping.
    if let Some(php_type) = cast_type_to_php_type(cast_type) {
        return php_type_string_to_tunion(php_type);
    }

    // 2. Strip `:argument` suffix for class-based casts.
    let class_part = cast_type.split(':').next().unwrap_or(cast_type);
    let clean = strip_leading_backslash(class_part);

    // 3. Look up the class in the codebase.
    let Some(cast_class) = codebase.get_class_like(clean) else {
        return get_mixed();
    };

    // 3a. Enums — the property type is the enum itself.
    if cast_class.kind.is_enum() {
        return TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(cast_class.name))));
    }

    // 3b. Castable implementations.
    if is_castable(cast_class, codebase) {
        return TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(cast_class.name))));
    }

    // 3c. Check `get()` method return type on CastsAttributes classes.
    if let Some(get_method) = codebase.get_method(&cast_class.name, "get")
        && let Some(ref return_type_meta) = get_method.return_type_metadata
    {
        return return_type_meta.type_union.clone();
    }

    // 3d. Fallback: extract TGet from `@implements CastsAttributes<TGet, TSet>`.
    if let Some(tget) = extract_tget_from_implements_generics(cast_class) {
        return tget;
    }

    // 4. Unknown cast type.
    get_mixed()
}

/// Check whether a class implements the `Castable` contract.
fn is_castable(class_metadata: &ClassLikeMetadata, codebase: &CodebaseMetadata) -> bool {
    // Check direct parent interfaces.
    for iface in class_metadata.all_parent_interfaces.iter() {
        let iface_str: &str = iface.as_ref();
        if iface_str.eq_ignore_ascii_case(CASTABLE) || extract_short_name(iface_str).eq_ignore_ascii_case("Castable") {
            return true;
        }
    }

    // Also check via is_instance_of for indirect implementations.
    codebase.is_instance_of(&class_metadata.name, CASTABLE)
}

/// Extract the `TGet` type from a cast class's template extended parameters
/// for `CastsAttributes<TGet, TSet>`.
///
/// Returns the first generic argument if the class declares template
/// parameters for `CastsAttributes` (matched by short name or FQN).
fn extract_tget_from_implements_generics(class_metadata: &ClassLikeMetadata) -> Option<TUnion> {
    // Look through template_extended_offsets for CastsAttributes.
    for (parent_name, type_params) in class_metadata.template_extended_offsets.iter() {
        let name_str: &str = parent_name.as_ref();
        if (name_str.eq_ignore_ascii_case(CASTS_ATTRIBUTES)
            || extract_short_name(name_str).eq_ignore_ascii_case(CASTS_ATTRIBUTES_SHORT))
            && let Some(first) = type_params.first()
            && !first.is_never()
            && !first.is_mixed()
        {
            return Some(first.clone());
        }
    }

    None
}

/// Convert a PHP type string (from `cast_type_to_php_type`) to a `TUnion`.
fn php_type_string_to_tunion(php_type: &str) -> TUnion {
    match php_type {
        "int" => get_int(),
        "float" => get_float(),
        "string" => get_string(),
        "bool" => get_bool(),
        "array" => {
            // Return a generic array type.
            use mago_codex::ttype::get_mixed_keyed_array;
            get_mixed_keyed_array()
        }
        "object" => get_object(),
        _ => {
            // FQN class reference like `\Illuminate\Support\Carbon`.
            let clean = strip_leading_backslash(php_type);
            TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(atom(clean)))))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn php_type_string_int() {
        let ty = php_type_string_to_tunion("int");
        assert!(ty.is_int());
    }

    #[test]
    fn php_type_string_float() {
        let ty = php_type_string_to_tunion("float");
        assert!(ty.is_float());
    }

    #[test]
    fn php_type_string_string() {
        let ty = php_type_string_to_tunion("string");
        assert!(ty.is_string());
    }

    #[test]
    fn php_type_string_bool() {
        let ty = php_type_string_to_tunion("bool");
        assert!(ty.is_bool());
    }

    #[test]
    fn php_type_string_object() {
        let ty = php_type_string_to_tunion("object");
        assert!(ty.is_objecty());
    }

    #[test]
    fn php_type_string_named_class() {
        let ty = php_type_string_to_tunion("\\Illuminate\\Support\\Carbon");
        assert!(ty.is_single());
        let single = ty.get_single();
        if let TAtomic::Object(TObject::Named(named)) = single {
            let name: &str = named.name.as_ref();
            assert_eq!(name, "Illuminate\\Support\\Carbon");
        } else {
            panic!("Expected named object, got {:?}", single);
        }
    }

    #[test]
    fn resolve_builtin_cast_datetime() {
        let codebase = CodebaseMetadata::default();
        let ty = resolve_cast_type("datetime", &codebase);
        assert!(ty.is_single());
        if let TAtomic::Object(TObject::Named(named)) = ty.get_single() {
            let name: &str = named.name.as_ref();
            assert_eq!(name, "Illuminate\\Support\\Carbon");
        } else {
            panic!("Expected Carbon, got {:?}", ty);
        }
    }

    #[test]
    fn resolve_builtin_cast_boolean() {
        let codebase = CodebaseMetadata::default();
        let ty = resolve_cast_type("boolean", &codebase);
        assert!(ty.is_bool());
    }

    #[test]
    fn resolve_builtin_cast_integer() {
        let codebase = CodebaseMetadata::default();
        let ty = resolve_cast_type("integer", &codebase);
        assert!(ty.is_int());
    }

    #[test]
    fn resolve_builtin_cast_decimal() {
        let codebase = CodebaseMetadata::default();
        let ty = resolve_cast_type("decimal:2", &codebase);
        assert!(ty.is_float());
    }

    #[test]
    fn resolve_unknown_cast_without_codebase_class() {
        let codebase = CodebaseMetadata::default();
        let ty = resolve_cast_type("App\\Casts\\CustomCast", &codebase);
        assert!(ty.is_mixed());
    }
}
