mod common;

use common::*;

use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known;

fn t_param_with_constraint<'arena>(
    f: &mut Fixture<'_, 'arena>,
    class_name: &str,
    template_name: &str,
    constraint: Type<'arena>,
) -> Atom<'arena> {
    f.t_template_of(class_name, template_name, constraint)
}

#[test]
fn same_t_reflexive_under_same_defining_entity() {
    fixture(|f| {
        let t1 = f.t_template("Box", "T");
        let t2 = f.t_template("Box", "T");
        assert!(atomic_is_contained(f, t1, t2, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn different_defining_entities_not_same_t() {
    fixture(|f| {
        let box_t = f.t_template("Box", "T");
        let bag_t = f.t_template("Bag", "T");
        assert!(!atomic_is_contained(f, box_t, bag_t, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn different_parameter_names_same_class_not_same_t() {
    fixture(|f| {
        let box_t = f.t_template("Box", "T");
        let box_u = f.t_template("Box", "U");
        assert!(!atomic_is_contained(f, box_t, box_u, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn template_with_mixed_constraint_refines_mixed() {
    fixture(|f| {
        let t = f.t_template("Box", "T");
        let mixed = f.mixed();
        assert!(atomic_is_contained(f, t, mixed, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn template_with_int_constraint_refines_int() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let t = t_param_with_constraint(f, "Box", "T", int_type);
        let int = f.t_int();
        assert!(atomic_is_contained(f, t, int, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn template_with_int_constraint_refines_array_key() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let t = t_param_with_constraint(f, "Box", "T", int_type);
        let array_key = f.t_array_key();
        assert!(atomic_is_contained(f, t, array_key, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn template_with_int_constraint_does_not_refine_string() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let t = t_param_with_constraint(f, "Box", "T", int_type);
        let string = f.t_string();
        assert!(!atomic_is_contained(f, t, string, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn concrete_value_does_not_refine_template_parameter() {
    fixture(|f| {
        let t = f.t_template("Box", "T");
        let int = f.t_int();
        let string = f.t_string();
        assert!(!atomic_is_contained(f, int, t, &empty_symbol_table(f.arena)));
        assert!(!atomic_is_contained(f, string, t, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn template_self_refines_via_mixed_top() {
    fixture(|f| {
        let t = f.t_template("Box", "T");
        let mixed = f.mixed();
        assert!(atomic_is_contained(f, t, t, &empty_symbol_table(f.arena)));
        assert!(atomic_is_contained(f, t, mixed, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn template_with_named_constraint_refines_ancestor() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Animal {} class Dog extends Animal {}");
        let dog = f.t_named("Dog");
        let dog_type = f.u(dog);
        let t = t_param_with_constraint(f, "Owner", "T", dog_type);
        let animal = f.t_named("Animal");
        let cat = f.t_named("Cat");
        assert!(atomic_is_contained(f, t, animal, &symbols));
        assert!(!atomic_is_contained(f, t, cat, &symbols));
    });
}

#[test]
fn never_refines_any_template() {
    fixture(|f| {
        let t = f.t_template("Box", "T");
        let t_type = f.u(t);
        assert!(is_contained(f, well_known::TYPE_NEVER, t_type, &empty_symbol_table(f.arena)));
    });
}

#[test]
fn inherited_t_refines_transferred_parameter() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php
/** @template TD */
class D {}
/**
 * @template TC
 * @extends D<TC>
 */
class C extends D {}",
        );
        let child_param = f.t_template("C", "TC");
        let parent_param = f.t_template("D", "TD");
        assert!(
            atomic_is_contained(f, child_param, parent_param, &symbols),
            "C extends D<TC>, so a TC value is the same variable as D's TD"
        );
    });
}

#[test]
fn inherited_t_is_transitive_across_three_levels() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php
/** @template TE */
class E {}
/**
 * @template TD
 * @extends E<TD>
 */
class D extends E {}
/**
 * @template TC
 * @extends D<TC>
 */
class C extends D {}",
        );
        let c_param = f.t_template("C", "TC");
        let d_param = f.t_template("D", "TD");
        let e_param = f.t_template("E", "TE");
        assert!(atomic_is_contained(f, c_param, d_param, &symbols), "TC <: TD (C extends D<TC>)");
        assert!(atomic_is_contained(f, d_param, e_param, &symbols), "TD <: TE (D extends E<TD>)");
        assert!(atomic_is_contained(f, c_param, e_param, &symbols), "transitivity: TC <: TD and TD <: TE imply TC <: TE");
    });
}

#[test]
fn inherited_t_relation_is_one_way() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php
/** @template TD */
class D {}
/**
 * @template TC
 * @extends D<TC>
 */
class C extends D {}",
        );
        let child_param = f.t_template("C", "TC");
        let parent_param = f.t_template("D", "TD");
        assert!(
            !atomic_is_contained(f, parent_param, child_param, &symbols),
            "a bare D's TD could be specialised to anything, so TD does not refine C's TC"
        );
    });
}

#[test]
fn inherited_t_requires_actual_transfer() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php
/** @template TD */
class D {}
/**
 * @template TC
 * @extends D<int>
 */
class C extends D {}",
        );
        let child_param = f.t_template("C", "TC");
        let parent_param = f.t_template("D", "TD");
        assert!(
            !atomic_is_contained(f, child_param, parent_param, &symbols),
            "C extends D<int>, not D<TC>, so TC and TD are unrelated"
        );
    });
}

#[test]
fn inherited_t_refines_implies_overlap() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php
/** @template TD */
class D {}
/**
 * @template TC
 * @extends D<TC>
 */
class C extends D {}",
        );
        let child_param = f.t_template("C", "TC");
        let parent_param = f.t_template("D", "TD");
        assert!(atomic_is_contained(f, child_param, parent_param, &symbols));
        assert!(
            atomic_overlaps(f, child_param, parent_param, &symbols),
            "TC <: TD must imply they overlap (refines/overlap consistency)"
        );
    });
}
