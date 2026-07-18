mod common;

use common::*;

use mago_oracle::ty::lattice::CoercionCause;

#[test]
fn box_int_in_box_int_reflexive() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template T */ class Box {}");

        let int_type = f.u(f.t_int());
        let lhs = f.t_generic_named("Box", vec![int_type]);
        let rhs = f.t_generic_named("Box", vec![int_type]);
        assert!(atomic_is_contained(f, lhs, rhs, &symbols));
    });
}

#[test]
fn box_int_not_in_box_scalar_invariant_default() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template T */ class Box {}");

        let int_type = f.u(f.t_int());
        let scalar_type = f.u(f.t_scalar());
        let lhs = f.t_generic_named("Box", vec![int_type]);
        let rhs = f.t_generic_named("Box", vec![scalar_type]);
        assert!(!atomic_is_contained(f, lhs, rhs, &symbols));
        assert!(!atomic_is_contained(f, rhs, lhs, &symbols));
    });
}

#[test]
fn container_int_in_container_scalar_when_covariant() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template-covariant T */ class Container {}");

        let int_type = f.u(f.t_int());
        let scalar_type = f.u(f.t_scalar());
        let lhs = f.t_generic_named("Container", vec![int_type]);
        let rhs = f.t_generic_named("Container", vec![scalar_type]);
        assert!(atomic_is_contained(f, lhs, rhs, &symbols));
        assert!(!atomic_is_contained(f, rhs, lhs, &symbols));
    });
}

#[test]
fn sink_scalar_in_sink_int_when_contravariant() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template-contravariant T */ class Sink {}");

        let int_type = f.u(f.t_int());
        let scalar_type = f.u(f.t_scalar());
        let lhs = f.t_generic_named("Sink", vec![scalar_type]);
        let rhs = f.t_generic_named("Sink", vec![int_type]);
        assert!(atomic_is_contained(f, lhs, rhs, &symbols));
        assert!(!atomic_is_contained(f, rhs, lhs, &symbols));
    });
}

#[test]
fn cell_int_not_in_cell_scalar_exploit_rejected() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template T */ class Cell {}");

        let int_type = f.u(f.t_int());
        let scalar_type = f.u(f.t_scalar());
        let cell_int = f.t_generic_named("Cell", vec![int_type]);
        let cell_scalar = f.t_generic_named("Cell", vec![scalar_type]);

        assert!(
            !atomic_is_contained(f, cell_int, cell_scalar, &symbols),
            "Cell<int> must NOT refine Cell<scalar> ; defaulting to covariant would let store_string() be called on a Cell<int>",
        );
    });
}

#[test]
fn descendant_with_concrete_parent_specialisation() {
    fixture(|f| {
        let symbols =
            symbol_table(f.arena, "<?php /** @template T */ class A {} /** @extends A<string> */ class B extends A {}");

        let b = f.t_named("B");
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let a_string = f.t_generic_named("A", vec![string_type]);
        let a_int = f.t_generic_named("A", vec![int_type]);

        assert!(atomic_is_contained(f, b, a_string, &symbols));
        assert!(!atomic_is_contained(f, b, a_int, &symbols));
    });
}

#[test]
fn descendant_threading_template() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php
/** @template T */
class A {}
/**
 * @template T
 * @extends A<T>
 */
class B extends A {}",
        );

        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let b_int = f.t_generic_named("B", vec![int_type]);
        let a_int = f.t_generic_named("A", vec![int_type]);
        let a_string = f.t_generic_named("A", vec![string_type]);

        assert!(atomic_is_contained(f, b_int, a_int, &symbols));
        assert!(!atomic_is_contained(f, b_int, a_string, &symbols));
    });
}

#[test]
fn descendant_with_template_in_nested_position() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php
/** @template T */
class A {}
/**
 * @template T
 * @extends A<list<T>>
 */
class B extends A {}",
        );

        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let b_int = f.t_generic_named("B", vec![int_type]);
        let list_int_atom = f.t_list(int_type, false);
        let list_int = f.u(list_int_atom);
        let a_list_int = f.t_generic_named("A", vec![list_int]);
        let list_string_atom = f.t_list(string_type, false);
        let list_string = f.u(list_string_atom);
        let a_list_string = f.t_generic_named("A", vec![list_string]);

        assert!(atomic_is_contained(f, b_int, a_list_int, &symbols));
        assert!(!atomic_is_contained(f, b_int, a_list_string, &symbols));
    });
}

#[test]
fn multi_level_chain_with_explicit_grandparent_link() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php
/** @template T */
class A {}
/**
 * @template T
 * @extends A<T>
 */
