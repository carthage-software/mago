use indexmap::IndexMap;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use mago_algebra::AlgebraThresholds;
use mago_algebra::clause::Clause;
use mago_algebra::disjoin_clauses;
use mago_algebra::find_satisfying_assignments;
use mago_algebra::negate_formula;
use mago_algebra::saturate_clauses;
use mago_atom::Atom;
use mago_atom::AtomSet;
use mago_atom::atom;
use mago_codex::assertion::Assertion;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_span::Span;

fn span(offset: u32) -> Span {
    Span::dummy(offset, offset.saturating_add(1))
}

fn variable(i: usize) -> Atom {
    atom(&format!("$var{i}"))
}

fn assertion_int() -> Assertion {
    Assertion::IsType(TAtomic::Scalar(TScalar::int()))
}

fn assertion_string() -> Assertion {
    Assertion::IsType(TAtomic::Scalar(TScalar::string()))
}

fn assertion_not_null() -> Assertion {
    Assertion::IsNotType(TAtomic::Null)
}

fn assertion_truthy() -> Assertion {
    Assertion::Truthy
}

fn assertion_falsy() -> Assertion {
    Assertion::Falsy
}

/// Builds a clause asserting a single variable has a single assertion.
/// Mirrors the most common analyzer pattern (e.g., `if ($x === null)`).
fn single_assertion_clause(var_index: usize, span_offset: u32, assertion: Assertion) -> Clause {
    let mut type_map: IndexMap<u64, Assertion> = IndexMap::new();
    type_map.insert(assertion.to_hash(), assertion);

    let mut possibilities: IndexMap<Atom, IndexMap<u64, Assertion>> = IndexMap::new();
    possibilities.insert(variable(var_index), type_map);

    let s = span(span_offset);
    Clause::new(possibilities, s, s, None, None, None)
}

/// Builds a disjunctive clause: a single variable with multiple assertions
/// (`$x === int || $x === string`).
fn disjunctive_clause(var_index: usize, span_offset: u32, assertions: Vec<Assertion>) -> Clause {
    let mut type_map: IndexMap<u64, Assertion> = IndexMap::new();
    for a in assertions {
        type_map.insert(a.to_hash(), a);
    }

    let mut possibilities: IndexMap<Atom, IndexMap<u64, Assertion>> = IndexMap::new();
    possibilities.insert(variable(var_index), type_map);

    let s = span(span_offset);
    Clause::new(possibilities, s, s, None, None, None)
}

/// Builds a clause covering multiple variables with one assertion each
/// (matches the shape produced by `&&` over different variables).
fn multi_var_clause(span_offset: u32, vars: &[(usize, Assertion)]) -> Clause {
    let mut possibilities: IndexMap<Atom, IndexMap<u64, Assertion>> = IndexMap::new();
    for (var_index, assertion) in vars {
        let mut type_map: IndexMap<u64, Assertion> = IndexMap::new();
        type_map.insert(assertion.to_hash(), assertion.clone());
        possibilities.insert(variable(*var_index), type_map);
    }

    let s = span(span_offset);
    Clause::new(possibilities, s, s, None, None, None)
}

/// Generates a small CNF formula (~5 clauses, distinct vars). Models a typical
/// `if ($a !== null && $b !== null && $c)` shape.
fn small_formula() -> Vec<Clause> {
    vec![
        single_assertion_clause(0, 0, assertion_not_null()),
        single_assertion_clause(1, 1, assertion_not_null()),
        single_assertion_clause(2, 2, assertion_truthy()),
        single_assertion_clause(3, 3, assertion_int()),
        single_assertion_clause(4, 4, assertion_string()),
    ]
}

/// Generates a medium CNF formula (~15 clauses with overlapping variables).
/// Models a moderately complex `match`/nested-`if`.
fn medium_formula() -> Vec<Clause> {
    let mut clauses = Vec::new();
    for i in 0..15 {
        let var_index = i % 5;
        let assertion = match i % 5 {
            0 => assertion_not_null(),
            1 => assertion_truthy(),
            2 => assertion_int(),
            3 => assertion_string(),
            _ => assertion_falsy(),
        };

        clauses.push(single_assertion_clause(var_index, i as u32, assertion));
    }

    clauses
}

/// Generates a pathological CNF formula (~60 clauses, heavy overlap).
/// Stresses the saturation and negation paths near the threshold cutoffs.
fn pathological_formula() -> Vec<Clause> {
    let mut clauses = Vec::new();
    for i in 0..60 {
        let var_index = i % 8;
        let assertions: Vec<Assertion> = match i % 4 {
            0 => vec![assertion_int(), assertion_string()],
            1 => vec![assertion_not_null()],
            2 => vec![assertion_truthy(), assertion_falsy()],
            _ => vec![assertion_int()],
        };

        if let [single] = assertions.as_slice() {
            clauses.push(single_assertion_clause(var_index, i as u32, single.clone()));
        } else {
            clauses.push(disjunctive_clause(var_index, i as u32, assertions));
        }
    }

    clauses
}

