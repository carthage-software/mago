#![allow(dead_code)]

use std::collections::BTreeMap;
use std::sync::Arc;

use mago_atom::Atom;
use mago_atom::atom;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::mixed::TMixed;
use mago_codex::ttype::atomic::mixed::truthiness::TMixedTruthiness;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::resource::TResource;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::float::TFloat;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::combiner::CombinerOptions;
use mago_codex::ttype::combiner::combine;
use mago_codex::ttype::union::TUnion;

/// Returns a fresh, empty codebase. The combiner is mostly codebase-independent;
/// the parts that depend on it (object hierarchy collapse) live in their own targeted tests.
#[must_use]
pub fn empty_codebase() -> CodebaseMetadata {
    CodebaseMetadata::new()
}

/// Default combiner options used by 99% of tests.
#[must_use]
pub fn default_opts() -> CombinerOptions {
    CombinerOptions::default()
}

/// Combines `atomics` with the default codebase and options.
#[must_use]
pub fn combine_default(atomics: Vec<TAtomic>) -> Vec<TAtomic> {
    combine(atomics, &empty_codebase(), default_opts())
}

/// Combines `atomics` with overwrite_empty_array enabled.
#[must_use]
pub fn combine_overwrite(atomics: Vec<TAtomic>) -> Vec<TAtomic> {
    combine(atomics, &empty_codebase(), default_opts().with_overwrite_empty_array())
}

/// Combines with a custom integer threshold (other thresholds use defaults).
#[must_use]
pub fn combine_with_int_threshold(atomics: Vec<TAtomic>, threshold: u16) -> Vec<TAtomic> {
    combine(atomics, &empty_codebase(), default_opts().with_integer_combination_threshold(threshold))
}

/// Combines with a custom string threshold.
#[must_use]
pub fn combine_with_string_threshold(atomics: Vec<TAtomic>, threshold: u16) -> Vec<TAtomic> {
    combine(atomics, &empty_codebase(), default_opts().with_string_combination_threshold(threshold))
}

/// Combines with a custom array threshold.
#[must_use]
pub fn combine_with_array_threshold(atomics: Vec<TAtomic>, threshold: u16) -> Vec<TAtomic> {
    combine(atomics, &empty_codebase(), default_opts().with_array_combination_threshold(threshold))
}

/// Asserts that the combined atomics, compared as a multiset of `Atom` ids,
/// equal the multiset of ids of `expected`. Order in the output is implementation-defined.
pub fn assert_combines_to(input: Vec<TAtomic>, expected: Vec<TAtomic>) {
    let actual = combine_default(input);
    assert_multiset_eq(&actual, &expected);
}

pub fn assert_combines_to_with(input: Vec<TAtomic>, opts: CombinerOptions, expected: Vec<TAtomic>) {
    let actual = combine(input, &empty_codebase(), opts);
    assert_multiset_eq(&actual, &expected);
}

/// Asserts that two atomic vectors are multiset-equal. Uses `TAtomic::Eq` directly,
/// because two atomics with the same `get_id` are always semantically equal in the
/// combiner's output (the combiner canonicalises before returning).
pub fn assert_multiset_eq(actual: &[TAtomic], expected: &[TAtomic]) {
    let mut actual_keys: Vec<String> = actual.iter().map(atomic_id_string).collect();
    let mut expected_keys: Vec<String> = expected.iter().map(atomic_id_string).collect();
    actual_keys.sort();
    expected_keys.sort();

    assert_eq!(
        actual_keys, expected_keys,
        "\n  actual:   {actual:#?}\n  expected: {expected:#?}",
    );
}

/// Stable string id for an atomic, used by `assert_multiset_eq`.
pub fn atomic_id_string(a: &TAtomic) -> String {
    a.get_id().to_string()
}

/// Asserts that the combined atomics produce exactly one element matching `predicate`.
pub fn assert_single<F: Fn(&TAtomic) -> bool>(input: Vec<TAtomic>, predicate: F) {
    let result = combine_default(input);
    assert_eq!(result.len(), 1, "expected single atom, got: {result:#?}");
    assert!(predicate(&result[0]), "predicate failed for: {:#?}", result[0]);
}

/// Asserts that combining a vector of `n` copies of `a` returns a single `a` (idempotency).
pub fn assert_self_idempotent(a: TAtomic, n: usize) {
    let input: Vec<TAtomic> = std::iter::repeat_n(a.clone(), n).collect();
    let out = combine_default(input);
    assert_eq!(out.len(), 1, "self-combination should produce 1 atom for {a:?}, got {out:?}");
    assert_eq!(out[0].get_id(), a.get_id(), "self-combination should preserve id for {a:?}");
}

