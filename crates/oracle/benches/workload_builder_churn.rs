//! Workload: steady-state builder churn.
//!
//! Models the type-construction phase of an analyzer: a long sequence of
//! [`TypeBuilder::union_of`] calls drawn from a realistic mix of
//! singletons, small unions, wide unions, and deeply nested shapes. This
//! is the primary stress for the builder's hash-consing tables: the
//! canonical-slice lookup on the fast path and the sort-buffer + insert
//! slow path on misses.
//!
//! Each iteration runs `OPS_PER_ITER` distinct construction calls. Most
//! hit the consing tables (the pool pre-warmed them, exactly as the
//! original suffete bench pre-warmed its process-global interner); every
//! 32nd op builds a fresh literal-int atom, which is an inline payload
//! with no arena traffic, mirroring the fresh-handle pressure of the
//! original.
//!
//! Measured-region note: the arenas and the [`TypeBuilder`] are created
//! once in setup - the suffete bench measured `intern_type` calls against
//! an already-initialised global interner, so the construction calls stay
//! inside the measured loop while arena/builder creation stays outside.

use core::hint::black_box;
use core::time::Duration;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use mago_allocator::LocalArena;
use mago_oracle::ty::Atom;
use mago_oracle::ty::TypeBuilder;

mod common;

use common::Rng;
use common::TypePool;

/// Tuned so one iteration runs ~500ms on a modern desktop core.
/// Codspeed measures cycles per iteration; the target keeps each
/// measurement well above the simulator's noise floor.
const OPS_PER_ITER: usize = 8_000_000;
const SEED: u64 = 0xCAFE_F00D;

fn workload(c: &mut Criterion) {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let pool = TypePool::new(SEED, &mut builder);
    let mut rng = Rng::new(SEED);
    let mut atom_buffers: Vec<Vec<Atom<'_>>> = Vec::with_capacity(512);
    for _ in 0..512 {
        let ty = rng.pick_from(&pool.weighted);
        atom_buffers.push(ty.atoms.to_vec());
    }

    let mut fresh_seed = Rng::new(SEED ^ 0xFFFF);

    c.bench_function("workload::builder_churn", |b| {
        b.iter(|| {
            let mut index: usize = 0;
            for _ in 0..OPS_PER_ITER {
                if index.is_multiple_of(32) {
                    let value = fresh_seed.next_u64() as i64;
                    let _ = black_box(Atom::int_literal(value));
                } else {
                    let buffer = &atom_buffers[index % atom_buffers.len()];
                    let _ = black_box(builder.union_of(buffer));
                }

                index = index.wrapping_add(1);
            }
        });
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10).measurement_time(Duration::from_secs(8));
    targets = workload
);

criterion_main!(benches);
