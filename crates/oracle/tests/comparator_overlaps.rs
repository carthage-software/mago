mod common;

use common::*;

use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::well_known;

fn t_param_with_constraint<'arena>(
    f: &mut Fixture<'_, 'arena>,
    class: &str,
    name: &str,
    constraint: Type<'arena>,
) -> Atom<'arena> {
    f.t_template_of(class, name, constraint)
}

#[test]
fn reflexive_overlap() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let zero = f.t_lit_int(0);
        assert!(atomic_overlaps(f, int, int, &empty_world()));
        assert!(atomic_overlaps(f, string, string, &empty_world()));
        assert!(atomic_overlaps(f, zero, zero, &empty_world()));
    });
}

#[test]
fn never_disjoint_with_anything() {
    fixture(|f| {
        let never = f.never();
        let int = f.t_int();
        let mixed = f.mixed();
        assert!(!atomic_overlaps(f, never, int, &empty_world()));
        assert!(!atomic_overlaps(f, never, mixed, &empty_world()));
        assert!(!atomic_overlaps(f, never, never, &empty_world()));
    });
}

#[test]
fn mixed_overlaps_anything() {
    fixture(|f| {
        let mixed = f.mixed();
        let int = f.t_int();
        let null = f.null();
        let foo = f.t_named("Foo");
        assert!(atomic_overlaps(f, mixed, int, &empty_world()));
        assert!(atomic_overlaps(f, mixed, null, &empty_world()));
        assert!(atomic_overlaps(f, mixed, foo, &empty_world()));
    });
}

#[test]
fn placeholder_overlaps_anything() {
    fixture(|f| {
        let placeholder = f.placeholder();
        let int = f.t_int();
        let string = f.t_string();
        assert!(atomic_overlaps(f, placeholder, int, &empty_world()));
        assert!(atomic_overlaps(f, placeholder, string, &empty_world()));
    });
}

#[test]
fn subtype_implies_overlap() {
    fixture(|f| {
        let five = f.t_lit_int(5);
        let int = f.t_int();
        let true_atom = f.t_true();
        let bool_atom = f.t_bool();
        let array_key = f.t_array_key();
        assert!(atomic_overlaps(f, five, int, &empty_world()));
        assert!(atomic_overlaps(f, int, five, &empty_world()));
        assert!(atomic_overlaps(f, true_atom, bool_atom, &empty_world()));
        assert!(atomic_overlaps(f, int, array_key, &empty_world()));
    });
}

#[test]
fn distinct_kinds_disjoint() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let null = f.null();
        let resource = f.t_resource();
        let foo = f.t_named("Foo");
        assert!(!atomic_overlaps(f, int, string, &empty_world()));
        assert!(!atomic_overlaps(f, int, null, &empty_world()));
        assert!(!atomic_overlaps(f, null, string, &empty_world()));
        assert!(!atomic_overlaps(f, int, resource, &empty_world()));
        assert!(!atomic_overlaps(f, string, foo, &empty_world()));
    });
}

#[test]
fn int_and_float_are_disjoint() {
    fixture(|f| {
        let int = f.t_int();
        let float = f.t_float();
        assert!(!atomic_overlaps(f, int, float, &empty_world()));
    });
}

#[test]
fn int_ranges_overlap_when_intervals_intersect() {
    fixture(|f| {
        let a = f.t_int_range(0, 10);
        let b = f.t_int_range(5, 15);
        assert!(atomic_overlaps(f, a, b, &empty_world()));
        assert!(atomic_overlaps(f, b, a, &empty_world()));
    });
}

#[test]
fn int_ranges_disjoint_when_intervals_separate() {
    fixture(|f| {
        let a = f.t_int_range(0, 10);
        let b = f.t_int_range(20, 30);
        assert!(!atomic_overlaps(f, a, b, &empty_world()));
        assert!(!atomic_overlaps(f, b, a, &empty_world()));
    });
}

#[test]
fn touching_int_ranges_overlap_at_endpoint() {
    fixture(|f| {
        let a = f.t_int_range(0, 10);
        let b = f.t_int_range(10, 20);
        assert!(atomic_overlaps(f, a, b, &empty_world()));
    });
}

#[test]
fn open_lower_int_overlaps_open_upper_int() {
    fixture(|f| {
        let a = f.t_int_to(5);
        let b = f.t_int_from(0);
        assert!(atomic_overlaps(f, a, b, &empty_world()));
    });
}

