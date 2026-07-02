mod common;

use common::*;

use mago_oracle::ty::Atom;
use mago_oracle::ty::AtomKind;
use mago_oracle::ty::inspect;

#[test]
fn primitive_kinds_have_no_intersections_by_default() {
    fixture(|f| {
        for atom in [f.t_int(), f.t_string(), f.t_lit_int(42), f.null(), f.t_true(), f.t_false()] {
            assert_eq!(atom.intersection_types(), &[] as &[Atom<'_>]);
            assert!(!atom.has_intersection_types());
            assert!(atom.can_be_intersected());
        }
    });
}

#[test]
fn object_can_be_intersected_but_has_no_intersections_when_unset() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        assert!(foo.can_be_intersected());
        assert!(!foo.has_intersection_types());
        assert!(foo.intersection_types().is_empty());
    });
}

#[test]
fn object_with_intersections_returns_them() {
    fixture(|f| {
        let bar = f.t_named("Bar");
        let foo_and_bar = f.t_named_intersected("Foo", &[bar]);
        assert_eq!(foo_and_bar.kind(), AtomKind::Intersected);
        assert!(foo_and_bar.has_intersection_types());
        assert_eq!(foo_and_bar.intersection_types(), &[bar]);
    });
}

#[test]
fn has_method_intersection_via_wrapper() {
    fixture(|f| {
        let head = f.t_has_method("foo");
        let other = f.t_has_method("bar");
        let chained = f.builder.intersected(head, &[other]);

        assert_eq!(chained.kind(), AtomKind::Intersected);
        assert!(chained.has_intersection_types());
        assert_eq!(chained.intersection_types(), &[other]);
    });
}

#[test]
fn has_property_intersection_via_wrapper() {
    fixture(|f| {
        let head = f.t_has_property("x");
        let other = f.t_has_property("y");
        let chained = f.builder.intersected(head, &[other]);

        assert_eq!(chained.kind(), AtomKind::Intersected);
        assert!(chained.has_intersection_types());
        assert_eq!(chained.intersection_types(), &[other]);
    });
}

#[test]
fn object_shape_intersection_via_wrapper() {
    fixture(|f| {
        let int = f.t_int();
        let int_type = f.u(int);
        let head = f.t_object_shape(&[("a", int_type, false)], true);
        let other = f.t_has_method("doStuff");
        let chained = f.builder.intersected(head, &[other]);

        assert_eq!(chained.kind(), AtomKind::Intersected);
        assert_eq!(chained.intersection_types(), &[other]);
    });
}

#[test]
fn intersection_types_descend_via_inspect() {
    fixture(|f| {
        let inner_int_lit = f.t_lit_int(42);
        let inner_int_type = f.u(inner_int_lit);
        let inner_obj = f.t_generic_named("Marker", vec![inner_int_type]);
        let head = f.t_has_method("foo");
        let chained = f.builder.intersected(head, &[inner_obj]);

        let ty = f.u(chained);
        assert!(
            inspect::any(ty, |atom| atom == inner_int_lit),
            "inspect::any should reach into the Intersected wrapper's conjuncts"
        );
    });
}

#[test]
fn intersection_round_trips_through_serializable() {
    fixture(|f| {
        let head = f.t_has_method("foo");
        let conjunct = f.t_has_method("bar");
        let original = f.builder.intersected(head, &[conjunct]);

        let restored = original.to_serializable().build(&mut f.builder);
        assert_eq!(original, restored);
        assert_eq!(restored.intersection_types(), &[conjunct]);
    });
}
