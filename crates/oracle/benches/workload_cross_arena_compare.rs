//! Workload: cross-arena comparison.
//!
//! Models the two-arena layering the analyzer uses: a long-lived "world"
//! arena holds signature types, a per-file arena holds inference results
//! that embed world atoms covariantly. Every iteration runs refines across
//! the boundary - the path where same-builder pointer identity never fires
//! and the lattice falls back to structural comparison throughout.

use core::hint::black_box;
use core::time::Duration;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use mago_allocator::LocalArena;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::lattice::refines;
use mago_oracle::world::NullWorld;

mod common;

use common::Rng;
use common::TypePool;
use common::ut;

const PAIRS_PER_ITER: usize = 200_000;
const SEED: u64 = 0xC0A5_70AE;

fn workload(c: &mut Criterion) {
    let world_arena = LocalArena::new();
    let world_scratch = LocalArena::new();
    let mut world_builder = TypeBuilder::new(&world_arena, &world_scratch);
    let world_pool = TypePool::new(SEED, &mut world_builder);

    let file_arena = LocalArena::new();
    let file_scratch = LocalArena::new();
    let mut file_builder = TypeBuilder::new(&file_arena, &file_scratch);
    let mut rng = Rng::new(SEED ^ 0xFFFF);

    let mut file_types: Vec<Type<'_>> = Vec::with_capacity(world_pool.weighted.len());
    for world_type in &world_pool.weighted {
        let extra = rng.pick_from(&world_pool.weighted);
        let local_literal = ut(&mut file_builder, Atom::int_literal(i64::from(rng.next_u32() % 100)));
        let mut atoms = world_type.atoms.to_vec();
        atoms.extend_from_slice(extra.atoms);
        atoms.extend_from_slice(local_literal.atoms);
        file_types.push(file_builder.union_of(&atoms));
    }

    let world = NullWorld;
    let options = LatticeOptions::default();

    let mut group = c.benchmark_group("workload");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("cross_arena_compare", |bencher| {
        bencher.iter(|| {
            let mut rng = Rng::new(SEED);
            let mut verdicts = 0usize;
            for _ in 0..PAIRS_PER_ITER {
                let file_type = rng.pick_from(&file_types);
                let world_type = rng.pick_from(&world_pool.weighted);
                let mut report = LatticeReport::new();
                if refines(file_type, world_type, &world, options, &mut report, &mut file_builder) {
                    verdicts = verdicts.wrapping_add(1);
                }
            }

            black_box(verdicts)
        });
    });

    group.finish();
}

criterion_group!(benches, workload);
criterion_main!(benches);
