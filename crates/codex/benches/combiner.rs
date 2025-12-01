use std::collections::BTreeMap;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use mago_atom::ascii_lowercase_atom;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::combiner::combine;
use mago_codex::ttype::get_int;
use mago_codex::ttype::get_string;

/// Benchmark combining simple scalar types (int, string, float, bool)
fn bench_combine_simple_scalars(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();

    c.bench_function("combine_3_scalars", |b| {
        b.iter(|| {
            let types = vec![
                TAtomic::Scalar(TScalar::int()),
                TAtomic::Scalar(TScalar::string()),
                TAtomic::Scalar(TScalar::float()),
            ];
            std::hint::black_box(combine(types, &codebase, false))
        })
    });

    c.bench_function("combine_4_scalars_to_scalar", |b| {
        b.iter(|| {
            let types = vec![
                TAtomic::Scalar(TScalar::int()),
                TAtomic::Scalar(TScalar::string()),
                TAtomic::Scalar(TScalar::float()),
                TAtomic::Scalar(TScalar::bool()),
            ];
            std::hint::black_box(combine(types, &codebase, false))
        })
    });

    c.bench_function("combine_true_false_to_bool", |b| {
        b.iter(|| {
            let types = vec![TAtomic::Scalar(TScalar::r#true()), TAtomic::Scalar(TScalar::r#false())];
            std::hint::black_box(combine(types, &codebase, false))
        })
    });
}

/// Benchmark combining literal integers
fn bench_combine_integers(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();

    c.bench_function("combine_5_literal_ints", |b| {
        b.iter(|| {
            let types = vec![
                TAtomic::Scalar(TScalar::Integer(TInteger::literal(1))),
                TAtomic::Scalar(TScalar::Integer(TInteger::literal(2))),
                TAtomic::Scalar(TScalar::Integer(TInteger::literal(3))),
                TAtomic::Scalar(TScalar::Integer(TInteger::literal(4))),
                TAtomic::Scalar(TScalar::Integer(TInteger::literal(5))),
            ];
            std::hint::black_box(combine(types, &codebase, false))
        })
    });

    c.bench_function("combine_int_ranges", |b| {
        b.iter(|| {
            let types = vec![
                TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, 10))),
                TAtomic::Scalar(TScalar::Integer(TInteger::Range(5, 20))),
                TAtomic::Scalar(TScalar::Integer(TInteger::literal(100))),
            ];
            std::hint::black_box(combine(types, &codebase, false))
        })
    });
}

/// Benchmark combining object types (tests class hierarchy checks)
fn bench_combine_objects(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();

    c.bench_function("combine_3_named_objects", |b| {
        b.iter(|| {
            let types = vec![
                TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("Foo")))),
                TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("Bar")))),
                TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("Baz")))),
            ];
            std::hint::black_box(combine(types, &codebase, false))
        })
    });

    c.bench_function("combine_5_named_objects", |b| {
        b.iter(|| {
            let types = vec![
                TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("Foo")))),
                TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("Bar")))),
                TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("Baz")))),
                TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("Qux")))),
                TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("Quux")))),
            ];
            std::hint::black_box(combine(types, &codebase, false))
        })
    });
}

/// Benchmark combining generic object types (tests type parameter combining)
fn bench_combine_generic_types(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();

    c.bench_function("combine_generic_object_1_param", |b| {
        b.iter(|| {
            let types = vec![
                TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
                    ascii_lowercase_atom("Container"),
                    Some(vec![get_int()]),
                ))),
                TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
                    ascii_lowercase_atom("Container"),
                    Some(vec![get_string()]),
                ))),
            ];
            std::hint::black_box(combine(types, &codebase, false))
        })
    });

    c.bench_function("combine_generic_object_3_params", |b| {
        b.iter(|| {
            let types = vec![
                TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
                    ascii_lowercase_atom("Container"),
                    Some(vec![get_int(), get_string(), get_int()]),
                ))),
                TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
                    ascii_lowercase_atom("Container"),
                    Some(vec![get_string(), get_int(), get_string()]),
                ))),
            ];
            std::hint::black_box(combine(types, &codebase, false))
        })
    });
}

