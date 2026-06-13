//! Workload: lattice storm.
//!
//! Models a flow-typing pass: a large pool of types compared pairwise
//! via every public lattice predicate ([`refines`], [`generalizes`],
//! [`overlaps`]). Each iteration runs `PAIRS_PER_ITER` random pairings,
//! one call to each predicate per pair, exercising the pairwise fold
//! inside `refines`/`overlaps`, the per-family union-cover rules, and
//! the consing tables for the intermediate types the rules construct.

use core::hint::black_box;
use core::time::Duration;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use mago_allocator::LocalArena;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::lattice::generalizes;
use mago_oracle::ty::lattice::overlaps;
use mago_oracle::ty::lattice::refines;
use mago_oracle::world::NullWorld;

mod common;

use common::Rng;
use common::TypePool;

/// Tuned so one iteration runs ~500ms (each pair fans out to three
/// `refines/generalizes/overlaps` calls).
const PAIRS_PER_ITER: usize = 2_000_000;
const SEED: u64 = 0xBEEF_BABE;

fn workload(c: &mut Criterion) {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let pool = TypePool::new(SEED, &mut builder);
    let world = NullWorld;
    let options = LatticeOptions::default();

    let mut rng = Rng::new(SEED);
    let mut pairs: Vec<(usize, usize)> = Vec::with_capacity(PAIRS_PER_ITER);
    for _ in 0..PAIRS_PER_ITER {
        pairs.push((rng.pick(pool.weighted.len()), rng.pick(pool.weighted.len())));
    }

    c.bench_function("workload::lattice_storm", |b| {
        b.iter(|| {
            let mut report = LatticeReport::new();
            let mut hits: u32 = 0;
            for &(left_index, right_index) in &pairs {
                let left = pool.weighted[left_index];
                let right = pool.weighted[right_index];
                if refines(black_box(left), black_box(right), &world, options, &mut report, &mut builder) {
                    hits = hits.wrapping_add(1);
                }
                if generalizes(black_box(left), black_box(right), &world, options, &mut report, &mut builder) {
                    hits = hits.wrapping_add(1);
                }
                if overlaps(black_box(left), black_box(right), &world, options, &mut report, &mut builder) {
                    hits = hits.wrapping_add(1);
                }
            }

            black_box(hits)
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10).measurement_time(Duration::from_secs(8));
    targets = workload
);

criterion_main!(benches);