#[test]
fn positive_int_disjoint_with_negative_int() {
    fixture(|f| {
        let positive = f.t_positive_int();
        let negative = f.t_negative_int();
        assert!(!atomic_overlaps(f, positive, negative, &empty_world()));
    });
}

#[test]
fn lit_int_overlaps_range_when_in_bounds() {
    fixture(|f| {
        let range = f.t_int_range(0, 10);
        let five = f.t_lit_int(5);
        assert!(atomic_overlaps(f, five, range, &empty_world()));
        assert!(atomic_overlaps(f, range, five, &empty_world()));
    });
}

#[test]
fn lit_int_disjoint_with_range_when_out_of_bounds() {
    fixture(|f| {
        let range = f.t_int_range(0, 10);
        let twenty = f.t_lit_int(20);
        assert!(!atomic_overlaps(f, twenty, range, &empty_world()));
    });
}

#[test]
fn distinct_int_literals_disjoint() {
    fixture(|f| {
        let one = f.t_lit_int(1);
        let two = f.t_lit_int(2);
        assert!(!atomic_overlaps(f, one, two, &empty_world()));
    });
}

#[test]
fn class_like_string_overlaps_string() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let string = f.t_string();
        let interface_string = f.t_interface_string();
        assert!(atomic_overlaps(f, class_string, string, &empty_world()));
        assert!(atomic_overlaps(f, string, interface_string, &empty_world()));
    });
}

#[test]
fn generic_parameter_overlaps_via_constraint() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let t = t_param_with_constraint(f, "Box", "T", int_type);
        let int = f.t_int();
        let five = f.t_lit_int(5);
        let string = f.t_string();
        assert!(atomic_overlaps(f, t, int, &empty_world()));
        assert!(atomic_overlaps(f, int, t, &empty_world()));
        assert!(atomic_overlaps(f, t, five, &empty_world()));
        assert!(!atomic_overlaps(f, t, string, &empty_world()));
    });
}

#[test]
fn unbounded_generic_parameter_overlaps_anything() {
    fixture(|f| {
        let t = f.t_template("Box", "T");
        let int = f.t_int();
        let string = f.t_string();
        let null = f.null();
        assert!(atomic_overlaps(f, t, int, &empty_world()));
        assert!(atomic_overlaps(f, t, string, &empty_world()));
        assert!(atomic_overlaps(f, t, null, &empty_world()));
    });
}

#[test]
fn union_overlap_distributes() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let float = f.t_float();
        let int_or_string = f.u_many(vec![int, string]);
        let string_or_float = f.u_many(vec![string, float]);
        assert!(overlaps(f, int_or_string, string_or_float, &empty_world()));
    });
}

#[test]
fn union_disjoint_when_no_pair_overlaps() {
    fixture(|f| {
        let int = f.t_int();
        let null = f.null();
        let string = f.t_string();
        let resource = f.t_resource();
        let int_or_null = f.u_many(vec![int, null]);
        let string_or_resource = f.u_many(vec![string, resource]);
        assert!(!overlaps(f, int_or_null, string_or_resource, &empty_world()));
    });
}

#[test]
fn nullable_overlap_via_null_branch() {
    fixture(|f| {
        let null = f.null();
        let int = f.t_int();
        let string = f.t_string();
        let nullable_int = f.u_many(vec![null, int]);
        let nullable_string = f.u_many(vec![null, string]);
        assert!(overlaps(f, nullable_int, nullable_string, &empty_world()));
    });
}

#[test]
fn truthy_mixed_overlaps_int() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        let int = f.t_int();
        assert!(atomic_overlaps(f, truthy, int, &empty_world()));
    });
}

#[test]
fn nonnull_mixed_disjoint_with_null() {
    fixture(|f| {
        let non_null = f.mixed_nonnull();
        let null = f.null();
        assert!(!atomic_overlaps(f, non_null, null, &empty_world()));
        assert!(!atomic_overlaps(f, null, non_null, &empty_world()));
    });
}

#[test]
fn truthy_mixed_disjoint_with_null() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        let null = f.null();
        let false_atom = f.t_false();
        let zero = f.t_lit_int(0);
        let empty_string = f.t_lit_string("");
        assert!(!atomic_overlaps(f, truthy, null, &empty_world()));
        assert!(!atomic_overlaps(f, truthy, false_atom, &empty_world()));
        assert!(!atomic_overlaps(f, truthy, zero, &empty_world()));
        assert!(!atomic_overlaps(f, truthy, empty_string, &empty_world()));
    });
}

