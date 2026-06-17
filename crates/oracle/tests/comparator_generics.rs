mod common;

use common::*;

use mago_oracle::symbol::part::generic::Variance;
use mago_oracle::ty::lattice::CoercionCause;

#[test]
fn box_int_in_box_int_reflexive() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Invariant)]);

        let int_type = f.u(f.t_int());
        let lhs = f.t_generic_named("Box", vec![int_type]);
        let rhs = f.t_generic_named("Box", vec![int_type]);
        assert!(atomic_is_contained(f, lhs, rhs, &world));
    });
}

#[test]
fn box_int_not_in_box_scalar_invariant_default() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Invariant)]);

        let int_type = f.u(f.t_int());
        let scalar_type = f.u(f.t_scalar());
        let lhs = f.t_generic_named("Box", vec![int_type]);
        let rhs = f.t_generic_named("Box", vec![scalar_type]);
        assert!(!atomic_is_contained(f, lhs, rhs, &world));
        assert!(!atomic_is_contained(f, rhs, lhs, &world));
    });
}

#[test]
fn container_int_in_container_scalar_when_covariant() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Container", &[("T", Variance::Covariant)]);

        let int_type = f.u(f.t_int());
        let scalar_type = f.u(f.t_scalar());
        let lhs = f.t_generic_named("Container", vec![int_type]);
        let rhs = f.t_generic_named("Container", vec![scalar_type]);
        assert!(atomic_is_contained(f, lhs, rhs, &world));
        assert!(!atomic_is_contained(f, rhs, lhs, &world));
    });
}

#[test]
fn sink_scalar_in_sink_int_when_contravariant() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Sink", &[("T", Variance::Contravariant)]);

        let int_type = f.u(f.t_int());
        let scalar_type = f.u(f.t_scalar());
        let lhs = f.t_generic_named("Sink", vec![scalar_type]);
        let rhs = f.t_generic_named("Sink", vec![int_type]);
        assert!(atomic_is_contained(f, lhs, rhs, &world));
        assert!(!atomic_is_contained(f, rhs, lhs, &world));
    });
}

#[test]
fn cell_int_not_in_cell_scalar_exploit_rejected() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Cell", &[("T", Variance::Invariant)]);

        let int_type = f.u(f.t_int());
        let scalar_type = f.u(f.t_scalar());
        let cell_int = f.t_generic_named("Cell", vec![int_type]);
        let cell_scalar = f.t_generic_named("Cell", vec![scalar_type]);

        assert!(
            !atomic_is_contained(f, cell_int, cell_scalar, &world),
            "Cell<int> must NOT refine Cell<scalar> ; defaulting to covariant would let store_string() be called on a Cell<int>",
        );
    });
}

#[test]
fn descendant_with_concrete_parent_specialisation() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Invariant)]);
        world.declare("B");
        let string_type = f.u(f.t_string());
        world.with_extended("B", "A", vec![string_type]);

        let b = f.t_named("B");
        let int_type = f.u(f.t_int());
        let a_string = f.t_generic_named("A", vec![string_type]);
        let a_int = f.t_generic_named("A", vec![int_type]);

        assert!(atomic_is_contained(f, b, a_string, &world));
        assert!(!atomic_is_contained(f, b, a_int, &world));
    });
}

#[test]
fn descendant_threading_template() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Invariant)]);
        world.with_templates("B", &[("T", Variance::Invariant)]);
        let b_t = f.t_template("B", "T");
        let b_t_type = f.u(b_t);
        world.with_extended("B", "A", vec![b_t_type]);

        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let b_int = f.t_generic_named("B", vec![int_type]);
        let a_int = f.t_generic_named("A", vec![int_type]);
        let a_string = f.t_generic_named("A", vec![string_type]);

        assert!(atomic_is_contained(f, b_int, a_int, &world));
        assert!(!atomic_is_contained(f, b_int, a_string, &world));
    });
}

#[test]
fn descendant_with_template_in_nested_position() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Invariant)]);
        world.with_templates("B", &[("T", Variance::Invariant)]);

        let b_t = f.t_template("B", "T");
        let b_t_type = f.u(b_t);
        let list_of_b_t_atom = f.t_list(b_t_type, false);
        let list_of_b_t = f.u(list_of_b_t_atom);
        world.with_extended("B", "A", vec![list_of_b_t]);

        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let b_int = f.t_generic_named("B", vec![int_type]);
        let list_int_atom = f.t_list(int_type, false);
        let list_int = f.u(list_int_atom);
        let a_list_int = f.t_generic_named("A", vec![list_int]);
        let list_string_atom = f.t_list(string_type, false);
        let list_string = f.u(list_string_atom);
        let a_list_string = f.t_generic_named("A", vec![list_string]);

        assert!(atomic_is_contained(f, b_int, a_list_int, &world));
        assert!(!atomic_is_contained(f, b_int, a_list_string, &world));
    });
}

