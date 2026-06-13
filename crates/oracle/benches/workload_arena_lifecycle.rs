//! Workload: arena lifecycle.
//!
//! Models the LSP rebuild loop the arena design exists for: every iteration
//! stands up a fresh output + scratch arena pair, seeds a [`TypeBuilder`],
//! builds a per-file batch of types (literals, unions, containers), runs a
//! handful of joins over them, and tears the whole thing down. This is the
//! cost suffete's process-global interner could never reclaim; here it is
//! the per-file steady state.

use core::hint::black_box;
use core::time::Duration;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use mago_allocator::LocalArena;
use mago_oracle::ty::Atom;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::join;
use mago_oracle::ty::well_known;

mod common;

use common::Rng;
use common::TypePool;
use common::ut;

const TYPES_PER_FILE: usize = 400;
const JOINS_PER_FILE: usize = 50;
const SEED: u64 = 0xA5EA_11FE;

fn workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("workload");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("arena_lifecycle", |bencher| {
        bencher.iter(|| {
            let arena = LocalArena::new();
            let scratch = LocalArena::new();
            let mut builder = TypeBuilder::new(&arena, &scratch);
            let pool = TypePool::new(SEED, &mut builder);
            let mut rng = Rng::new(SEED);

            let mut built = Vec::with_capacity(TYPES_PER_FILE);
            for index in 0..TYPES_PER_FILE {
                let literal = i64::from(rng.next_u32() % 500);
                let atom = match index % 5 {
                    0 => Atom::int_literal(literal),
                    1 => builder.string_literal(literal.to_string().as_bytes()),
                    2 => builder.int_range(Some(0), Some(literal.abs())),
                    3 => {
                        let element = builder.union_of(&[Atom::int_literal(literal)]);
                        builder.list_of(element, false)
                    }
                    _ => well_known::STRING,
                };
                let single = ut(&mut builder, atom);
                let mut atoms = single.atoms.to_vec();
                atoms.push(well_known::NULL);
                built.push(builder.union_of(&atoms));
            }

            let mut accumulator = 0usize;
            for _ in 0..JOINS_PER_FILE {
                let left = rng.pick_from(&pool.weighted);
                let right = rng.pick_from(&built);
                let mut atoms: Vec<Atom<'_>> = left.atoms.to_vec();
                atoms.extend_from_slice(right.atoms);
                let joined = join::compute(&atoms, &mut builder);
                accumulator = accumulator.wrapping_add(joined.len());
            }

            black_box(accumulator)
        });
    });

    group.finish();
}

criterion_group!(benches, workload);
criterion_main!(benches);
