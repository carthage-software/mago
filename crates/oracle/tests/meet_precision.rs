mod common;

use std::collections::BTreeMap;

use common::*;

use mago_oracle::ty::Type;
use mago_oracle::ty::lattice::LatticeOptions;
use mago_oracle::ty::lattice::LatticeReport;
use mago_oracle::ty::meet;
use mago_oracle::ty::well_known;
use mago_oracle::world::Variance;
use mago_oracle::world::World;

fn meet_eq<'arena>(f: &mut Fixture<'_, 'arena>, a: Type<'arena>, b: Type<'arena>, expected: Type<'arena>) {
    let w = empty_world();
    let mut report = LatticeReport::new();
    let result = meet::compute(a, b, &w, LatticeOptions::default(), &mut report, &mut f.builder);
    assert_eq!(result, expected, "meet({a}, {b}) = {result}, expected {expected}");
}

fn meet_eq_with<'arena, W>(
    f: &mut Fixture<'_, 'arena>,
    a: Type<'arena>,
    b: Type<'arena>,
    expected: Type<'arena>,
    world: &W,
) where
    W: World<'arena>,
{
    let mut report = LatticeReport::new();
    let result = meet::compute(a, b, world, LatticeOptions::default(), &mut report, &mut f.builder);
    assert_eq!(result, expected, "meet({a}, {b}) = {result}, expected {expected}");
}

