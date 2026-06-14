//! Workload: narrowing funnel.
//!
//! Models assertion-driven narrowing on a wide union: take a many-atom
//! type and run a long sequence of [`meet::narrow`] and
//! [`subtract::narrow`] calls against it, mimicking what an analyzer
//! does across a chain of `if ($x is Foo) ... elseif (...)` guards.
//!
//! Stresses: the per-input-atom × per-narrowing-atom inner loops in
//! `meet::narrow` and `subtract::narrow`, the scratch-buffer reuse in
//! the subtraction fold, the consing of per-pair result types, and the
//! negated-atom meet paths when narrowings overlap.
//!
//! Measured-region note: suffete consumed each outcome through its
//! flow-meta byte; [`Type`] carries no flow state in this crate, so the
//! loop consumes the atom count of the resulting type instead.

use core::hint::black_box;
use core::time::Duration;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use mago_allocator::LocalArena;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::meet;
use mago_oracle::ty::subtract;
use mago_oracle::world::NullWorld;

mod common;

use common::Rng;
use common::TypePool;

/// Tuned so each meet/subtract bench runs ~500ms per iter.
const NARROWS_PER_ITER: usize = 100_000;
const SEED: u64 = 0xC0DE_FEED;

fn workload(c: &mut Criterion) {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let pool = TypePool::new(SEED, &mut builder);
    let world = NullWorld;
    let options = LatticeOptions::default();

    let mut rng = Rng::new(SEED);
    let mut inputs: Vec<Type<'_>> = Vec::with_capacity(64);
    for _ in 0..32 {
        inputs.push(rng.pick_from(&pool.wide_unions));
    }

    for _ in 0..16 {
        inputs.push(rng.pick_from(&pool.deep_nested));
    }

    for _ in 0..16 {
        inputs.push(rng.pick_from(&pool.small_unions));
    }

    let mut narrowings: Vec<Type<'_>> = Vec::with_capacity(NARROWS_PER_ITER);
    for _ in 0..NARROWS_PER_ITER {
        narrowings.push(rng.pick_from(&pool.weighted));
    }

    let pairs: Vec<(usize, usize)> = (0..NARROWS_PER_ITER).map(|i| (i % inputs.len(), i)).collect();

    c.bench_function("workload::narrow_funnel::meet", |b| {
        b.iter(|| {
            let mut report = LatticeReport::new();
            let mut accumulated: usize = 0;
            for &(input_index, narrowing_index) in &pairs {
                let outcome = meet::narrow(
                    black_box(inputs[input_index]),
                    black_box(narrowings[narrowing_index]),
                    &world,
                    options,
                    &mut report,
                    &mut builder,
                );

                accumulated = accumulated.wrapping_add(outcome.into_type().atoms.len());
            }

            black_box(accumulated)
        });
    });

    c.bench_function("workload::narrow_funnel::subtract", |b| {
        b.iter(|| {
            let mut report = LatticeReport::new();
            let mut accumulated: usize = 0;
            for &(input_index, narrowing_index) in &pairs {
                let outcome = subtract::narrow(
                    black_box(inputs[input_index]),
                    black_box(narrowings[narrowing_index]),
                    &world,
                    options,
                    &mut report,
                    &mut builder,
                );

                accumulated = accumulated.wrapping_add(outcome.into_type().atoms.len());
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
