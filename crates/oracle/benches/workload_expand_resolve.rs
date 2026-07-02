//! Workload: alias / reference expansion.
//!
//! Models the unresolved-type-resolution phase of an analyzer: a forest
//! of types is expanded under a [`SymbolTable`](mago_oracle::symbol::SymbolTable)
//! via [`expand::expand_with`]. Stresses the structural flat-map walk in
//! `expand` and the per-atom resolution dispatch.
//!
//! Uses the default [`ExpansionContext`] plus a "full" variant that
//! enables every toggleable expansion stage.
//!
//! Measured-region note: suffete consumed each result through its
//! flow-meta byte; `Type` carries no flow state in this crate, so the
//! loop consumes the atom count of the resulting type instead.

use core::hint::black_box;
use core::time::Duration;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use mago_allocator::LocalArena;
use mago_oracle::symbol::SymbolTable;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::expand;
use mago_oracle::ty::expand::ExpansionContext;

mod common;

use common::Rng;
use common::TypePool;

const OPS_PER_ITER: usize = 6_000_000;
const SEED: u64 = 0xDEAD_BEEF;

fn workload(c: &mut Criterion) {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let pool = TypePool::new(SEED, &mut builder);
    let symbols = SymbolTable::new_in(&arena);
    let context_default = ExpansionContext::default();
    let context_full = ExpansionContext::default()
        .with_evaluate_conditional(true)
        .with_fill_template_defaults(true)
        .with_substitute_template_constraints(true)
        .with_function_is_final(true);

    let mut rng = Rng::new(SEED);
    let mut targets: Vec<Type<'_>> = Vec::with_capacity(OPS_PER_ITER);
    for _ in 0..OPS_PER_ITER {
        targets.push(rng.pick_from(&pool.weighted));
    }

    c.bench_function("workload::expand_resolve::default", |b| {
        b.iter(|| {
            let mut accumulated: usize = 0;
            for ty in &targets {
                let result = expand::expand_with(black_box(*ty), &symbols, &context_default, &mut builder);
                accumulated = accumulated.wrapping_add(result.atoms.len());
            }
            black_box(accumulated)
        });
    });

    c.bench_function("workload::expand_resolve::full", |b| {
        b.iter(|| {
            let mut accumulated: usize = 0;
            for ty in &targets {
                let result = expand::expand_with(black_box(*ty), &symbols, &context_full, &mut builder);
                accumulated = accumulated.wrapping_add(result.atoms.len());
            }
            black_box(accumulated)
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10).measurement_time(Duration::from_secs(8));
    targets = workload
);
criterion_main!(benches);