class B extends A {}
/**
 * @template U
 * @extends B<U>
 */
class C extends B {}",
        );

        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let c_int = f.t_generic_named("C", vec![int_type]);
        let a_int = f.t_generic_named("A", vec![int_type]);
        let a_string = f.t_generic_named("A", vec![string_type]);

        assert!(atomic_is_contained(f, c_int, a_int, &symbols));
        assert!(!atomic_is_contained(f, c_int, a_string, &symbols));
    });
}

#[test]
fn covariant_descendant_widening() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php
/** @template-covariant T */
class A {}
/**
 * @template-covariant T
 * @extends A<T>
 */
class B extends A {}",
        );

        let int_type = f.u(f.t_int());
        let scalar_type = f.u(f.t_scalar());
        let b_int = f.t_generic_named("B", vec![int_type]);
        let a_scalar = f.t_generic_named("A", vec![scalar_type]);

        assert!(atomic_is_contained(f, b_int, a_scalar, &symbols));
    });
}

#[test]
fn unrelated_classes_disjoint_with_args() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template T */ class Foo {} /** @template T */ class Bar {}");

        let int_type = f.u(f.t_int());
        let foo_int = f.t_generic_named("Foo", vec![int_type]);
        let bar_int = f.t_generic_named("Bar", vec![int_type]);

        assert!(!atomic_is_contained(f, foo_int, bar_int, &symbols));
        assert!(!atomic_is_contained(f, bar_int, foo_int, &symbols));
    });
}

#[test]
fn non_generic_named_into_non_generic_descendant() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php class Animal {} class Dog extends Animal {}");

        let dog = f.t_named("Dog");
        let animal = f.t_named("Animal");
        assert!(atomic_is_contained(f, dog, animal, &symbols));
        assert!(!atomic_is_contained(f, animal, dog, &symbols));
    });
}

#[test]
fn unspecialised_box_does_not_flow_into_box_int_under_invariance() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template T */ class Box {}");

        let unspecialised = f.t_named("Box");
        let int_type = f.u(f.t_int());
        let box_int = f.t_generic_named("Box", vec![int_type]);
        assert!(!atomic_is_contained(f, unspecialised, box_int, &symbols));
    });
}

#[test]
fn box_int_flows_into_unspecialised_box_under_invariance() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template T */ class Box {}");

        let int_type = f.u(f.t_int());
        let box_int = f.t_generic_named("Box", vec![int_type]);
        let unspecialised = f.t_named("Box");
        let (verdict, report) = atomic_is_contained_capturing(f, box_int, unspecialised, &symbols);
        assert!(verdict);
        assert!(report.causes.contains(CoercionCause::TemplateDefault));
    });
}

#[test]
fn unspec_to_unspec_box_passes_via_reflexivity() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template T */ class Box {}");

        let lhs = f.t_named("Box");
        let rhs = f.t_named("Box");
        let (verdict, report) = atomic_is_contained_capturing(f, lhs, rhs, &symbols);
        assert!(verdict);
        assert!(!report.coerced());
    });
}

#[test]
fn nested_unspec_box_does_not_flow_into_box_int_under_invariance() {
    fixture(|f| {
        let symbols = symbol_table(f.arena, "<?php /** @template T */ class Box {} /** @template T */ class Outer {}");

        let unspecialised_box = f.t_named("Box");
        let unspecialised_box_type = f.u(unspecialised_box);
        let outer_with_unspecialised_box = f.t_generic_named("Outer", vec![unspecialised_box_type]);
        let int_type = f.u(f.t_int());
        let box_int = f.t_generic_named("Box", vec![int_type]);
        let box_int_type = f.u(box_int);
        let outer_with_box_int = f.t_generic_named("Outer", vec![box_int_type]);

        assert!(!atomic_is_contained(f, outer_with_unspecialised_box, outer_with_box_int, &symbols));
    });
}

#[test]
fn descendant_reaches_generic_ancestor_without_own_templates() {
    fixture(|f| {
        let symbols = symbol_table(
            f.arena,
            "<?php /** @template T */ class A {} /** @extends A<string> */ class StringList extends A {}",
        );

        let string_list = f.t_named("StringList");
        let string_type = f.u(f.t_string());
        let a_string = f.t_generic_named("A", vec![string_type]);
        let int_type = f.u(f.t_int());
        let a_int = f.t_generic_named("A", vec![int_type]);

        assert!(atomic_is_contained(f, string_list, a_string, &symbols));
        assert!(!atomic_is_contained(f, string_list, a_int, &symbols));
    });
}
