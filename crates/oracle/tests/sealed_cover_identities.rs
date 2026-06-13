mod common;

use common::*;

use mago_oracle::ty::Type;
use mago_oracle::ty::lattice;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::lattice::is_uninhabited;
use mago_oracle::ty::meet;
use mago_oracle::ty::subtract;
use mago_oracle::ty::well_known;
use mago_oracle::world::World;

fn create_sealed_world<'arena>() -> MockWorld<'arena> {
    let mut world = MockWorld::new();
    world
        .add_edge("Error", "Throwable")
        .add_edge("Exception", "Throwable")
        .add_edge("RuntimeException", "Exception")
        .add_edge("LogicException", "Exception")
        .with_sealed("Throwable", &["Error", "Exception"]);
    world
}

fn refines_of<'arena, W>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>, world: &W) -> bool
where
    W: World<'arena>,
{
    let mut report = LatticeReport::new();
    lattice::refines(a, b, world, LatticeOptions::default(), &mut report, &mut f.builder)
}

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

#[test]
fn throwable_minus_exception_refines_error() {
    fixture(|f| {
        let world = create_sealed_world();
        let throwable = f.t_named("Throwable");
        let exception = f.t_named("Exception");
        let error = f.t_named("Error");

        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_minus_exception = f.builder.intersected(throwable, &[negated_exception]);

        let input = f.u(throwable_minus_exception);
        let container = f.u(error);
        assert!(refines_of(f, input, container, &world));
    });
}

#[test]
fn error_refines_throwable_minus_exception() {
    fixture(|f| {
        let world = create_sealed_world();
        let throwable = f.t_named("Throwable");
        let exception = f.t_named("Exception");
        let error = f.t_named("Error");

        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_minus_exception = f.builder.intersected(throwable, &[negated_exception]);

        let input = f.u(error);
        let container = f.u(throwable_minus_exception);
        assert!(refines_of(f, input, container, &world));
    });
}

#[test]
fn throwable_refines_error_or_exception_union() {
    fixture(|f| {
        let world = create_sealed_world();
        let throwable = f.t_named("Throwable");
        let error = f.t_named("Error");
        let exception = f.t_named("Exception");

        let union = f.u_many(vec![error, exception]);
        let throwable_type = f.u(throwable);
        assert!(refines_of(f, throwable_type, union, &world));
    });
}

#[test]
fn error_or_exception_union_refines_throwable() {
    fixture(|f| {
        let world = create_sealed_world();
        let throwable = f.t_named("Throwable");
        let error = f.t_named("Error");
        let exception = f.t_named("Exception");

        let union = f.u_many(vec![error, exception]);
        let throwable_type = f.u(throwable);
        assert!(refines_of(f, union, throwable_type, &world));
    });
}

#[test]
fn throwable_with_full_negations_is_uninhabited() {
    fixture(|f| {
        let world = create_sealed_world();
        let throwable = f.t_named("Throwable");
        let error = f.t_named("Error");
        let exception = f.t_named("Exception");

        let error_type = f.u(error);
        let negated_error = f.builder.negated(error_type);
        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_without_inheritors = f.builder.intersected(throwable, &[negated_error, negated_exception]);

        assert!(is_uninhabited(throwable_without_inheritors, &world, &mut f.builder));
    });
}

#[test]
fn throwable_with_full_negations_meet_anything_is_never() {
    fixture(|f| {
        let world = create_sealed_world();
        let throwable = f.t_named("Throwable");
        let error = f.t_named("Error");
        let exception = f.t_named("Exception");

        let error_type = f.u(error);
        let negated_error = f.builder.negated(error_type);
        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_without_inheritors = f.builder.intersected(throwable, &[negated_error, negated_exception]);

        let input = f.u(throwable_without_inheritors);
        assert_eq!(meet_of(f, input, well_known::TYPE_MIXED, &world), well_known::TYPE_NEVER);
    });
}

#[test]
fn subtract_throwable_by_error_or_exception_is_never() {
    fixture(|f| {
        let world = create_sealed_world();
        let throwable = f.t_named("Throwable");
        let error = f.t_named("Error");
        let exception = f.t_named("Exception");

        let union = f.u_many(vec![error, exception]);
        let throwable_type = f.u(throwable);
        assert_eq!(subtract_of(f, throwable_type, union, &world), well_known::TYPE_NEVER);
    });
}

fn create_traversable_world<'arena>() -> MockWorld<'arena> {
    let mut world = MockWorld::new();
    world
        .add_edge("Iterator", "Traversable")
        .add_edge("IteratorAggregate", "Traversable")
        .with_sealed("Traversable", &["Iterator", "IteratorAggregate"]);
    world
}

