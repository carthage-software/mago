mod common;

use common::*;

use mago_oracle::ty::Type;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::lattice::is_uninhabited;
use mago_oracle::ty::meet;
use mago_oracle::ty::subtract;
use mago_oracle::ty::well_known;
use mago_oracle::world::World;

fn meet_of<'arena, W>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>, world: &W) -> Type<'arena>
where
    W: World<'arena>,
{
    let mut report = LatticeReport::new();
    meet::compute(a, b, world, LatticeOptions::default(), &mut report, &mut f.builder)
}

fn subtract_of<'arena, W>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>, world: &W) -> Type<'arena>
where
    W: World<'arena>,
{
    let mut report = LatticeReport::new();
    subtract::compute(a, b, world, LatticeOptions::default(), &mut report, &mut f.builder)
}

fn sealed_throwable_world<'arena>(f: &mut Fixture<'_, 'arena>) -> MockWorld<'arena> {
    let mut world = MockWorld::new();
    world
        .add_edge("Error", "Throwable")
        .add_edge("Exception", "Throwable")
        .add_edge("RuntimeException", "Exception")
        .with_sealed(&mut f.builder, "Throwable", &["Error", "Exception"]);
    world
}

#[test]
fn subtract_throwable_by_exception_canonicalises_to_error() {
    fixture(|f| {
        let world = sealed_throwable_world(f);
        let throwable = f.t_named("Throwable");
        let exception = f.t_named("Exception");
        let error = f.t_named("Error");

        let throwable_type = f.u(throwable);
        let exception_type = f.u(exception);
        let result = subtract_of(f, throwable_type, exception_type, &world);
        let expected = f.u(error);
        assert_eq!(result, expected);
    });
}

#[test]
fn subtract_throwable_by_error_canonicalises_to_exception() {
    fixture(|f| {
        let world = sealed_throwable_world(f);
        let throwable = f.t_named("Throwable");
        let error = f.t_named("Error");
        let exception = f.t_named("Exception");

        let throwable_type = f.u(throwable);
        let error_type = f.u(error);
        let result = subtract_of(f, throwable_type, error_type, &world);
        let expected = f.u(exception);
        assert_eq!(result, expected);
    });
}

#[test]
fn meet_throwable_with_negated_exception_canonicalises_to_error() {
    fixture(|f| {
        let world = sealed_throwable_world(f);
        let throwable = f.t_named("Throwable");
        let exception = f.t_named("Exception");
        let error = f.t_named("Error");

        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_without_exception = f.builder.intersected(throwable, &[negated_exception]);
        let input = f.u(throwable_without_exception);
        let error_type = f.u(error);
        let result = meet_of(f, input, error_type, &world);
        assert_eq!(result, error_type);
    });
}

#[test]
fn subtract_throwable_by_error_or_exception_is_never() {
    fixture(|f| {
        let world = sealed_throwable_world(f);
        let throwable = f.t_named("Throwable");
        let error = f.t_named("Error");
        let exception = f.t_named("Exception");

        let union = f.u_many(vec![error, exception]);
        let throwable_type = f.u(throwable);
        assert_eq!(subtract_of(f, throwable_type, union, &world), well_known::TYPE_NEVER);
    });
}

#[test]
fn traversable_minus_iterator_minus_iterator_aggregate_is_never() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.add_edge("Iterator", "Traversable").add_edge("IteratorAggregate", "Traversable").with_sealed(
            &mut f.builder,
            "Traversable",
            &["Iterator", "IteratorAggregate"],
        );

        let traversable = f.t_named("Traversable");
        let iterator = f.t_named("Iterator");
        let iterator_aggregate = f.t_named("IteratorAggregate");

        let traversable_type = f.u(traversable);
        let iterator_type = f.u(iterator);
        let after_iterator = subtract_of(f, traversable_type, iterator_type, &world);
        let iterator_aggregate_type = f.u(iterator_aggregate);
        let final_result = subtract_of(f, after_iterator, iterator_aggregate_type, &world);
        assert_eq!(final_result, well_known::TYPE_NEVER);
    });
}

#[test]
fn partial_cover_does_not_collapse_when_residual_has_multiple() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.add_edge("A", "Foo").add_edge("B", "Foo").add_edge("C", "Foo").with_sealed(
            &mut f.builder,
            "Foo",
            &["A", "B", "C"],
        );

        let foo = f.t_named("Foo");
        let inheritor = f.t_named("A");
        let inheritor_type = f.u(inheritor);
        let negated_inheritor = f.builder.negated(inheritor_type);
        let foo_without_inheritor = f.builder.intersected(foo, &[negated_inheritor]);

        assert!(!is_uninhabited(foo_without_inheritor, &world, &mut f.builder));
    });
}

#[test]
fn cycle_with_direct_coverage_collapses() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.add_edge("B", "A").add_edge("A", "B").with_sealed(&mut f.builder, "A", &["B"]).with_sealed(
            &mut f.builder,
            "B",
            &["A"],
        );

        let head = f.t_named("A");
        let inheritor = f.t_named("B");
        let inheritor_type = f.u(inheritor);
        let negated_inheritor = f.builder.negated(inheritor_type);
        let covered = f.builder.intersected(head, &[negated_inheritor]);

        assert!(is_uninhabited(covered, &world, &mut f.builder));
    });
}

#[test]
fn cycle_without_direct_coverage_terminates() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.add_edge("B", "A").add_edge("A", "B").with_sealed(&mut f.builder, "A", &["B"]).with_sealed(
            &mut f.builder,
            "B",
            &["A"],
        );

        let head = f.t_named("A");
        let unrelated = f.t_named("Unrelated");
        let unrelated_type = f.u(unrelated);
        let negated_unrelated = f.builder.negated(unrelated_type);
        let residual = f.builder.intersected(head, &[negated_unrelated]);

        assert!(!is_uninhabited(residual, &world, &mut f.builder));
    });
}

#[test]
fn depth_cap_does_not_overflow() {
    fixture(|f| {
        let mut world = MockWorld::new();
        for index in 0..20 {
            let parent = format!("S{index}");
            let child = format!("S{}", index + 1);
            world.add_edge(&child, &parent);
        }
        world.with_sealed(&mut f.builder, "S0", &["S1"]);

        let root = f.t_named("S0");
        let deepest = f.t_named("S21");
        let deepest_type = f.u(deepest);
        let negated_deepest = f.builder.negated(deepest_type);
        let residual = f.builder.intersected(root, &[negated_deepest]);

        assert!(!is_uninhabited(residual, &world, &mut f.builder));
    });
}

#[test]
fn final_class_with_negated_self_is_never() {
    fixture(|f| {
        let final_class = f.t_named("Final");
        let final_type = f.u(final_class);
        let negated_final = f.builder.negated(final_type);
        let contradiction = f.builder.intersected(final_class, &[negated_final]);
        assert_eq!(contradiction, well_known::NEVER);
    });
}

#[test]
fn non_class_head_skips_sealed_logic() {
    fixture(|f| {
        let world = sealed_throwable_world(f);
        let exception = f.t_named("Exception");
        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let int_without_exception = f.builder.intersected(well_known::INT, &[negated_exception]);
        assert!(!is_uninhabited(int_without_exception, &world, &mut f.builder));
    });
}

#[test]
fn null_world_returns_no_sealed_inheritors() {
    fixture(|f| {
        let world = empty_world();
        let throwable = f.t_named("Throwable");
        let exception = f.t_named("Exception");
        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_minus_exception = f.builder.intersected(throwable, &[negated_exception]);
        assert!(!is_uninhabited(throwable_minus_exception, &world, &mut f.builder));
    });
}
