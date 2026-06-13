mod common;

use common::*;

#[test]
fn intersected_input_refines_each_conjunct() {
    fixture(|f| {
        let world = MockWorld::from_edges(&[]);
        let bar = f.t_named("Bar");
        let foo_and_bar = f.t_named_intersected("Foo", &[bar]);
        let foo = f.t_named("Foo");
        assert!(atomic_is_contained(f, foo_and_bar, foo, &world));
        assert!(atomic_is_contained(f, foo_and_bar, bar, &world));
    });
}

#[test]
fn intersected_input_does_not_refine_unrelated_class() {
    fixture(|f| {
        let world = MockWorld::from_edges(&[]);
        let bar = f.t_named("Bar");
        let foo_and_bar = f.t_named_intersected("Foo", &[bar]);
        let quux = f.t_named("Quux");
        assert!(!atomic_is_contained(f, foo_and_bar, quux, &world));
    });
}

#[test]
fn intersected_input_refines_ancestor_of_any_conjunct() {
    fixture(|f| {
        let world = MockWorld::from_edges(&[("Bar", "BarAncestor")]);
        let bar = f.t_named("Bar");
        let foo_and_bar = f.t_named_intersected("Foo", &[bar]);
        let bar_ancestor = f.t_named("BarAncestor");
        assert!(atomic_is_contained(f, foo_and_bar, bar_ancestor, &world));
    });
}

#[test]
fn input_must_refine_every_conjunct_of_intersected_container() {
    fixture(|f| {
        let world = MockWorld::from_edges(&[("Foo", "Bar"), ("Foo", "Baz")]);
        let baz = f.t_named("Baz");
        let bar_and_baz = f.t_named_intersected("Bar", &[baz]);
        let foo = f.t_named("Foo");
        assert!(atomic_is_contained(f, foo, bar_and_baz, &world));
    });
}

#[test]
fn input_missing_one_conjunct_fails_intersected_container() {
    fixture(|f| {
        let world = MockWorld::from_edges(&[("Foo", "Bar")]);
        let baz = f.t_named("Baz");
        let bar_and_baz = f.t_named_intersected("Bar", &[baz]);
        let foo = f.t_named("Foo");
        assert!(!atomic_is_contained(f, foo, bar_and_baz, &world));
    });
}

#[test]
fn intersected_input_into_intersected_container_both_directions() {
    fixture(|f| {
        let world = MockWorld::from_edges(&[]);
        let bar = f.t_named("Bar");
        let lhs = f.t_named_intersected("Foo", &[bar]);
        let rhs = f.t_named_intersected("Foo", &[bar]);
        assert!(atomic_is_contained(f, lhs, rhs, &world));
    });
}

#[test]
fn static_container_rejects_plain_named_input() {
    fixture(|f| {
        let world = MockWorld::from_edges(&[]);
        let plain_foo = f.t_named("Foo");
        let static_foo = f.t_named_static("Foo");
        assert!(!atomic_is_contained(f, plain_foo, static_foo, &world));
    });
}

#[test]
fn static_container_accepts_static_input() {
    fixture(|f| {
        let world = MockWorld::from_edges(&[]);
        let static_foo = f.t_named_static("Foo");
        assert!(atomic_is_contained(f, static_foo, static_foo, &world));
    });
}

#[test]
fn this_container_accepts_only_this_input() {
    fixture(|f| {
        let world = MockWorld::from_edges(&[]);
        let this_foo = f.t_named_this("Foo");
        let static_foo = f.t_named_static("Foo");
        let plain_foo = f.t_named("Foo");
        assert!(atomic_is_contained(f, this_foo, this_foo, &world));
        assert!(!atomic_is_contained(f, static_foo, this_foo, &world));
        assert!(!atomic_is_contained(f, plain_foo, this_foo, &world));
    });
}

#[test]
fn this_input_refines_static_container() {
    fixture(|f| {
        let world = MockWorld::from_edges(&[]);
        let this_foo = f.t_named_this("Foo");
        let static_foo = f.t_named_static("Foo");
        assert!(atomic_is_contained(f, this_foo, static_foo, &world));
    });
}

#[test]
fn static_input_refines_plain_named_container() {
    fixture(|f| {
        let world = MockWorld::from_edges(&[]);
        let static_foo = f.t_named_static("Foo");
        let plain_foo = f.t_named("Foo");
        assert!(atomic_is_contained(f, static_foo, plain_foo, &world));
    });
}