/// Asserts that `combine([a, b]) == combine([b, a])` (commutativity).
pub fn assert_commutative(a: TAtomic, b: TAtomic) {
    let ab = combine_default(vec![a.clone(), b.clone()]);
    let ba = combine_default(vec![b.clone(), a.clone()]);
    let mut a_keys: Vec<String> = ab.iter().map(atomic_id_string).collect();
    let mut b_keys: Vec<String> = ba.iter().map(atomic_id_string).collect();
    a_keys.sort();
    b_keys.sort();
    assert_eq!(a_keys, b_keys, "combine is not commutative for {a:?} ∨ {b:?}");
}

#[must_use]
pub fn never() -> TAtomic {
    TAtomic::Never
}
#[must_use]
pub fn null() -> TAtomic {
    TAtomic::Null
}
#[must_use]
pub fn void() -> TAtomic {
    TAtomic::Void
}
#[must_use]
pub fn placeholder() -> TAtomic {
    TAtomic::Placeholder
}
#[must_use]
pub fn mixed() -> TAtomic {
    TAtomic::Mixed(TMixed::new())
}
#[must_use]
pub fn mixed_truthy() -> TAtomic {
    TAtomic::Mixed(TMixed::new().with_truthiness(TMixedTruthiness::Truthy))
}
#[must_use]
pub fn mixed_falsy() -> TAtomic {
    TAtomic::Mixed(TMixed::new().with_truthiness(TMixedTruthiness::Falsy))
}
#[must_use]
pub fn mixed_nonnull() -> TAtomic {
    TAtomic::Mixed(TMixed::new().with_is_non_null(true))
}

#[must_use]
pub fn t_true() -> TAtomic {
    TAtomic::Scalar(TScalar::r#true())
}
#[must_use]
pub fn t_false() -> TAtomic {
    TAtomic::Scalar(TScalar::r#false())
}
#[must_use]
pub fn t_bool() -> TAtomic {
    TAtomic::Scalar(TScalar::bool())
}

#[must_use]
pub fn t_int() -> TAtomic {
    TAtomic::Scalar(TScalar::int())
}
#[must_use]
pub fn t_lit_int(v: i64) -> TAtomic {
    TAtomic::Scalar(TScalar::Integer(TInteger::literal(v)))
}
#[must_use]
pub fn t_int_from(from: i64) -> TAtomic {
    TAtomic::Scalar(TScalar::Integer(TInteger::From(from)))
}
#[must_use]
pub fn t_int_to(to: i64) -> TAtomic {
    TAtomic::Scalar(TScalar::Integer(TInteger::To(to)))
}
#[must_use]
pub fn t_int_range(lo: i64, hi: i64) -> TAtomic {
    TAtomic::Scalar(TScalar::Integer(TInteger::Range(lo, hi)))
}
#[must_use]
pub fn t_int_unspec_lit() -> TAtomic {
    TAtomic::Scalar(TScalar::Integer(TInteger::UnspecifiedLiteral))
}
#[must_use]
pub fn t_positive_int() -> TAtomic {
    TAtomic::Scalar(TScalar::Integer(TInteger::positive()))
}
#[must_use]
pub fn t_negative_int() -> TAtomic {
    TAtomic::Scalar(TScalar::Integer(TInteger::negative()))
}
#[must_use]
pub fn t_non_negative_int() -> TAtomic {
    TAtomic::Scalar(TScalar::Integer(TInteger::non_negative()))
}
#[must_use]
pub fn t_non_positive_int() -> TAtomic {
    TAtomic::Scalar(TScalar::Integer(TInteger::non_positive()))
}

#[must_use]
pub fn t_float() -> TAtomic {
    TAtomic::Scalar(TScalar::float())
}
#[must_use]
pub fn t_lit_float(v: f64) -> TAtomic {
    TAtomic::Scalar(TScalar::literal_float(v))
}
#[must_use]
pub fn t_unspec_lit_float() -> TAtomic {
    TAtomic::Scalar(TScalar::Float(TFloat::UnspecifiedLiteral))
}

#[must_use]
pub fn t_string() -> TAtomic {
    TAtomic::Scalar(TScalar::string())
}
#[must_use]
pub fn t_lit_string(s: &str) -> TAtomic {
    TAtomic::Scalar(TScalar::literal_string(atom(s)))
}
#[must_use]
pub fn t_non_empty_string() -> TAtomic {
    TAtomic::Scalar(TScalar::non_empty_string())
}
#[must_use]
pub fn t_numeric_string() -> TAtomic {
    TAtomic::Scalar(TScalar::numeric_string())
}
#[must_use]
pub fn t_lower_string() -> TAtomic {
    TAtomic::Scalar(TScalar::String(TString::lowercase()))
}
#[must_use]
pub fn t_upper_string() -> TAtomic {
    TAtomic::Scalar(TScalar::String(TString::uppercase()))
}
#[must_use]
pub fn t_truthy_string() -> TAtomic {
    TAtomic::Scalar(TScalar::String(TString::truthy()))
}
#[must_use]
pub fn t_unspec_lit_string(non_empty: bool) -> TAtomic {
    TAtomic::Scalar(TScalar::unspecified_literal_string(non_empty))
}
#[must_use]
pub fn t_callable_string() -> TAtomic {
    TAtomic::Scalar(TScalar::String(TString::callable()))
}

#[must_use]
pub fn t_array_key() -> TAtomic {
    TAtomic::Scalar(TScalar::array_key())
}
#[must_use]
pub fn t_numeric() -> TAtomic {
    TAtomic::Scalar(TScalar::numeric())
}
#[must_use]
pub fn t_scalar() -> TAtomic {
    TAtomic::Scalar(TScalar::generic())
}

#[must_use]
pub fn t_class_string() -> TAtomic {
    TAtomic::Scalar(TScalar::class_string())
}
#[must_use]
pub fn t_interface_string() -> TAtomic {
    TAtomic::Scalar(TScalar::interface_string())
}
#[must_use]
pub fn t_enum_string() -> TAtomic {
    TAtomic::Scalar(TScalar::enum_string())
}
#[must_use]
pub fn t_trait_string() -> TAtomic {
    TAtomic::Scalar(TScalar::trait_string())
}
#[must_use]
pub fn t_lit_class_string(name: &str) -> TAtomic {
    TAtomic::Scalar(TScalar::literal_class_string(atom(name)))
}

// Resources
#[must_use]
pub fn t_resource() -> TAtomic {
    TAtomic::Resource(TResource { closed: None })
}
#[must_use]
pub fn t_open_resource() -> TAtomic {
    TAtomic::Resource(TResource { closed: Some(false) })
}
#[must_use]
pub fn t_closed_resource() -> TAtomic {
    TAtomic::Resource(TResource { closed: Some(true) })
}

// Objects
#[must_use]
pub fn t_object_any() -> TAtomic {
    TAtomic::Object(TObject::Any)
}
#[must_use]
pub fn t_named(name: &str) -> TAtomic {
    TAtomic::Object(TObject::new_named(atom(name)))
}
#[must_use]
pub fn t_generic_named(name: &str, params: Vec<TUnion>) -> TAtomic {
    TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(atom(name), Some(params))))
}
#[must_use]
pub fn t_enum(name: &str) -> TAtomic {
    TAtomic::Object(TObject::new_enum(atom(name)))
}
#[must_use]
pub fn t_enum_case(name: &str, case: &str) -> TAtomic {
    TAtomic::Object(TObject::new_enum_case(atom(name), atom(case)))
}

