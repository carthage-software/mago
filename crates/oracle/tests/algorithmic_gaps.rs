mod common;

use common::*;

use mago_oracle::symbol::part::generic::Variance;
use mago_oracle::ty::Type;
use mago_oracle::ty::lattice;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::meet;
use mago_oracle::ty::subtract;
use mago_oracle::ty::well_known;
use mago_oracle::world::World;

fn lattice_meet<'arena, W>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>, world: &W) -> Type<'arena>
where
    W: World<'arena>,
{
    meet::compute(a, b, world, LatticeOptions::default(), &mut LatticeReport::new(), &mut f.builder)
}

fn lattice_subtract<'arena, W>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>, world: &W) -> Type<'arena>
where
    W: World<'arena>,
{
    subtract::compute(a, b, world, LatticeOptions::default(), &mut LatticeReport::new(), &mut f.builder)
}

fn does_refine<'arena, W>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>, world: &W) -> bool
where
    W: World<'arena>,
{
    lattice::refines(a, b, world, LatticeOptions::default(), &mut LatticeReport::new(), &mut f.builder)
}

#[test]
fn gap_compose_descendant_invariant_mismatch_collapses_to_never() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("B", &[("T", Variance::Invariant)]);
        world.with_templates("C", &[("T", Variance::Invariant)]);
        let template = f.t_template("B", "T");
        let template_type = f.u(template);
        world.with_extended("B", "C", vec![template_type]);

        let int = f.t_int();
        let int_type = f.u(int);
        let b_int_atom = f.t_generic_named("B", vec![int_type]);
        let b_int = f.u(b_int_atom);
        let object = f.t_object_any();
        let object_type = f.u(object);
        let c_object_atom = f.t_generic_named("C", vec![object_type]);
        let c_object = f.u(c_object_atom);

        let met = lattice_meet(f, b_int, c_object, &world);
        assert_eq!(
            met,
            well_known::TYPE_NEVER,
            "B<int> as C is C<int>; meet with C<object> under invariant T should be never (got {met})"
        );
    });
}

#[test]
fn gap_subtract_scalar_minus_int_splits_to_other_scalars() {
    fixture(|f| {
        let world = empty_world();
        let difference = lattice_subtract(f, well_known::TYPE_SCALAR, well_known::TYPE_INT, &world);
        let bool_atom = f.t_bool();
        let float = f.t_float();
        let string = f.t_string();
        let expected = f.u_many(vec![bool_atom, float, string]);
        assert!(
            does_refine(f, difference, expected, &world) && does_refine(f, expected, difference, &world),
            "scalar \\ int should equal bool|float|string (got {difference})"
        );
    });
}

#[test]
fn gap_subtract_array_key_minus_int_yields_string() {
    fixture(|f| {
        let world = empty_world();
        let difference = lattice_subtract(f, well_known::TYPE_ARRAY_KEY, well_known::TYPE_INT, &world);
        let string = f.t_string();
        let expected = f.u(string);
        assert!(
            does_refine(f, difference, expected, &world) && does_refine(f, expected, difference, &world),
            "array-key \\ int should equal string (got {difference})"
        );
    });
}

#[test]
fn gap_subtract_numeric_minus_int_yields_float_or_numeric_string() {
    fixture(|f| {
        let world = empty_world();
        let difference = lattice_subtract(f, well_known::TYPE_NUMERIC, well_known::TYPE_INT, &world);
        let float = f.t_float();
        let numeric_string = f.t_numeric_string();
        let expected = f.u_many(vec![float, numeric_string]);
        assert!(
            does_refine(f, difference, expected, &world) && does_refine(f, expected, difference, &world),
            "numeric \\ int should equal float|numeric-string (got {difference})"
        );
    });
}

#[test]
fn gap_subtract_string_minus_string_literal_preserves_narrowing() {
    fixture(|f| {
        let world = empty_world();
        let string = f.t_string();
        let string_type = f.u(string);
        let literal = f.us("tbl");
        let difference = lattice_subtract(f, string_type, literal, &world);
        assert!(does_refine(f, difference, string_type, &world), "result must refine string (got {difference})");
        let removed = f.us("tbl");
        let met = lattice_meet(f, difference, removed, &world);
        assert_eq!(
            met,
            well_known::TYPE_NEVER,
            "string \\ string('tbl') must be disjoint from string('tbl') (got meet {met})"
        );
    });
}

#[test]
fn gap_subtract_b_minus_descendant_a_excludes_a_instances() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.declare("B");
        world.declare("A");
        world.with_extended("A", "B", vec![]);

        let base_atom = f.t_named("B");
        let base = f.u(base_atom);
        let descendant_atom = f.t_named("A");
        let descendant = f.u(descendant_atom);
        let difference = lattice_subtract(f, base, descendant, &world);

        let met = lattice_meet(f, difference, descendant, &world);
        assert_eq!(met, well_known::TYPE_NEVER, "(B \\ A) ∩ A should be never (got {met}; B \\ A = {difference})");
    });
}

#[test]
fn gap_meet_iterable_with_array_yields_array() {
    fixture(|f| {
        let world = empty_world();
        let int = f.t_int();
        let int_type = f.u(int);
        let string = f.t_string();
        let string_type = f.u(string);
        let iterable_atom = f.t_iterable(int_type, string_type);
        let iterable = f.u(iterable_atom);
        let array_key = f.t_array_key();
        let array_key_type = f.u(array_key);
        let array_atom = f.t_keyed_unsealed(array_key_type, string_type, false);
        let array = f.u(array_atom);
        let met = lattice_meet(f, iterable, array, &world);
        let expected_atom = f.t_keyed_unsealed(int_type, string_type, false);
        let expected = f.u(expected_atom);
        assert!(
            does_refine(f, met, expected, &world) && does_refine(f, expected, met, &world),
            "meet(iterable<int,string>, array<array-key,string>) should narrow to array<int,string> (got {met})"
        );
    });
}

#[test]
fn gap_refines_string_covered_by_lowercase_and_non_lowercase() {
    fixture(|f| {
        let world = empty_world();
        let string = f.t_string();
        let string_type = f.u(string);
        let lower = f.t_lower_string();
        let lower_type = f.u(lower);
        let non_lower = f.builder.negated(lower_type);
        let split = f.u_many(vec![lower, non_lower]);
        assert!(
            does_refine(f, string_type, split, &world),
            "string should refine lowercase-string | !lowercase-string"
        );
    });
}

#[test]
fn gap_concrete_class_does_not_refine_negated_descendant() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.add_edge("Bar", "Foo");
        let foo_atom = f.t_named("Foo");
        let foo = f.u(foo_atom);
        let bar_atom = f.t_named("Bar");
        let bar = f.u(bar_atom);
        let not_bar = f.builder.negated(bar);
        let not_bar_type = f.u(not_bar);
        assert!(
            !does_refine(f, foo, not_bar_type, &world),
            "a Foo value can be a Bar (Bar extends Foo), so Foo does not refine !Bar"
        );
    });
}

#[test]
fn gap_arity_zero_class_with_explicit_args_reduces_to_bare() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.declare("Foo");
        let int = f.t_int();
        let int_type = f.u(int);
        let with_args_atom = f.t_generic_named("Foo", vec![int_type]);
        let with_args = f.u(with_args_atom);
        let bare_atom = f.t_named("Foo");
        let bare = f.u(bare_atom);
        assert!(
            does_refine(f, with_args, bare, &world) && does_refine(f, bare, with_args, &world),
            "arity-0 Foo<int> should be value-equivalent to bare Foo\n  with_args={with_args}\n  bare={bare}"
        );
    });
}