#[test]
fn multi_level_chain_with_explicit_grandparent_link() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Invariant)]);
        world.with_templates("B", &[("T", Variance::Invariant)]);
        world.with_templates("C", &[("U", Variance::Invariant)]);
        let b_t = f.t_template("B", "T");
        let b_t_type = f.u(b_t);
        world.with_extended("B", "A", vec![b_t_type]);
        let c_u = f.t_template("C", "U");
        let c_u_type = f.u(c_u);
        world.with_extended("C", "B", vec![c_u_type]);
        world.with_extended("C", "A", vec![c_u_type]);

        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let c_int = f.t_generic_named("C", vec![int_type]);
        let a_int = f.t_generic_named("A", vec![int_type]);
        let a_string = f.t_generic_named("A", vec![string_type]);

        assert!(atomic_is_contained(f, c_int, a_int, &world));
        assert!(!atomic_is_contained(f, c_int, a_string, &world));
    });
}

#[test]
fn covariant_descendant_widening() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Covariant)]);
        world.with_templates("B", &[("T", Variance::Covariant)]);
        let b_t = f.t_template("B", "T");
        let b_t_type = f.u(b_t);
        world.with_extended("B", "A", vec![b_t_type]);

        let int_type = f.u(f.t_int());
        let scalar_type = f.u(f.t_scalar());
        let b_int = f.t_generic_named("B", vec![int_type]);
        let a_scalar = f.t_generic_named("A", vec![scalar_type]);

        assert!(atomic_is_contained(f, b_int, a_scalar, &world));
    });
}

#[test]
fn unrelated_classes_disjoint_with_args() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Foo", &[("T", Variance::Invariant)]);
        world.with_templates("Bar", &[("T", Variance::Invariant)]);

        let int_type = f.u(f.t_int());
        let foo_int = f.t_generic_named("Foo", vec![int_type]);
        let bar_int = f.t_generic_named("Bar", vec![int_type]);

        assert!(!atomic_is_contained(f, foo_int, bar_int, &world));
        assert!(!atomic_is_contained(f, bar_int, foo_int, &world));
    });
}

#[test]
fn non_generic_named_into_non_generic_descendant() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.add_edge("Dog", "Animal");

        let dog = f.t_named("Dog");
        let animal = f.t_named("Animal");
        assert!(atomic_is_contained(f, dog, animal, &world));
        assert!(!atomic_is_contained(f, animal, dog, &world));
    });
}

#[test]
fn unspecialised_box_does_not_flow_into_box_int_under_invariance() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Invariant)]);

        let unspecialised = f.t_named("Box");
        let int_type = f.u(f.t_int());
        let box_int = f.t_generic_named("Box", vec![int_type]);
        assert!(!atomic_is_contained(f, unspecialised, box_int, &world));
    });
}

#[test]
fn box_int_flows_into_unspecialised_box_under_invariance() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Invariant)]);

        let int_type = f.u(f.t_int());
        let box_int = f.t_generic_named("Box", vec![int_type]);
        let unspecialised = f.t_named("Box");
        let (verdict, report) = atomic_is_contained_capturing(f, box_int, unspecialised, &world);
        assert!(verdict);
        assert!(report.causes.contains(CoercionCause::TemplateDefault));
    });
}

#[test]
fn unspec_to_unspec_box_passes_via_reflexivity() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Invariant)]);

        let lhs = f.t_named("Box");
        let rhs = f.t_named("Box");
        let (verdict, report) = atomic_is_contained_capturing(f, lhs, rhs, &world);
        assert!(verdict);
        assert!(!report.coerced());
    });
}

#[test]
fn nested_unspec_box_does_not_flow_into_box_int_under_invariance() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("Box", &[("T", Variance::Invariant)]);
        world.with_templates("Outer", &[("T", Variance::Invariant)]);

        let unspecialised_box = f.t_named("Box");
        let unspecialised_box_type = f.u(unspecialised_box);
        let outer_with_unspecialised_box = f.t_generic_named("Outer", vec![unspecialised_box_type]);
        let int_type = f.u(f.t_int());
        let box_int = f.t_generic_named("Box", vec![int_type]);
        let box_int_type = f.u(box_int);
        let outer_with_box_int = f.t_generic_named("Outer", vec![box_int_type]);

        assert!(!atomic_is_contained(f, outer_with_unspecialised_box, outer_with_box_int, &world));
    });
}

#[test]
fn descendant_reaches_generic_ancestor_without_own_templates() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.with_templates("A", &[("T", Variance::Invariant)]);
        world.declare("StringList");
        let string_type = f.u(f.t_string());
        world.with_extended("StringList", "A", vec![string_type]);

        let string_list = f.t_named("StringList");
        let a_string = f.t_generic_named("A", vec![string_type]);
        let int_type = f.u(f.t_int());
        let a_int = f.t_generic_named("A", vec![int_type]);

        assert!(atomic_is_contained(f, string_list, a_string, &world));
        assert!(!atomic_is_contained(f, string_list, a_int, &world));
    });
}
