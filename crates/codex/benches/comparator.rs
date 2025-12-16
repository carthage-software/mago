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
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::get_int;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_string;
use mago_codex::ttype::union::TUnion;

/// Benchmark union comparisons with simple types
fn bench_union_simple_comparison(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();

    c.bench_function("is_contained_by_same_type", |b| {
        let input = get_int();
        let container = get_int();
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &input,
                &container,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });

    c.bench_function("is_contained_by_int_in_mixed", |b| {
        let input = get_int();
        let container = get_mixed();
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &input,
                &container,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });

    c.bench_function("is_contained_by_int_in_string", |b| {
        let input = get_int();
        let container = get_string();
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &input,
                &container,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });
}

/// Benchmark union comparisons with multiple types
fn bench_union_multi_type_comparison(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();

    c.bench_function("is_contained_by_3_type_union", |b| {
        let input =
            TUnion::from_vec(vec![TAtomic::Scalar(TScalar::int()), TAtomic::Scalar(TScalar::string()), TAtomic::Null]);
        let container = TUnion::from_vec(vec![
            TAtomic::Scalar(TScalar::int()),
            TAtomic::Scalar(TScalar::string()),
            TAtomic::Scalar(TScalar::float()),
            TAtomic::Null,
        ]);
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &input,
                &container,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });

    c.bench_function("is_contained_by_5_type_union", |b| {
        let input = TUnion::from_vec(vec![
            TAtomic::Scalar(TScalar::int()),
            TAtomic::Scalar(TScalar::string()),
            TAtomic::Scalar(TScalar::float()),
            TAtomic::Scalar(TScalar::bool()),
            TAtomic::Null,
        ]);
        let container = TUnion::from_vec(vec![
            TAtomic::Scalar(TScalar::int()),
            TAtomic::Scalar(TScalar::string()),
            TAtomic::Scalar(TScalar::float()),
            TAtomic::Scalar(TScalar::bool()),
            TAtomic::Null,
            TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("Foo")))),
        ]);
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &input,
                &container,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });
}

/// Benchmark integer range comparisons
fn bench_integer_comparison(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();

    c.bench_function("is_contained_by_literal_int_in_int", |b| {
        let input = TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::literal(42))));
        let container = get_int();
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &input,
                &container,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });

    c.bench_function("is_contained_by_range_in_range", |b| {
        let input = TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::Range(5, 10))));
        let container = TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::Range(0, 100))));
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &input,
                &container,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });

    c.bench_function("is_contained_by_int_in_union_of_ints", |b| {
        let input = TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::literal(5))));
        let container = TUnion::from_vec(vec![
            TAtomic::Scalar(TScalar::Integer(TInteger::literal(1))),
            TAtomic::Scalar(TScalar::Integer(TInteger::literal(2))),
            TAtomic::Scalar(TScalar::Integer(TInteger::literal(3))),
            TAtomic::Scalar(TScalar::Integer(TInteger::literal(4))),
            TAtomic::Scalar(TScalar::Integer(TInteger::literal(5))),
        ]);
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &input,
                &container,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });
}

/// Benchmark array comparisons
fn bench_array_comparison(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();

    c.bench_function("is_contained_by_list_in_list", |b| {
        let input = TUnion::from_atomic(TAtomic::Array(TArray::List(TList::new(Box::new(get_int())))));
        let container = TUnion::from_atomic(TAtomic::Array(TArray::List(TList::new(Box::new(get_int())))));
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &input,
                &container,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });

    c.bench_function("is_contained_by_tuple_in_list", |b| {
        let input =
            TUnion::from_atomic(TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([
                (0, (false, get_int())),
                (1, (false, get_int())),
                (2, (false, get_int())),
            ])))));
        let container = TUnion::from_atomic(TAtomic::Array(TArray::List(TList::new(Box::new(get_int())))));
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &input,
                &container,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });

    c.bench_function("is_contained_by_keyed_in_keyed", |b| {
        let input = TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray::new().with_known_items(
            BTreeMap::from_iter([
                (ArrayKey::String(ascii_lowercase_atom("a")), (false, get_int())),
                (ArrayKey::String(ascii_lowercase_atom("b")), (false, get_string())),
            ]),
        ))));
        let container = TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray::new().with_known_items(
            BTreeMap::from_iter([
                (ArrayKey::String(ascii_lowercase_atom("a")), (false, get_int())),
                (ArrayKey::String(ascii_lowercase_atom("b")), (false, get_string())),
                (ArrayKey::String(ascii_lowercase_atom("c")), (true, get_int())),
            ]),
        ))));
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &input,
                &container,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });
}

/// Benchmark object comparisons
fn bench_object_comparison(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();

    c.bench_function("is_contained_by_object_same", |b| {
        let input =
            TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("Foo")))));
        let container =
            TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("Foo")))));
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &input,
                &container,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });

    c.bench_function("is_contained_by_object_different", |b| {
        let input =
            TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("Foo")))));
        let container =
            TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("Bar")))));
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &input,
                &container,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });

    c.bench_function("is_contained_by_generic_object", |b| {
        let input = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
            ascii_lowercase_atom("Container"),
            Some(vec![get_int()]),
        ))));
        let container = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
            ascii_lowercase_atom("Container"),
            Some(vec![get_int()]),
        ))));
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &input,
                &container,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });
}

/// Benchmark `can_expression_types_be_identical`
fn bench_can_be_identical(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();

    c.bench_function("can_be_identical_same_type", |b| {
        let type1 = get_int();
        let type2 = get_int();
        b.iter(|| {
            std::hint::black_box(union_comparator::can_expression_types_be_identical(
                &codebase, &type1, &type2, false, false,
            ))
        });
    });

    c.bench_function("can_be_identical_different_types", |b| {
        let type1 = get_int();
        let type2 = get_string();
        b.iter(|| {
            std::hint::black_box(union_comparator::can_expression_types_be_identical(
                &codebase, &type1, &type2, false, false,
            ))
        });
    });

    c.bench_function("can_be_identical_5_type_unions", |b| {
        let type1 = TUnion::from_vec(vec![
            TAtomic::Scalar(TScalar::int()),
            TAtomic::Scalar(TScalar::string()),
            TAtomic::Scalar(TScalar::float()),
            TAtomic::Scalar(TScalar::bool()),
            TAtomic::Null,
        ]);
        let type2 = TUnion::from_vec(vec![
            TAtomic::Scalar(TScalar::string()),
            TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom("Foo")))),
            TAtomic::Scalar(TScalar::float()),
            TAtomic::Array(TArray::List(TList::new(Box::new(get_int())))),
            TAtomic::Null,
        ]);
        b.iter(|| {
            std::hint::black_box(union_comparator::can_expression_types_be_identical(
                &codebase, &type1, &type2, false, false,
            ))
        });
    });
}

criterion_group!(
    comparator_benches,
    bench_union_simple_comparison,
    bench_union_multi_type_comparison,
    bench_integer_comparison,
    bench_array_comparison,
    bench_object_comparison,
    bench_can_be_identical,
);

criterion_main!(comparator_benches);
