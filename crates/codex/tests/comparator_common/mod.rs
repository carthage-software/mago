#![allow(dead_code)]

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::Arc;

use bumpalo::Bump;
use mago_atom::Atom;
use mago_atom::atom;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::populator::populate_codebase;
use mago_codex::reference::SymbolReferences;
use mago_codex::scanner::scan_program;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::atomic::iterable::TIterable;
use mago_codex::ttype::atomic::mixed::TMixed;
use mago_codex::ttype::atomic::mixed::truthiness::TMixedTruthiness;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::resource::TResource;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::float::TFloat;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::atomic_comparator;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::union::TUnion;

use mago_database::Database;
use mago_database::DatabaseReader;
use mago_database::file::File;
use mago_names::resolver::NameResolver;
use mago_syntax::parser::parse_file;

#[must_use]
pub fn empty_codebase() -> CodebaseMetadata {
    CodebaseMetadata::new()
}

#[must_use]
pub fn codebase_from_php(code: &'static str) -> CodebaseMetadata {
    let file = File::ephemeral(Cow::Borrowed("code.php"), Cow::Borrowed(code));
    let config = mago_database::DatabaseConfiguration::new(std::path::Path::new("/"), vec![], vec![], vec![], vec![])
        .into_static();
    let database = Database::single(file, config);

    let mut codebase = CodebaseMetadata::new();
    let arena = Bump::new();
    for file in database.files() {
        let program = parse_file(&arena, &file);
        assert!(!program.has_errors(), "Parse failed: {:?}", program.errors);
        let resolved_names = NameResolver::new(&arena).resolve(program);
        let program_codebase = scan_program(&arena, &file, program, &resolved_names);

        codebase.extend(program_codebase);
    }

    populate_codebase(&mut codebase, &mut SymbolReferences::new(), Default::default(), Default::default());

    codebase
}

pub fn is_contained(input: &TUnion, container: &TUnion, codebase: &CodebaseMetadata) -> bool {
    let mut r = ComparisonResult::new();
    union_comparator::is_contained_by(codebase, input, container, false, false, false, &mut r)
}

pub fn is_contained_with(
    input: &TUnion,
    container: &TUnion,
    codebase: &CodebaseMetadata,
    ignore_null: bool,
    ignore_false: bool,
    inside_assertion: bool,
) -> bool {
    let mut r = ComparisonResult::new();
    union_comparator::is_contained_by(codebase, input, container, ignore_null, ignore_false, inside_assertion, &mut r)
}

pub fn is_contained_capturing(
    input: &TUnion,
    container: &TUnion,
    codebase: &CodebaseMetadata,
) -> (bool, ComparisonResult) {
    let mut r = ComparisonResult::new();
    let v = union_comparator::is_contained_by(codebase, input, container, false, false, false, &mut r);
    (v, r)
}

pub fn atomic_is_contained(input: &TAtomic, container: &TAtomic, codebase: &CodebaseMetadata) -> bool {
    let mut r = ComparisonResult::new();
    atomic_comparator::is_contained_by(codebase, input, container, false, &mut r)
}

pub fn atomic_is_contained_capturing(
    input: &TAtomic,
    container: &TAtomic,
    codebase: &CodebaseMetadata,
) -> (bool, ComparisonResult) {
    let mut r = ComparisonResult::new();
    let v = atomic_comparator::is_contained_by(codebase, input, container, false, &mut r);
    (v, r)
}

pub fn assert_subtype(input: &TUnion, container: &TUnion) {
    let cb = empty_codebase();
    assert!(is_contained(input, container, &cb), "expected {input:?} <: {container:?} but it is not");
}

pub fn assert_not_subtype(input: &TUnion, container: &TUnion) {
    let cb = empty_codebase();
    assert!(!is_contained(input, container, &cb), "expected NOT ({input:?} <: {container:?}) but it is");
}

pub fn assert_atomic_subtype(input: &TAtomic, container: &TAtomic) {
    let cb = empty_codebase();
    assert!(atomic_is_contained(input, container, &cb), "expected atomic {input:?} <: {container:?}");
}

pub fn assert_atomic_not_subtype(input: &TAtomic, container: &TAtomic) {
    let cb = empty_codebase();
    assert!(!atomic_is_contained(input, container, &cb), "expected NOT (atomic {input:?} <: {container:?})");
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
pub fn t_keyed_with_both(
    key: TUnion,
    value: TUnion,
    items: BTreeMap<ArrayKey, (bool, TUnion)>,
    non_empty: bool,
) -> TAtomic {
    TAtomic::Array(TArray::Keyed(TKeyedArray {
        known_items: Some(items),
        parameters: Some((Arc::new(key), Arc::new(value))),
        non_empty,
    }))
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

#[must_use]
pub fn t_iterable(key: TUnion, value: TUnion) -> TAtomic {
    TAtomic::Iterable(TIterable::new(Arc::new(key), Arc::new(value)))
}

#[must_use]
pub fn t_callable_mixed() -> TAtomic {
    use mago_codex::ttype::atomic::callable::TCallableSignature;
    TAtomic::Callable(TCallable::Signature(TCallableSignature::mixed(false)))
}
#[must_use]
pub fn t_closure_mixed() -> TAtomic {
    use mago_codex::ttype::atomic::callable::TCallableSignature;
    TAtomic::Callable(TCallable::Signature(TCallableSignature::mixed(true)))
}

#[must_use]
pub fn u(a: TAtomic) -> TUnion {
    TUnion::from_atomic(a)
}

#[must_use]
pub fn u_many(atoms: Vec<TAtomic>) -> TUnion {
    TUnion::from_vec(atoms)
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
pub fn name(s: &str) -> Atom {
    atom(s)
}