#[test]
fn numeric_meet_string_is_numeric_string() {
    fixture(|f| {
        let numeric = f.t_numeric();
        let lhs = f.u(numeric);
        let string = f.t_string();
        let rhs = f.u(string);
        let numeric_string = f.t_numeric_string();
        let expected = f.u(numeric_string);
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn lower_meet_upper_keeps_only_empty() {
    fixture(|f| {
        let lower = f.t_lower_string();
        let lhs = f.u(lower);
        let upper = f.t_upper_string();
        let rhs = f.u(upper);
        let expected = f.us("");
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn lower_meet_non_empty_is_lower_non_empty() {
    fixture(|f| {
        let lower = f.t_lower_string();
        let lhs = f.u(lower);
        let non_empty = f.t_non_empty_string();
        let rhs = f.u(non_empty);
        let expected = f.u(well_known::NON_EMPTY_LOWERCASE_STRING);
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn upper_meet_non_empty_is_upper_non_empty() {
    fixture(|f| {
        let upper = f.t_upper_string();
        let lhs = f.u(upper);
        let non_empty = f.t_non_empty_string();
        let rhs = f.u(non_empty);
        let expected = f.u(well_known::NON_EMPTY_UPPERCASE_STRING);
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn truthy_meet_numeric_is_truthy_numeric() {
    fixture(|f| {
        let truthy = f.t_truthy_string();
        let lhs = f.u(truthy);
        let numeric = f.t_numeric_string();
        let rhs = f.u(numeric);
        let expected = f.u(well_known::TRUTHY_NUMERIC_STRING);
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn array_key_meet_int_is_int() {
    fixture(|f| {
        let array_key = f.t_array_key();
        let lhs = f.u(array_key);
        let int = f.t_int();
        let rhs = f.u(int);
        let expected = f.u(int);
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn array_key_meet_string_is_string() {
    fixture(|f| {
        let array_key = f.t_array_key();
        let lhs = f.u(array_key);
        let string = f.t_string();
        let rhs = f.u(string);
        let expected = f.u(string);
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn scalar_meet_bool_is_bool() {
    fixture(|f| {
        let scalar = f.t_scalar();
        let lhs = f.u(scalar);
        let bool_atom = f.t_bool();
        let rhs = f.u(bool_atom);
        let expected = f.u(bool_atom);
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn open_resource_meet_closed_resource_is_never() {
    fixture(|f| {
        let open = f.t_open_resource();
        let lhs = f.u(open);
        let closed = f.t_closed_resource();
        let rhs = f.u(closed);
        meet_eq(f, lhs, rhs, well_known::TYPE_NEVER);
    });
}

#[test]
fn class_string_unrelated_meet_is_never() {
    fixture(|f| {
        let foo = f.t_lit_class_string("Foo");
        let lhs = f.u(foo);
        let bar = f.t_lit_class_string("Bar");
        let rhs = f.u(bar);
        meet_eq(f, lhs, rhs, well_known::TYPE_NEVER);
    });
}

#[test]
fn class_string_descendant_meet_is_descendant() {
    fixture(|f| {
        let mut w = MockWorld::new();
        w.add_edge("Bar", "Foo");
        let foo_named = f.t_named("Foo");
        let foo_ty = f.u(foo_named);
        let parent_atom = f.t_class_string_of(foo_ty);
        let parent = f.u(parent_atom);
        let bar_named = f.t_named("Bar");
        let bar_ty = f.u(bar_named);
        let child_atom = f.t_class_string_of(bar_ty);
        let child = f.u(child_atom);
        meet_eq_with(f, parent, child, child, &w);
    });
}

#[test]
fn list_int_meet_list_string_is_list_never() {
    fixture(|f| {
        let lhs_atom = f.t_list(well_known::TYPE_INT, false);
        let lhs = f.u(lhs_atom);
        let rhs_atom = f.t_list(well_known::TYPE_STRING, false);
        let rhs = f.u(rhs_atom);
        let expected_atom = f.t_list(well_known::TYPE_NEVER, false);
        let expected = f.u(expected_atom);
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn keyed_array_disjoint_keys_meet_is_combined_shape() {
    fixture(|f| {
        let key_a = f.ak_str("a");
        let key_b = f.ak_str("b");
        let lhs_atom = f.t_keyed_sealed(BTreeMap::from([(key_a, (false, well_known::TYPE_INT))]), false);
        let lhs = f.u(lhs_atom);
        let rhs_atom = f.t_keyed_sealed(BTreeMap::from([(key_b, (false, well_known::TYPE_STRING))]), false);
        let rhs = f.u(rhs_atom);
        let expected_atom = f.t_keyed_sealed(
            BTreeMap::from([(key_a, (false, well_known::TYPE_INT)), (key_b, (false, well_known::TYPE_STRING))]),
            false,
        );
        let expected = f.u(expected_atom);
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn iterable_int_int_meet_iterable_int_string_is_iterable_int_never() {
    fixture(|f| {
        let lhs_atom = f.t_iterable(well_known::TYPE_INT, well_known::TYPE_INT);
        let lhs = f.u(lhs_atom);
        let rhs_atom = f.t_iterable(well_known::TYPE_INT, well_known::TYPE_STRING);
        let rhs = f.u(rhs_atom);
        let expected_atom = f.t_iterable(well_known::TYPE_INT, well_known::TYPE_NEVER);
        let expected = f.u(expected_atom);
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn iterable_int_a_meet_iterable_int_b_keys_intersect() {
    fixture(|f| {
        let lhs_atom = f.t_iterable(well_known::TYPE_ARRAY_KEY, well_known::TYPE_MIXED);
        let lhs = f.u(lhs_atom);
        let rhs_atom = f.t_iterable(well_known::TYPE_INT, well_known::TYPE_MIXED);
        let rhs = f.u(rhs_atom);
        let expected_atom = f.t_iterable(well_known::TYPE_INT, well_known::TYPE_MIXED);
        let expected = f.u(expected_atom);
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn callable_meet_with_compatible_signatures_intersects_return_unions_params() {
    fixture(|f| {
        let lhs_atom = f.t_callable(&[well_known::TYPE_INT], well_known::TYPE_INT);
        let lhs = f.u(lhs_atom);
        let rhs_atom = f.t_callable(&[well_known::TYPE_INT], well_known::TYPE_STRING);
        let rhs = f.u(rhs_atom);
        let expected_atom = f.t_callable(&[well_known::TYPE_INT], well_known::TYPE_NEVER);
        let expected = f.u(expected_atom);
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn class_string_unrelated_constraints_meet_is_never() {
    fixture(|f| {
        let w = MockWorld::new();
        let foo_named = f.t_named("Foo");
        let foo_ty = f.u(foo_named);
        let lhs_atom = f.t_class_string_of(foo_ty);
        let lhs = f.u(lhs_atom);
        let bar_named = f.t_named("Bar");
        let bar_ty = f.u(bar_named);
        let rhs_atom = f.t_class_string_of(bar_ty);
        let rhs = f.u(rhs_atom);
        meet_eq_with(f, lhs, rhs, well_known::TYPE_NEVER, &w);
    });
}

#[test]
fn class_string_kinds_disjoint_meet_is_never() {
    fixture(|f| {
        let w = MockWorld::new();
        let foo_named = f.t_named("Foo");
        let foo_ty = f.u(foo_named);
        let class_atom = f.t_class_string_of(foo_ty);
        let class = f.u(class_atom);
        let interface_atom = f.t_interface_string_of(foo_ty);
        let interface = f.u(interface_atom);
        meet_eq_with(f, class, interface, well_known::TYPE_NEVER, &w);
    });
}

#[test]
fn enum_meet_enum_case_is_case() {
    fixture(|f| {
        let w = MockWorld::new();
        let any_atom = f.t_enum("E");
        let any = f.u(any_atom);
        let case_atom = f.t_enum_case("E", "A");
        let case = f.u(case_atom);
        meet_eq_with(f, any, case, case, &w);
    });
}

#[test]
fn distinct_enum_cases_meet_is_never() {
    fixture(|f| {
        let w = MockWorld::new();
        let a_atom = f.t_enum_case("E", "A");
        let a = f.u(a_atom);
        let b_atom = f.t_enum_case("E", "B");
        let b = f.u(b_atom);
        meet_eq_with(f, a, b, well_known::TYPE_NEVER, &w);
    });
}

#[test]
fn distinct_enums_meet_is_never() {
    fixture(|f| {
        let w = MockWorld::new();
        let e_atom = f.t_enum("E");
        let e = f.u(e_atom);
        let f_atom = f.t_enum("F");
        let f_ty = f.u(f_atom);
        meet_eq_with(f, e, f_ty, well_known::TYPE_NEVER, &w);
    });
}

#[test]
fn has_method_meet_has_method_composes() {
    fixture(|f| {
        let lhs_atom = f.t_has_method("foo");
        let lhs = f.u(lhs_atom);
        let rhs_atom = f.t_has_method("bar");
        let rhs = f.u(rhs_atom);
        let w = empty_world();
        let mut report = LatticeReport::new();
        let result = meet::compute(lhs, rhs, &w, LatticeOptions::default(), &mut report, &mut f.builder);
        assert_ne!(result, well_known::TYPE_NEVER, "has-method ∧ has-method should compose, got NEVER");
    });
}

#[test]
fn named_object_with_method_meet_has_method_passes_when_world_confirms() {
    fixture(|f| {
        let mut w = MockWorld::new();
        w.with_method("Foo", "doFoo");
        let named_atom = f.t_named("Foo");
        let named = f.u(named_atom);
        let constraint_atom = f.t_has_method("doFoo");
        let constraint = f.u(constraint_atom);
        let mut report = LatticeReport::new();
        let result = meet::compute(named, constraint, &w, LatticeOptions::default(), &mut report, &mut f.builder);
        assert_eq!(result, named, "Named(Foo) ∧ has_method(doFoo) should reduce to Named(Foo)");
    });
}

#[test]
fn empty_array_meet_list_int_is_empty_array() {
    fixture(|f| {
        let empty = f.t_empty_array();
        let lhs = f.u(empty);
        let list_atom = f.t_list(well_known::TYPE_INT, false);
        let rhs = f.u(list_atom);
        meet_eq(f, lhs, rhs, lhs);
    });
}

#[test]
fn list_int_meet_keyed_int_int_is_list_int() {
    fixture(|f| {
        let lhs_atom = f.t_list(well_known::TYPE_INT, false);
        let lhs = f.u(lhs_atom);
        let rhs_atom = f.t_keyed_unsealed(well_known::TYPE_INT, well_known::TYPE_INT, false);
        let rhs = f.u(rhs_atom);
        meet_eq(f, lhs, rhs, lhs);
    });
}

#[test]
fn truthy_mixed_meet_falsy_mixed_is_never() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        let lhs = f.u(truthy);
        let falsy = f.mixed_falsy();
        let rhs = f.u(falsy);
        meet_eq(f, lhs, rhs, well_known::TYPE_NEVER);
    });
}

#[test]
fn nonnull_mixed_meet_null_is_never() {
    fixture(|f| {
        let nonnull = f.mixed_nonnull();
        let lhs = f.u(nonnull);
        let null = f.null();
        let rhs = f.u(null);
        meet_eq(f, lhs, rhs, well_known::TYPE_NEVER);
    });
}

#[test]
fn truthy_mixed_meet_int_is_truthy_int_set() {
    fixture(|f| {
        let w = empty_world();
        let truthy = f.mixed_truthy();
        let lhs = f.u(truthy);
        let int = f.t_int();
        let rhs = f.u(int);
        let mut report = LatticeReport::new();
        let result = meet::compute(lhs, rhs, &w, LatticeOptions::default(), &mut report, &mut f.builder);
        assert_ne!(result, well_known::TYPE_NEVER, "truthy_mixed ∧ int should be non-empty");
    });
}

#[test]
fn template_with_int_or_string_meet_int_narrows_constraint_to_int() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let int_or_string = f.u_many(vec![int, string]);
        let lhs_atom = f.t_template_of("C", "T", int_or_string);
        let lhs = f.u(lhs_atom);
        let rhs = f.u(int);
        let int_ty = f.u(int);
        let expected_atom = f.t_template_of("C", "T", int_ty);
        let expected = f.u(expected_atom);
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn template_with_int_or_string_meet_string_narrows_constraint_to_string() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let int_or_string = f.u_many(vec![int, string]);
        let lhs_atom = f.t_template_of("C", "T", int_or_string);
        let lhs = f.u(lhs_atom);
        let rhs = f.u(string);
        let string_ty = f.u(string);
        let expected_atom = f.t_template_of("C", "T", string_ty);
        let expected = f.u(expected_atom);
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn template_with_int_meet_string_is_impossible() {
    fixture(|f| {
        let int = f.t_int();
        let int_ty = f.u(int);
        let lhs_atom = f.t_template_of("C", "T", int_ty);
        let lhs = f.u(lhs_atom);
        let string = f.t_string();
        let rhs = f.u(string);
        meet_eq(f, lhs, rhs, well_known::TYPE_NEVER);
    });
}

#[test]
fn template_with_int_meet_int_is_redundant_keeps_template() {
    fixture(|f| {
        let int = f.t_int();
        let int_ty = f.u(int);
        let lhs_atom = f.t_template_of("C", "T", int_ty);
        let lhs = f.u(lhs_atom);
        let rhs = f.u(int);
        meet_eq(f, lhs, rhs, lhs);
    });
}

#[test]
fn same_template_meet_with_overlapping_constraints_intersects_them() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let float = f.t_float();
        let int_or_string = f.u_many(vec![int, string]);
        let int_or_float = f.u_many(vec![int, float]);
        let lhs_atom = f.t_template_of("C", "T", int_or_string);
        let lhs = f.u(lhs_atom);
        let rhs_atom = f.t_template_of("C", "T", int_or_float);
        let rhs = f.u(rhs_atom);
        let int_ty = f.u(int);
        let expected_atom = f.t_template_of("C", "T", int_ty);
        let expected = f.u(expected_atom);
        meet_eq(f, lhs, rhs, expected);
    });
}

#[test]
fn distinct_templates_have_no_meet_rule_and_collapse_to_never() {
    fixture(|f| {
        let int = f.t_int();
        let int_ty = f.u(int);
        let lhs_atom = f.t_template_of("C", "T", int_ty);
        let lhs = f.u(lhs_atom);
        let rhs_atom = f.t_template_of("C", "U", int_ty);
        let rhs = f.u(rhs_atom);
        meet_eq(f, lhs, rhs, well_known::TYPE_NEVER);
    });
}

#[test]
fn contravariant_a_object_meet_a_int_under_contravariant_t_subsumes_to_more_specific() {
    fixture(|f| {
        let mut w = MockWorld::new();
        w.with_templates("A", &[("T", Variance::Contravariant)]);
        let object_named = f.t_named("Object");
        let object_ty = f.u(object_named);
        let a_object_atom = f.t_generic_named("A", vec![object_ty]);
        let a_object = f.u(a_object_atom);
        let int = f.t_int();
        let int_ty = f.u(int);
        let a_int_atom = f.t_generic_named("A", vec![int_ty]);
        let a_int = f.u(a_int_atom);

        let mut report = LatticeReport::new();
        let m = meet::compute(a_object, a_int, &w, LatticeOptions::default(), &mut report, &mut f.builder);
        let r1 = is_contained(f, m, a_object, &w);
        let r2 = is_contained(f, m, a_int, &w);
        assert!(r1, "meet={m} should refine {a_object}");
        assert!(r2, "meet={m} should refine {a_int}");
    });
}

#[test]
fn associativity_array_bool_int_meet_via_arb_failing_case() {
    fixture(|f| {
        let w = empty_world();

        let a_atom = f.t_keyed_unsealed(well_known::TYPE_INT, well_known::TYPE_INT, false);
        let a = f.u(a_atom);
        let array_bool_int_atom = f.t_keyed_unsealed(well_known::TYPE_BOOL, well_known::TYPE_INT, false);
        let array_bool_int = f.u(array_bool_int_atom);
        let b_atoms: Vec<_> = array_bool_int.atoms.iter().chain(well_known::TYPE_INT.atoms.iter()).copied().collect();
        let b = f.u_many(b_atoms);
        let c_atom = f.t_list(well_known::TYPE_INT, false);
        let c = f.u(c_atom);

        let mut report = LatticeReport::new();
        let bc = meet::compute(b, c, &w, LatticeOptions::default(), &mut report, &mut f.builder);
        let r = meet::compute(a, bc, &w, LatticeOptions::default(), &mut report, &mut f.builder);

        let r_refines_b = is_contained(f, r, b, &w);
        assert!(r_refines_b);
    });
}

#[test]
fn final_class_intersected_with_unrelated_is_uninhabited_in_meet() {
    fixture(|f| {
        let mut w = MockWorld::new();
        w.with_final("Foo");
        w.declare("Bar");
        let foo_atom = f.t_named("Foo");
        let foo = f.u(foo_atom);
        let bar_atom = f.t_named("Bar");
        let bar = f.u(bar_atom);
        meet_eq_with(f, foo, bar, well_known::TYPE_NEVER, &w);
    });
}

#[test]
fn enum_intersected_with_unrelated_class_is_uninhabited() {
    fixture(|f| {
        let mut w = MockWorld::new();
        w.with_pure_enum("Color");
        w.declare("Bar");
        let color_atom = f.t_enum("Color");
        let color = f.u(color_atom);
        let bar_atom = f.t_named("Bar");
        let bar = f.u(bar_atom);
        meet_eq_with(f, color, bar, well_known::TYPE_NEVER, &w);
    });
}

#[test]
fn unrelated_objects_no_finality_overlap_is_open() {
    fixture(|f| {
        let mut w = MockWorld::new();
        w.with_templates("A", &[("T", Variance::Invariant)]);
        w.with_templates("B", &[("T", Variance::Contravariant)]);
        w.declare("E");
        w.with_extended("A", "B", vec![well_known::TYPE_MIXED]);
        w.with_final("B");

        let object_ty = f.u(well_known::OBJECT);
        let a_atom = f.t_generic_named("A", vec![object_ty]);
        let a = f.u(a_atom);
        let e_atom = f.t_named("E");
        let e = f.u(e_atom);
        let o = overlaps(f, a, e, &w);
        let mut report = LatticeReport::new();
        let m = meet::compute(a, e, &w, LatticeOptions::default(), &mut report, &mut f.builder);
        if m != well_known::TYPE_NEVER {
            assert!(o, "non-never meet ({m}) should imply overlap");
        }
    });
}

#[test]
fn empty_array_meet_array_int_int_collapses_to_empty() {
    fixture(|f| {
        let w = empty_world();
        let array_int_int_atom = f.t_keyed_unsealed(well_known::TYPE_INT, well_known::TYPE_INT, false);
        let array_int_int = f.u(array_int_int_atom);
        let empty_array = f.u(well_known::EMPTY_ARRAY);
        let mut report = LatticeReport::new();
        let m = meet::compute(array_int_int, empty_array, &w, LatticeOptions::default(), &mut report, &mut f.builder);
        assert!(is_contained(f, m, empty_array, &w));
        assert!(is_contained(f, m, array_int_int, &w));
    });
}

#[test]
fn associativity_array_int_int_meet_via_arb_failing_case() {
    fixture(|f| {
        let w = empty_world();

        let array_int_int_atom = f.t_keyed_unsealed(well_known::TYPE_INT, well_known::TYPE_INT, false);
        let array_int_int = f.u(array_int_int_atom);
        let list_never_atom = f.t_list(well_known::TYPE_NEVER, false);
        let list_never = f.u(list_never_atom);
        let empty = f.u(well_known::EMPTY_ARRAY);
        let bc = f.u_many(vec![list_never.atoms[0], empty.atoms[0]]);

        let mut report = LatticeReport::new();
        let r = meet::compute(array_int_int, bc, &w, LatticeOptions::default(), &mut report, &mut f.builder);

        let b_atom = f.t_keyed_unsealed(well_known::TYPE_INT, well_known::TYPE_INT, false);
        let b = f.u(b_atom);
        let r_refines_b = is_contained(f, r, b, &w);
        assert!(r_refines_b, "result should refine the b-shaped target");
    });
}

#[test]
fn refines_a_descending_c_int_under_contravariant_t() {
    fixture(|f| {
        let mut w = MockWorld::new();
        w.with_templates("A", &[("T", Variance::Invariant)]);
        w.with_templates("B", &[("T", Variance::Contravariant)]);
        w.with_templates("C", &[("T", Variance::Contravariant)]);
        w.with_extended("A", "B", vec![well_known::TYPE_MIXED]);
        w.with_extended("B", "C", vec![well_known::TYPE_MIXED]);

        let a_atom = f.t_named("A");
        let a = f.u(a_atom);
        let int = f.t_int();
        let int_ty = f.u(int);
        let c_int_atom = f.t_generic_named("C", vec![int_ty]);
        let c_int = f.u(c_int_atom);

        let result = is_contained(f, a, c_int, &w);
        assert!(
            result,
            "A should refine C<int> under contravariant T (mixed inherited from chain refines int via contravariance)"
        );
    });
}

#[test]
fn associativity_a_numeric_meet_a_intersected_has_method_meet_a_a() {
    fixture(|f| {
        let mut w = MockWorld::new();
        w.with_templates("A", &[("T", Variance::Invariant)]);

        let numeric = f.t_numeric();
        let numeric_ty = f.u(numeric);
        let a_numeric_atom = f.t_generic_named("A", vec![numeric_ty]);
        let a_numeric = f.u(a_numeric_atom);
        let int = f.t_int();
        let a_int = f.u(int);
        let a_named = f.t_named("A");
        let a_named_ty = f.u(a_named);
        let a_a_atom = f.t_generic_named("A", vec![a_named_ty]);
        let a_a = f.u(a_a_atom);
        let has_method = f.t_has_method("doFoo");
        let a_intersected_atom = f.t_named_intersected("A", &[has_method]);
        let a_intersected_has_method = f.u(a_intersected_atom);

        let int_ty = a_int;
        let a_top = f.u_many(vec![a_numeric.atoms[0], int_ty.atoms[0]]);
        let b_top = a_intersected_has_method;
        let c_top = a_a;

        let mut report = LatticeReport::new();
        let bc = meet::compute(b_top, c_top, &w, LatticeOptions::default(), &mut report, &mut f.builder);
        let r = meet::compute(a_top, bc, &w, LatticeOptions::default(), &mut report, &mut f.builder);
        let r_refines_c = is_contained(f, r, c_top, &w);
        assert!(r_refines_c, "a∩(b∩c) ({r}) should refine c ({c_top})");
    });
}

#[test]
fn list_intersection_overlap_consistency_arb_case() {
    fixture(|f| {
        let mut w = MockWorld::new();
        w.with_templates("A", &[("T", Variance::Invariant)]);
        w.with_templates("B", &[("T", Variance::Invariant)]);
        w.with_templates("E", &[("T", Variance::Invariant)]);

        let e_named = f.t_named("E");
        let b_and_e_atom = f.t_named_intersected("B", &[e_named]);
        let b_and_e = f.u(b_and_e_atom);
        let int_atom = f.t_int();
        let int = f.u(int_atom);
        let int_zero = f.ui(0);
        let never = well_known::TYPE_NEVER;
        let mut elems: Vec<_> = b_and_e.atoms.to_vec();
        elems.extend_from_slice(int.atoms);
        elems.extend_from_slice(int_zero.atoms);
        elems.extend_from_slice(never.atoms);
        let element_union = f.u_many(elems);
        let a_atom = f.t_list(element_union, true);
        let a = f.u(a_atom);

        let int_ty = f.u(int_atom);
        let a_int_atom = f.t_generic_named("A", vec![int_ty]);
        let a_int_ty = f.u(a_int_atom);
        let a_int_list_atom = f.t_list(a_int_ty, true);
        let a_int_in_list = f.u(a_int_list_atom);

        let mut report = LatticeReport::new();
        let m = meet::compute(a, a_int_in_list, &w, LatticeOptions::default(), &mut report, &mut f.builder);
        let o = overlaps(f, a, a_int_in_list, &w);
        if m != well_known::TYPE_NEVER {
            assert!(o, "non-never meet ({m}) should imply overlap");
        }
    });
}

#[test]
fn invariant_a_associativity_arb_failing_case() {
    fixture(|f| {
        let mut w = MockWorld::new();
        w.with_templates("A", &[("T", Variance::Invariant)]);
        w.with_templates("B", &[("T", Variance::Contravariant)]);
        w.with_templates("C", &[("T", Variance::Contravariant)]);
        w.with_templates("D", &[("T", Variance::Invariant)]);
        w.with_extended("B", "C", vec![well_known::TYPE_MIXED]);
        w.with_extended("A", "B", vec![well_known::TYPE_MIXED]);

        let object = f.u(well_known::OBJECT);
        let a_object_atom = f.t_generic_named("A", vec![object]);
        let a_object = f.u(a_object_atom);
        let int_atom = f.t_int();
        let int_ty = f.u(int_atom);
        let a_int_atom = f.t_generic_named("A", vec![int_ty]);
        let a_int = f.u(a_int_atom);
        let a_bare_atom = f.t_named("A");
        let a_bare = f.u(a_bare_atom);

        let a_t = f.u_many(vec![a_object.atoms[0], int_atom]);
        let b_t = f.u_many(vec![a_bare.atoms[0], int_atom]);
        let c_t = a_int;

        let mut report = LatticeReport::new();
        let ab = meet::compute(a_t, b_t, &w, LatticeOptions::default(), &mut report, &mut f.builder);
        let l = meet::compute(ab, c_t, &w, LatticeOptions::default(), &mut report, &mut f.builder);
        let bc = meet::compute(b_t, c_t, &w, LatticeOptions::default(), &mut report, &mut f.builder);
        let r = meet::compute(a_t, bc, &w, LatticeOptions::default(), &mut report, &mut f.builder);

        let l_refines_c = is_contained(f, l, c_t, &w);
        let r_refines_c = is_contained(f, r, c_t, &w);
        assert!(l_refines_c, "(a∩b)∩c should refine c");
        assert!(r_refines_c, "a∩(b∩c) should refine c");
    });
}