#[test]
fn truthy_mixed_overlaps_truthy_inputs() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        let true_atom = f.t_true();
        let forty_two = f.t_lit_int(42);
        let foo = f.t_named("Foo");
        let resource = f.t_resource();
        assert!(atomic_overlaps(f, truthy, true_atom, &empty_world()));
        assert!(atomic_overlaps(f, truthy, forty_two, &empty_world()));
        assert!(atomic_overlaps(f, truthy, foo, &empty_world()));
        assert!(atomic_overlaps(f, truthy, resource, &empty_world()));
    });
}

#[test]
fn falsy_mixed_disjoint_with_truthy_inputs() {
    fixture(|f| {
        let falsy = f.mixed_falsy();
        let true_atom = f.t_true();
        let foo = f.t_named("Foo");
        let resource = f.t_resource();
        let forty_two = f.t_lit_int(42);
        assert!(!atomic_overlaps(f, falsy, true_atom, &empty_world()));
        assert!(!atomic_overlaps(f, falsy, foo, &empty_world()));
        assert!(!atomic_overlaps(f, falsy, resource, &empty_world()));
        assert!(!atomic_overlaps(f, falsy, forty_two, &empty_world()));
    });
}

#[test]
fn falsy_mixed_overlaps_falsy_inputs() {
    fixture(|f| {
        let falsy = f.mixed_falsy();
        let null = f.null();
        let false_atom = f.t_false();
        let zero = f.t_lit_int(0);
        let empty_string = f.t_lit_string("");
        assert!(atomic_overlaps(f, falsy, null, &empty_world()));
        assert!(atomic_overlaps(f, falsy, false_atom, &empty_world()));
        assert!(atomic_overlaps(f, falsy, zero, &empty_world()));
        assert!(atomic_overlaps(f, falsy, empty_string, &empty_world()));
    });
}

#[test]
fn truthy_mixed_disjoint_with_falsy_mixed() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        let falsy = f.mixed_falsy();
        assert!(!atomic_overlaps(f, truthy, falsy, &empty_world()));
    });
}

#[test]
fn truthy_mixed_overlaps_undetermined_inputs() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        let int = f.t_int();
        let string = f.t_string();
        let bool_atom = f.t_bool();
        assert!(atomic_overlaps(f, truthy, int, &empty_world()));
        assert!(atomic_overlaps(f, truthy, string, &empty_world()));
        assert!(atomic_overlaps(f, truthy, bool_atom, &empty_world()));
    });
}

#[test]
fn class_string_literal_overlaps_lit_class_string() {
    fixture(|f| {
        let literal = f.t_lit_class_string("Foo");
        let class_string = f.t_class_string();
        assert!(atomic_overlaps(f, literal, class_string, &empty_world()));
        assert!(atomic_overlaps(f, class_string, literal, &empty_world()));
    });
}

#[test]
fn never_type_disjoint_at_type_level() {
    fixture(|f| {
        assert!(!overlaps(f, well_known::TYPE_NEVER, well_known::TYPE_INT, &empty_world()));
    });
}

#[test]
fn unrelated_concrete_classes_do_not_overlap() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.declare("Foo");
        world.declare("Bar");
        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        assert!(!atomic_overlaps(f, foo, bar, &world), "single inheritance: no object is both Foo and Bar");
        assert!(!atomic_overlaps(f, bar, foo, &world));
    });
}

#[test]
fn concrete_class_overlaps_unrelated_interface() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.declare("Foo");
        world.declare_interface("Drawable");
        let foo = f.t_named("Foo");
        let drawable = f.t_named("Drawable");
        assert!(atomic_overlaps(f, foo, drawable, &world), "a subclass of Foo could implement Drawable");
        assert!(atomic_overlaps(f, drawable, foo, &world));
    });
}

#[test]
fn related_concrete_classes_overlap() {
    fixture(|f| {
        let mut world = MockWorld::new();
        world.add_edge("Dog", "Animal");
        let dog = f.t_named("Dog");
        let animal = f.t_named("Animal");
        assert!(atomic_overlaps(f, dog, animal, &world), "Dog is an Animal, so they share Dog instances");
    });
}

#[test]
fn unknown_kind_classes_stay_optimistic_overlap() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        assert!(
            atomic_overlaps(f, foo, bar, &empty_world()),
            "with no world knowledge of the kinds, overlap stays conservatively true"
        );
    });
}