/// Generates a multi-variable formula stressing `find_satisfying_assignments`.
fn multi_var_formula() -> Vec<Clause> {
    let mut clauses = Vec::new();
    for i in 0..20 {
        let span_offset = i as u32;
        clauses.push(multi_var_clause(
            span_offset,
            &[(i % 6, assertion_not_null()), ((i + 1) % 6, assertion_truthy()), ((i + 2) % 6, assertion_int())],
        ));
    }

    clauses
}

fn bench_saturate(c: &mut Criterion) {
    let thresholds = AlgebraThresholds::default();

    let small = small_formula();
    c.bench_function("saturate/small", |b| {
        b.iter(|| {
            let result = saturate_clauses(small.iter(), &thresholds);
            std::hint::black_box(result)
        });
    });

    let medium = medium_formula();
    c.bench_function("saturate/medium", |b| {
        b.iter(|| {
            let result = saturate_clauses(medium.iter(), &thresholds);
            std::hint::black_box(result)
        });
    });

    let pathological = pathological_formula();
    c.bench_function("saturate/pathological", |b| {
        b.iter(|| {
            let result = saturate_clauses(pathological.iter(), &thresholds);
            std::hint::black_box(result)
        });
    });
}

fn bench_negate(c: &mut Criterion) {
    let thresholds = AlgebraThresholds::default();

    let small = small_formula();
    c.bench_function("negate/small", |b| {
        b.iter(|| {
            let result = negate_formula(small.clone(), &thresholds);
            std::hint::black_box(result)
        });
    });

    let medium = medium_formula();
    c.bench_function("negate/medium", |b| {
        b.iter(|| {
            let result = negate_formula(medium.clone(), &thresholds);
            std::hint::black_box(result)
        });
    });

    let pathological = pathological_formula();
    c.bench_function("negate/pathological", |b| {
        b.iter(|| {
            let result = negate_formula(pathological.clone(), &thresholds);
            std::hint::black_box(result)
        });
    });
}

fn bench_disjoin(c: &mut Criterion) {
    let thresholds = AlgebraThresholds::default();
    let conditional_id = span(1000);

    let small_left = small_formula();
    let small_right = small_formula();
    c.bench_function("disjoin/small", |b| {
        b.iter(|| {
            let result = disjoin_clauses(small_left.clone(), small_right.clone(), conditional_id, &thresholds);
            std::hint::black_box(result)
        });
    });

    let medium_left = medium_formula();
    let medium_right = medium_formula();
    c.bench_function("disjoin/medium", |b| {
        b.iter(|| {
            let result = disjoin_clauses(medium_left.clone(), medium_right.clone(), conditional_id, &thresholds);
            std::hint::black_box(result)
        });
    });

    let pathological_left = pathological_formula();
    let pathological_right = small_formula();
    c.bench_function("disjoin/pathological_x_small", |b| {
        b.iter(|| {
            let result =
                disjoin_clauses(pathological_left.clone(), pathological_right.clone(), conditional_id, &thresholds);
            std::hint::black_box(result)
        });
    });
}

fn bench_find_satisfying_assignments(c: &mut Criterion) {
    let small = small_formula();
    c.bench_function("find_satisfying/small", |b| {
        b.iter(|| {
            let mut referenced = AtomSet::default();
            let result = find_satisfying_assignments(&small, None, &mut referenced);
            std::hint::black_box(result)
        });
    });

    let medium = medium_formula();
    c.bench_function("find_satisfying/medium", |b| {
        b.iter(|| {
            let mut referenced = AtomSet::default();
            let result = find_satisfying_assignments(&medium, None, &mut referenced);
            std::hint::black_box(result)
        });
    });

    let multi = multi_var_formula();
    c.bench_function("find_satisfying/multi_var", |b| {
        b.iter(|| {
            let mut referenced = AtomSet::default();
            let result = find_satisfying_assignments(&multi, None, &mut referenced);
            std::hint::black_box(result)
        });
    });
}

fn bench_pipelines(c: &mut Criterion) {
    let thresholds = AlgebraThresholds::default();
    let conditional_id = span(2000);

    let formula = medium_formula();

    c.bench_function("pipeline/saturate_then_negate", |b| {
        b.iter(|| {
            let saturated = saturate_clauses(formula.iter(), &thresholds);
            let negated = negate_formula(saturated, &thresholds);
            std::hint::black_box(negated)
        });
    });

    let left = medium_formula();
    let right = small_formula();
    c.bench_function("pipeline/disjoin_then_saturate", |b| {
        b.iter(|| {
            let disjoined = disjoin_clauses(left.clone(), right.clone(), conditional_id, &thresholds);
            let saturated = saturate_clauses(disjoined.iter(), &thresholds);
            std::hint::black_box(saturated)
        });
    });
}

criterion_group!(
    benches,
    bench_saturate,
    bench_negate,
    bench_disjoin,
    bench_find_satisfying_assignments,
    bench_pipelines
);
criterion_main!(benches);