/// Benchmark combining list arrays with known elements
fn bench_combine_list_arrays(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();

    c.bench_function("combine_2_lists_3_elements", |b| {
        b.iter(|| {
            let types = vec![
                TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([
                    (0, (false, get_int())),
                    (1, (false, get_string())),
                    (2, (false, get_int())),
                ])))),
                TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([
                    (0, (false, get_string())),
                    (1, (false, get_int())),
                    (2, (false, get_string())),
                ])))),
            ];
            std::hint::black_box(combine(types, &codebase, false))
        })
    });

    c.bench_function("combine_list_with_generic_list", |b| {
        b.iter(|| {
            let types = vec![
                TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([
                    (0, (false, get_int())),
                    (1, (false, get_int())),
                ])))),
                TAtomic::Array(TArray::List(TList::new(Box::new(get_int())))),
            ];
            std::hint::black_box(combine(types, &codebase, false))
        })
    });
}

/// Benchmark combining keyed arrays
fn bench_combine_keyed_arrays(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();

    c.bench_function("combine_2_keyed_arrays_3_keys", |b| {
        b.iter(|| {
            let types = vec![
                TAtomic::Array(TArray::Keyed(TKeyedArray::new().with_known_items(BTreeMap::from_iter([
                    (ArrayKey::String(ascii_lowercase_atom("a")), (false, get_int())),
                    (ArrayKey::String(ascii_lowercase_atom("b")), (false, get_string())),
                    (ArrayKey::String(ascii_lowercase_atom("c")), (false, get_int())),
                ])))),
                TAtomic::Array(TArray::Keyed(TKeyedArray::new().with_known_items(BTreeMap::from_iter([
                    (ArrayKey::String(ascii_lowercase_atom("a")), (false, get_string())),
                    (ArrayKey::String(ascii_lowercase_atom("b")), (false, get_int())),
                    (ArrayKey::String(ascii_lowercase_atom("d")), (false, get_string())),
                ])))),
            ];
            std::hint::black_box(combine(types, &codebase, false))
        })
    });

    c.bench_function("combine_keyed_5_keys_each", |b| {
        b.iter(|| {
            let types = vec![
                TAtomic::Array(TArray::Keyed(TKeyedArray::new().with_known_items(BTreeMap::from_iter([
                    (ArrayKey::String(ascii_lowercase_atom("a")), (false, get_int())),
                    (ArrayKey::String(ascii_lowercase_atom("b")), (false, get_string())),
                    (ArrayKey::String(ascii_lowercase_atom("c")), (false, get_int())),
                    (ArrayKey::String(ascii_lowercase_atom("d")), (false, get_string())),
                    (ArrayKey::String(ascii_lowercase_atom("e")), (false, get_int())),
                ])))),
                TAtomic::Array(TArray::Keyed(TKeyedArray::new().with_known_items(BTreeMap::from_iter([
                    (ArrayKey::String(ascii_lowercase_atom("a")), (false, get_string())),
                    (ArrayKey::String(ascii_lowercase_atom("b")), (false, get_int())),
                    (ArrayKey::String(ascii_lowercase_atom("c")), (false, get_string())),
                    (ArrayKey::String(ascii_lowercase_atom("f")), (false, get_int())),
                    (ArrayKey::String(ascii_lowercase_atom("g")), (false, get_string())),
                ])))),
            ];
            std::hint::black_box(combine(types, &codebase, false))
        })
    });
}

/// Benchmark combining mixed scenarios
fn bench_combine_mixed(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();

    c.bench_function("combine_null_with_type", |b| {
        b.iter(|| {
            let types = vec![TAtomic::Null, TAtomic::Scalar(TScalar::int())];
            std::hint::black_box(combine(types, &codebase, false))
        })
    });

    c.bench_function("combine_mixed_types_5", |b| {
        b.iter(|| {
            let types = vec![
                TAtomic::Null,
                TAtomic::Scalar(TScalar::int()),
                TAtomic::Scalar(TScalar::string()),
                TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("Foo")))),
                TAtomic::Array(TArray::List(TList::new(Box::new(get_int())))),
            ];
            std::hint::black_box(combine(types, &codebase, false))
        })
    });
}

criterion_group!(
    combiner_benches,
    bench_combine_simple_scalars,
    bench_combine_integers,
    bench_combine_objects,
    bench_combine_generic_types,
    bench_combine_list_arrays,
    bench_combine_keyed_arrays,
    bench_combine_mixed,
);

criterion_main!(combiner_benches);
