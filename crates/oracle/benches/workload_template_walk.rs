//! Workload: template substitution + structural walk.
//!
//! Models the generic-instantiation phase of an analyzer: a forest of
//! template-bearing types is walked end-to-end via [`transform::map`]
//! and [`template::substitute`] to specialise `T -> concrete` across
//! thousands of call sites.
//!
//! Stresses: the post-order walker in `transform`, every payload-bearing
//! arm of the nested-type walk (object arguments, list element type,
//! keyed-array key/value, iterable, callable signature, intersections),
//! and `template::substitute`'s per-atom decision under the flat-map
//! walk.
//!
//! Measured-region note: suffete consumed each result through its
//! flow-meta byte; [`Type`] carries no flow state in this crate, so the
//! loop consumes the atom count of the resulting type instead.

use core::hint::black_box;
use core::time::Duration;

use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use mago_allocator::LocalArena;
use mago_oracle::ty::Atom;
use mago_oracle::ty::AtomKind;
use mago_oracle::ty::Type;
use mago_oracle::ty::TypeBuilder;
use mago_oracle::ty::atom::payload::generic_parameter::DefiningEntity;
use mago_oracle::ty::atom::payload::generic_parameter::GenericParameterAtom;
use mago_oracle::ty::template;
use mago_oracle::ty::transform;
use mago_oracle::ty::well_known;

mod common;

use common::Rng;
use common::TypePool;
use common::ut;

const SUBSTITUTE_OPS: usize = 200_000;
const MAP_OPS: usize = 5_000_000;
const SEED: u64 = 0x1337_DEAD;

fn workload(c: &mut Criterion) {
    let arena = LocalArena::new();
    let scratch = LocalArena::new();
    let mut builder = TypeBuilder::new(&arena, &scratch);

    let pool = TypePool::new(SEED, &mut builder);
    let mut rng = Rng::new(SEED);

    let class_name = builder.intern_class_like_path(b"Foo");
    let parameter_name = builder.intern(b"T");
    let parameter_atom = builder.generic_parameter(GenericParameterAtom {
        name: parameter_name,
        defining_entity: DefiningEntity::ClassLike(class_name),
        constraint: well_known::TYPE_MIXED,
    });
    let template_parameter = ut(&mut builder, parameter_atom);

    let mut templated_tree: Vec<Type<'_>> = Vec::with_capacity(64);
    for _ in 0..64 {
        let depth = 1 + (rng.next_u32() % 3) as usize;
        let mut current = template_parameter;
        for _ in 0..depth {
            let list_atom = builder.list_of(current, false);
            current = ut(&mut builder, list_atom);
        }
        templated_tree.push(current);
    }

    let mut replacements: Vec<Type<'_>> = Vec::with_capacity(256);
    for _ in 0..256 {
        replacements.push(rng.pick_from(&pool.singletons));
    }

    let substitute_work: Vec<(usize, usize)> =
        (0..SUBSTITUTE_OPS).map(|i| (i % templated_tree.len(), i % replacements.len())).collect();
    let map_work: Vec<usize> = (0..MAP_OPS).map(|i| i % templated_tree.len()).collect();

    c.bench_function("workload::template_walk::substitute", |b| {
        b.iter(|| {
            let mut accumulated: usize = 0;
            for &(tree_index, replacement_index) in &substitute_work {
                let target = templated_tree[tree_index];
                let replacement = replacements[replacement_index];
                let result = template::substitute(
                    black_box(target),
                    &|_: &GenericParameterAtom<'_>| Some(replacement),
                    &mut builder,
                );

                accumulated = accumulated.wrapping_add(result.atoms.len());
            }

            black_box(accumulated)
        });
    });

    c.bench_function("workload::template_walk::transform_map", |b| {
        let identity_target = Atom::int_literal(0);
        b.iter(|| {
            let mut accumulated: usize = 0;
            for &tree_index in &map_work {
                let target = templated_tree[tree_index];
                let result = transform::map(
                    black_box(target),
                    |atom| if atom.kind() == AtomKind::Int { identity_target } else { atom },
                    &mut builder,
                );

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