// Arrays
#[must_use]
pub fn t_empty_array() -> TAtomic {
    TAtomic::Array(TArray::Keyed(TKeyedArray { known_items: None, parameters: None, non_empty: false }))
}
#[must_use]
pub fn t_keyed_unsealed(key: TUnion, value: TUnion, non_empty: bool) -> TAtomic {
    TAtomic::Array(TArray::Keyed(TKeyedArray {
        known_items: None,
        parameters: Some((Arc::new(key), Arc::new(value))),
        non_empty,
    }))
}
#[must_use]
pub fn t_keyed_sealed(items: BTreeMap<ArrayKey, (bool, TUnion)>, non_empty: bool) -> TAtomic {
    TAtomic::Array(TArray::Keyed(TKeyedArray { known_items: Some(items), parameters: None, non_empty }))
}
#[must_use]
pub fn t_list(element: TUnion, non_empty: bool) -> TAtomic {
    let l = TList::new(Arc::new(element));
    TAtomic::Array(TArray::List(if non_empty { l.clone_non_empty() } else { l }))
}
#[must_use]
pub fn t_sealed_list(known: BTreeMap<usize, (bool, TUnion)>) -> TAtomic {
    TAtomic::Array(TArray::List(TList::from_known_elements(known)))
}

// Helpers for building TUnion easily
#[must_use]
pub fn u(a: TAtomic) -> TUnion {
    TUnion::from_atomic(a)
}

#[must_use]
pub fn ui(v: i64) -> TUnion {
    u(t_lit_int(v))
}

#[must_use]
pub fn us(s: &str) -> TUnion {
    u(t_lit_string(s))
}

#[must_use]
pub fn ak_int(n: i64) -> ArrayKey {
    ArrayKey::Integer(n)
}

#[must_use]
pub fn ak_str(s: &str) -> ArrayKey {
    ArrayKey::String(atom(s))
}

#[must_use]
pub fn name_atom(s: &str) -> Atom {
    atom(s)
}
