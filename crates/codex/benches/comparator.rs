use std::collections::BTreeMap;
use std::sync::Arc;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use mago_codex::metadata::CodebaseMetadata;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::enum_case::EnumCaseMetadata;
use mago_codex::metadata::flags::MetadataFlags;
use mago_codex::misc::GenericParent;
use mago_codex::symbol::SymbolKind;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::reference::TReference;
use mago_codex::ttype::atomic::reference::TReferenceMemberSelector;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::expand_union;
use mago_codex::ttype::get_int;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_string;
use mago_codex::ttype::union::TUnion;
use mago_span::Span;
use mago_word::ascii_lowercase_word;
use mago_word::word;

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

    c.bench_function("is_contained_by_generic_constraint_union", |b| {
        let input = TUnion::from_atomic(TAtomic::GenericParameter(TGenericParameter::new(
            ascii_lowercase_word(b"T"),
            Arc::new(TUnion::from_vec(vec![
                TAtomic::Scalar(TScalar::int()),
                TAtomic::Scalar(TScalar::string()),
                TAtomic::Scalar(TScalar::float()),
                TAtomic::Scalar(TScalar::bool()),
                TAtomic::Null,
            ])),
            GenericParent::FunctionLike((ascii_lowercase_word(b"bench"), ascii_lowercase_word(b"compare"))),
        )));

        let container = TUnion::from_vec(vec![
            TAtomic::Scalar(TScalar::int()),
            TAtomic::Scalar(TScalar::string()),
            TAtomic::Scalar(TScalar::float()),
            TAtomic::Scalar(TScalar::bool()),
            TAtomic::Null,
            TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_word(b"Foo")))),
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
            TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_word(b"Foo")))),
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
        let input = TUnion::from_atomic(TAtomic::Array(TArray::List(TList::new(Arc::new(get_int())))));
        let container = TUnion::from_atomic(TAtomic::Array(TArray::List(TList::new(Arc::new(get_int())))));
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
        let container = TUnion::from_atomic(TAtomic::Array(TArray::List(TList::new(Arc::new(get_int())))));
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
                (ArrayKey::String(ascii_lowercase_word(b"a")), (false, get_int())),
                (ArrayKey::String(ascii_lowercase_word(b"b")), (false, get_string())),
            ]),
        ))));
        let container = TUnion::from_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray::new().with_known_items(
            BTreeMap::from_iter([
                (ArrayKey::String(ascii_lowercase_word(b"a")), (false, get_int())),
                (ArrayKey::String(ascii_lowercase_word(b"b")), (false, get_string())),
                (ArrayKey::String(ascii_lowercase_word(b"c")), (true, get_int())),
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
            TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_word(b"Foo")))));
        let container =
            TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_word(b"Foo")))));
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
            TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_word(b"Foo")))));
        let container =
            TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_word(b"Bar")))));
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
            ascii_lowercase_word(b"Container"),
            Some(vec![get_int()]),
        ))));
        let container = TUnion::from_atomic(TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
            ascii_lowercase_word(b"Container"),
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

fn bench_enum_comparison(c: &mut Criterion) {
    let enum_name = word(b"Suit");
    let hearts = word(b"Hearts");
    let spades = word(b"Spades");
    let mut metadata = ClassLikeMetadata::new(
        ascii_lowercase_word(enum_name.as_bytes()),
        enum_name,
        Span::dummy(0, 0),
        None,
        MetadataFlags::empty(),
    );
    metadata.kind = SymbolKind::Enum;
    metadata
        .enum_cases
        .insert(hearts, EnumCaseMetadata::new(hearts, Span::dummy(0, 0), Span::dummy(0, 0), MetadataFlags::empty()));
    metadata
        .enum_cases
        .insert(spades, EnumCaseMetadata::new(spades, Span::dummy(0, 0), Span::dummy(0, 0), MetadataFlags::empty()));

    let mut codebase = CodebaseMetadata::new();
    codebase.symbols.add_symbol_name(metadata.name, SymbolKind::Enum);
    codebase.class_likes.insert(metadata.name, metadata);

    let enum_type = TUnion::from_atomic(TAtomic::Object(TObject::new_enum(enum_name)));
    let enum_case = TUnion::from_atomic(TAtomic::Object(TObject::new_enum_case(enum_name, hearts)));
    let enum_cases = TUnion::from_vec(vec![
        TAtomic::Object(TObject::new_enum_case(enum_name, hearts)),
        TAtomic::Object(TObject::new_enum_case(enum_name, spades)),
    ]);
    let enum_wildcard =
        TUnion::from_atomic(TAtomic::Reference(TReference::new_member(enum_name, TReferenceMemberSelector::Wildcard)));
    let mut expanded_enum_wildcard = enum_wildcard.clone();
    expand_union(&codebase, &mut expanded_enum_wildcard, &TypeExpansionOptions::default());

    c.bench_function("is_contained_by_enum_same", |b| {
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &enum_type,
                &enum_type,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });

    c.bench_function("is_contained_by_enum_case", |b| {
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &enum_case,
                &enum_type,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });

    c.bench_function("is_contained_by_enum_case_union", |b| {
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &enum_type,
                &enum_cases,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });

    c.bench_function("is_contained_by_enum_wildcard", |b| {
        b.iter(|| {
            let mut result = ComparisonResult::new();
            std::hint::black_box(union_comparator::is_contained_by(
                &codebase,
                &enum_type,
                &expanded_enum_wildcard,
                false,
                false,
                false,
                &mut result,
            ))
        });
    });

    c.bench_function("expand_enum_wildcard", |b| {
        b.iter(|| {
            let mut expanded = enum_wildcard.clone();
            expand_union(&codebase, &mut expanded, &TypeExpansionOptions::default());
            std::hint::black_box(expanded)
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
            TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_word(b"Foo")))),
            TAtomic::Scalar(TScalar::float()),
            TAtomic::Array(TArray::List(TList::new(Arc::new(get_int())))),
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
    bench_enum_comparison,
    bench_can_be_identical,
);

criterion_main!(comparator_benches);