#[test]
fn traversable_refines_iterator_or_iterator_aggregate() {
    fixture(|f| {
        let world = create_traversable_world();
        let traversable = f.t_named("Traversable");
        let iterator = f.t_named("Iterator");
        let iterator_aggregate = f.t_named("IteratorAggregate");

        let union = f.u_many(vec![iterator, iterator_aggregate]);
        let traversable_type = f.u(traversable);
        assert!(refines_of(f, traversable_type, union, &world));
    });
}

#[test]
fn traversable_with_full_negations_is_uninhabited() {
    fixture(|f| {
        let world = create_traversable_world();
        let traversable = f.t_named("Traversable");
        let iterator = f.t_named("Iterator");
        let iterator_aggregate = f.t_named("IteratorAggregate");

        let iterator_type = f.u(iterator);
        let negated_iterator = f.builder.negated(iterator_type);
        let iterator_aggregate_type = f.u(iterator_aggregate);
        let negated_iterator_aggregate = f.builder.negated(iterator_aggregate_type);
        let traversable_without_inheritors =
            f.builder.intersected(traversable, &[negated_iterator, negated_iterator_aggregate]);

        assert!(is_uninhabited(traversable_without_inheritors, &world, &mut f.builder));
    });
}

#[test]
fn partial_cover_does_not_collapse_when_residual_has_multiple() {
    fixture(|f| {
        let world = create_sealed_world();
        let throwable = f.t_named("Throwable");
        let exception = f.t_named("Exception");

        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_minus_exception = f.builder.intersected(throwable, &[negated_exception]);

        assert!(
            !is_uninhabited(throwable_minus_exception, &world, &mut f.builder),
            "Throwable & !Exception is not uninhabited: Error survives"
        );
    });
}

#[test]
fn transitive_negation_via_descendant() {
    fixture(|f| {
        let world = create_sealed_world();
        let throwable = f.t_named("Throwable");
        let exception = f.t_named("Exception");
        let runtime_exception = f.t_named("RuntimeException");

        let exception_type = f.u(exception);
        let negated_exception = f.builder.negated(exception_type);
        let throwable_minus_exception = f.builder.intersected(throwable, &[negated_exception]);
        let conjoined = f.builder.intersected(throwable_minus_exception, &[runtime_exception]);

        assert!(
            is_uninhabited(conjoined, &world, &mut f.builder),
            "RuntimeException refines Exception, so !Exception excludes it"
        );
    });
}

#[test]
fn transitive_sealing_collapses_to_never() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world
            .add_edge("Bar", "Foo")
            .add_edge("Baz", "Foo")
            .add_edge("Bar1", "Bar")
            .add_edge("Bar2", "Bar")
            .with_sealed("Foo", &["Bar", "Baz"])
            .with_sealed("Bar", &["Bar1", "Bar2"]);

        let foo = f.t_named("Foo");
        let bar1 = f.t_named("Bar1");
        let bar2 = f.t_named("Bar2");
        let baz = f.t_named("Baz");

        let bar1_type = f.u(bar1);
        let negated_bar1 = f.builder.negated(bar1_type);
        let bar2_type = f.u(bar2);
        let negated_bar2 = f.builder.negated(bar2_type);
        let baz_type = f.u(baz);
        let negated_baz = f.builder.negated(baz_type);
        let covered = f.builder.intersected(foo, &[negated_bar1, negated_bar2, negated_baz]);

        assert!(is_uninhabited(covered, &world, &mut f.builder));
    });
}

#[test]
fn unrelated_negation_does_not_affect_cover() {
    fixture(|f| {
        let world = create_sealed_world();
        let throwable = f.t_named("Throwable");

        let negated_int = f.builder.negated(well_known::TYPE_INT);
        let throwable_without_int = f.builder.intersected(throwable, &[negated_int]);

        assert!(
            !is_uninhabited(throwable_without_int, &world, &mut f.builder),
            "int is not in the sealed cover, so coverage is unaffected"
        );
    });
}

#[test]
fn non_class_head_skips_sealed_logic() {
    fixture(|f| {
        let world = create_sealed_world();
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

        assert!(
            !is_uninhabited(throwable_minus_exception, &world, &mut f.builder),
            "with NullWorld, Throwable is not sealed"
        );
    });
}

#[test]
fn final_class_with_negated_self_is_being_uninhabited_by_existing_rules() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.add_edge("Final", "Final");
        let final_class = f.t_named("Final");
        let final_type = f.u(final_class);
        let negated_final = f.builder.negated(final_type);
        let contradiction = f.builder.intersected(final_class, &[negated_final]);
        assert_eq!(contradiction, well_known::NEVER, "self-negation is always empty via intersected construction");
    });
}
