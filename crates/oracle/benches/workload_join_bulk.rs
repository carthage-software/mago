//! Workload: bulk union construction.
//!
//! Models the union-construction phase of an analyzer: thousands of
//! [`join::compute`] calls with varying-width input slices. Stresses
//! the canonicalisation pass (sort + dedup + consing), subtype-driven
//! absorption, the per-family payload merges (range merging,
//! string-axis collapse, scalar synthesis, list / keyed-array
//! element-type unions), and the literal-collapse threshold counting.

use core::hint::black_box;
use core::time::Duration;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use mago_allocator::LocalArena;
use mago_oracle::ty::Atom;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::join;

mod common;

use common::Rng;
use common::TypePool;

/// Tuned so one iteration runs ~500ms.
const JOINS_PER_ITER: usize = 500_000;
const SEED: u64 = 0xF00D_BABE;

fn workload(c: &mut Criterion) {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let pool = TypePool::new(SEED, &mut builder);
    let mut rng = Rng::new(SEED);
    let mut inputs: Vec<Vec<Atom<'_>>> = Vec::with_capacity(JOINS_PER_ITER);
    for _ in 0..JOINS_PER_ITER {
        let count = 1 + (rng.next_u32() % 8) as usize;
        let mut atoms: Vec<Atom<'_>> = Vec::with_capacity(count * 2);
        for _ in 0..count {
            let ty = rng.pick_from(&pool.weighted);
            atoms.extend_from_slice(ty.atoms);
        }

        inputs.push(atoms);
    }

    c.bench_function("workload::join_bulk", |b| {
        b.iter(|| {
            let mut accumulated: usize = 0;
            for input in &inputs {
                let result = join::compute(black_box(input.as_slice()), &mut builder);
                accumulated = accumulated.wrapping_add(result.len());
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
