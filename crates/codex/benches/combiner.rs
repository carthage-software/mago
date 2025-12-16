use std::collections::BTreeMap;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use mago_atom::ascii_lowercase_atom;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::combiner::combine;
use mago_codex::ttype::get_int;
use mago_codex::ttype::get_string;
use mago_codex::ttype::union::TUnion;

/// Core benchmarks for combiner performance
fn bench_combiner(c: &mut Criterion) {
    let codebase = CodebaseMetadata::new();

    // Basic: combine few scalar types
    c.bench_function("scalars_4", |b| {
        b.iter(|| {
            let types = vec![
                TAtomic::Scalar(TScalar::int()),
                TAtomic::Scalar(TScalar::string()),
                TAtomic::Scalar(TScalar::float()),
                TAtomic::Scalar(TScalar::bool()),
            ];
            std::hint::black_box(combine(types, &codebase, false))
        });
    });

    // Tests Vec-based integer collection (Phase 2 optimization)
    c.bench_function("literal_ints_50", |b| {
        b.iter(|| {
            let types: Vec<TAtomic> =
                (0..50).map(|i| TAtomic::Scalar(TScalar::Integer(TInteger::literal(i)))).collect();
            std::hint::black_box(combine(types, &codebase, false))
        });
    });

    // Tests Vec-based string collection (Phase 2 optimization)
    c.bench_function("literal_strings_50", |b| {
        b.iter(|| {
            let types: Vec<TAtomic> = (0..50)
                .map(|i| TAtomic::Scalar(TScalar::literal_string(ascii_lowercase_atom(&format!("s{i}")))))
                .collect();
            std::hint::black_box(combine(types, &codebase, false))
        });
    });

    // Tests sealed array deferred processing (Phase 3 optimization)
    c.bench_function("sealed_lists_20", |b| {
        b.iter(|| {
            let types: Vec<TAtomic> = (0..20)
                .map(|_| {
                    TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([
                        (0, (false, get_int())),
                        (1, (false, get_string())),
                    ]))))
                })
                .collect();
            std::hint::black_box(combine(types, &codebase, false))
        });
    });

    // Tests object combining
    c.bench_function("named_objects_50", |b| {
        b.iter(|| {
            let types: Vec<TAtomic> = (0..50)
                .map(|i| TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom(&format!("C{i}"))))))
                .collect();
            std::hint::black_box(combine(types, &codebase, false))
        });
    });

    // Tests generic object key generation (Phase 4 target)
    c.bench_function("generic_objects_20", |b| {
        b.iter(|| {
            let types: Vec<TAtomic> = (0..20)
                .map(|i| {
                    TAtomic::Object(TObject::Named(TNamedObject::new_with_type_parameters(
                        ascii_lowercase_atom("Container"),
                        Some(vec![TUnion::from_atomic(TAtomic::Scalar(TScalar::Integer(TInteger::literal(
                            i64::from(i),
                        ))))]),
                    )))
                })
                .collect();
            std::hint::black_box(combine(types, &codebase, false))
        });
    });

    // Mixed types benchmark
    c.bench_function("mixed_40", |b| {
        b.iter(|| {
            let mut types: Vec<TAtomic> = Vec::with_capacity(40);
            for i in 0..10 {
                types.push(TAtomic::Scalar(TScalar::Integer(TInteger::literal(i))));
            }
            for i in 0..10 {
                types.push(TAtomic::Scalar(TScalar::literal_string(ascii_lowercase_atom(&format!("s{i}")))));
            }
            for i in 0..10 {
                types.push(TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom(&format!("C{i}"))))));
            }
            for _ in 0..10 {
                types.push(TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([(
                    0,
                    (false, get_int()),
                )])))));
            }
            std::hint::black_box(combine(types, &codebase, false))
        });
    });

    // HIGH-CARDINALITY BENCHMARK: 10,000 mixed types (real-world worst case)
    c.bench_function("mixed_10k", |b| {
        b.iter(|| {
            let mut types: Vec<TAtomic> = Vec::with_capacity(10_000);
            // 2500 literal integers
            for i in 0..2500 {
                types.push(TAtomic::Scalar(TScalar::Integer(TInteger::literal(i))));
            }
            // 2500 literal strings
            for i in 0..2500 {
                types.push(TAtomic::Scalar(TScalar::literal_string(ascii_lowercase_atom(&format!("str{i}")))));
            }
            // 2500 named objects
            for i in 0..2500 {
                types.push(TAtomic::Object(TObject::Named(TNamedObject::new(ascii_lowercase_atom(&format!(
                    "Class{i}"
                ))))));
            }
            // 2500 sealed arrays
            for _ in 0..2500 {
                types.push(TAtomic::Array(TArray::List(TList::from_known_elements(BTreeMap::from_iter([(
                    0,
                    (false, get_int()),
                )])))));
            }
            std::hint::black_box(combine(types, &codebase, false))
        });
    });

    // 5000 literal strings only - stress test string handling
    c.bench_function("literal_strings_5k", |b| {
        b.iter(|| {
            let types: Vec<TAtomic> = (0..5000)
                .map(|i| TAtomic::Scalar(TScalar::literal_string(ascii_lowercase_atom(&format!("s{i}")))))
                .collect();
            std::hint::black_box(combine(types, &codebase, false))
        });
    });

    // 5000 literal integers only - stress test integer handling
    c.bench_function("literal_ints_5k", |b| {
        b.iter(|| {
            let types: Vec<TAtomic> =
                (0..5000).map(|i| TAtomic::Scalar(TScalar::Integer(TInteger::literal(i)))).collect();
            std::hint::black_box(combine(types, &codebase, false))
        });
    });
}

criterion_group!(combiner_benches, bench_combiner);
criterion_main!(combiner_benches);
