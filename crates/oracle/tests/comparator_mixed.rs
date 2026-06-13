mod common;

use common::*;

#[test]
fn mixed_reflexive() {
    fixture(|f| {
        let mixed = f.mixed();
        assert_atomic_subtype(f, mixed, mixed);
    });
}

#[test]
fn truthy_mixed_reflexive() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        assert_atomic_subtype(f, truthy, truthy);
    });
}

#[test]
fn falsy_mixed_reflexive() {
    fixture(|f| {
        let falsy = f.mixed_falsy();
        assert_atomic_subtype(f, falsy, falsy);
    });
}

#[test]
fn nonnull_mixed_reflexive() {
    fixture(|f| {
        let non_null = f.mixed_nonnull();
        assert_atomic_subtype(f, non_null, non_null);
    });
}

#[test]
fn truthy_mixed_in_mixed() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        let mixed = f.mixed();
        assert_atomic_subtype(f, truthy, mixed);
    });
}

#[test]
fn falsy_mixed_in_mixed() {
    fixture(|f| {
        let falsy = f.mixed_falsy();
        let mixed = f.mixed();
        assert_atomic_subtype(f, falsy, mixed);
    });
}

#[test]
fn nonnull_mixed_in_mixed() {
    fixture(|f| {
        let non_null = f.mixed_nonnull();
        let mixed = f.mixed();
        assert_atomic_subtype(f, non_null, mixed);
    });
}

#[test]
fn int_in_mixed() {
    fixture(|f| {
        let int = f.t_int();
        let mixed = f.mixed();
        assert_atomic_subtype(f, int, mixed);
    });
}

#[test]
fn string_in_mixed() {
    fixture(|f| {
        let string = f.t_string();
        let mixed = f.mixed();
        assert_atomic_subtype(f, string, mixed);
    });
}

#[test]
fn float_in_mixed() {
    fixture(|f| {
        let float = f.t_float();
        let mixed = f.mixed();
        assert_atomic_subtype(f, float, mixed);
    });
}

#[test]
fn bool_in_mixed() {
    fixture(|f| {
        let bool_atom = f.t_bool();
        let mixed = f.mixed();
        assert_atomic_subtype(f, bool_atom, mixed);
    });
}

#[test]
fn null_in_mixed() {
    fixture(|f| {
        let null = f.null();
        let mixed = f.mixed();
        assert_atomic_subtype(f, null, mixed);
    });
}

#[test]
fn void_in_mixed() {
    fixture(|f| {
        let void = f.void();
        let mixed = f.mixed();
        assert_atomic_subtype(f, void, mixed);
    });
}

#[test]
fn object_in_mixed() {
    fixture(|f| {
        let object = f.t_object_any();
        let foo = f.t_named("Foo");
        let mixed = f.mixed();
        assert_atomic_subtype(f, object, mixed);
        assert_atomic_subtype(f, foo, mixed);
    });
}

#[test]
fn array_in_mixed() {
    fixture(|f| {
        let empty_array = f.t_empty_array();
        let mixed = f.mixed();
        assert_atomic_subtype(f, empty_array, mixed);
    });
}

#[test]
fn resource_in_mixed() {
    fixture(|f| {
        let resource = f.t_resource();
        let mixed = f.mixed();
        assert_atomic_subtype(f, resource, mixed);
    });
}

#[test]
fn mixed_not_in_int() {
    fixture(|f| {
        let mixed = f.mixed();
        let int = f.t_int();
        assert_atomic_not_subtype(f, mixed, int);
    });
}

#[test]
fn mixed_not_in_string() {
    fixture(|f| {
        let mixed = f.mixed();
        let string = f.t_string();
        assert_atomic_not_subtype(f, mixed, string);
    });
}

#[test]
fn mixed_not_in_float() {
    fixture(|f| {
        let mixed = f.mixed();
        let float = f.t_float();
        assert_atomic_not_subtype(f, mixed, float);
    });
}

#[test]
fn mixed_not_in_bool() {
    fixture(|f| {
        let mixed = f.mixed();
        let bool_atom = f.t_bool();
        assert_atomic_not_subtype(f, mixed, bool_atom);
    });
}

#[test]
fn mixed_not_in_null() {
    fixture(|f| {
        let mixed = f.mixed();
        let null = f.null();
        assert_atomic_not_subtype(f, mixed, null);
    });
}

#[test]
fn mixed_not_in_object() {
    fixture(|f| {
        let mixed = f.mixed();
        let object = f.t_object_any();
        assert_atomic_not_subtype(f, mixed, object);
    });
}

#[test]
fn mixed_not_in_array() {
    fixture(|f| {
        let mixed = f.mixed();
        let empty_array = f.t_empty_array();
        assert_atomic_not_subtype(f, mixed, empty_array);
    });
}

#[test]
fn never_in_mixed_variants() {
    fixture(|f| {
        let never = f.never();
        let mixed = f.mixed();
        let truthy = f.mixed_truthy();
        let falsy = f.mixed_falsy();
        let non_null = f.mixed_nonnull();
        assert_atomic_subtype(f, never, mixed);
        assert_atomic_subtype(f, never, truthy);
        assert_atomic_subtype(f, never, falsy);
        assert_atomic_subtype(f, never, non_null);
    });
}

#[test]
fn null_not_in_nonnull_mixed() {
    fixture(|f| {
        let null = f.null();
        let non_null = f.mixed_nonnull();
        assert_atomic_not_subtype(f, null, non_null);
    });
}

#[test]
fn int_in_nonnull_mixed() {
    fixture(|f| {
        let int = f.t_int();
        let zero = f.t_lit_int(0);
        let forty_two = f.t_lit_int(42);
        let non_null = f.mixed_nonnull();
        assert_atomic_subtype(f, int, non_null);
        assert_atomic_subtype(f, zero, non_null);
        assert_atomic_subtype(f, forty_two, non_null);
    });
}

#[test]
fn string_in_nonnull_mixed() {
    fixture(|f| {
        let string = f.t_string();
        let empty = f.t_lit_string("");
        let non_null = f.mixed_nonnull();
        assert_atomic_subtype(f, string, non_null);
        assert_atomic_subtype(f, empty, non_null);
    });
}

#[test]
fn object_in_nonnull_mixed() {
    fixture(|f| {
        let object = f.t_object_any();
        let foo = f.t_named("Foo");
        let non_null = f.mixed_nonnull();
        assert_atomic_subtype(f, object, non_null);
        assert_atomic_subtype(f, foo, non_null);
    });
}

#[test]
fn vanilla_mixed_not_in_nonnull_mixed() {
    fixture(|f| {
        let mixed = f.mixed();
        let non_null = f.mixed_nonnull();
        assert_atomic_not_subtype(f, mixed, non_null);
    });
}

#[test]
fn nonnull_mixed_in_nonnull_mixed() {
    fixture(|f| {
        let non_null = f.mixed_nonnull();
        assert_atomic_subtype(f, non_null, non_null);
    });
}

#[test]
fn truthy_mixed_in_nonnull_mixed() {
    fixture(|f| {
        let truthy = f.mixed_truthy();
        let non_null = f.mixed_nonnull();
        assert_atomic_subtype(f, truthy, non_null);
    });
}

#[test]
fn true_in_truthy_mixed() {
    fixture(|f| {
        let true_atom = f.t_true();
        let truthy = f.mixed_truthy();
        assert_atomic_subtype(f, true_atom, truthy);
    });
}

#[test]
fn false_not_in_truthy_mixed() {
    fixture(|f| {
        let false_atom = f.t_false();
        let truthy = f.mixed_truthy();
        assert_atomic_not_subtype(f, false_atom, truthy);
    });
}

#[test]
fn bool_not_in_truthy_or_falsy_mixed() {
    fixture(|f| {
        let bool_atom = f.t_bool();
        let truthy = f.mixed_truthy();
        let falsy = f.mixed_falsy();
        assert_atomic_not_subtype(f, bool_atom, truthy);
        assert_atomic_not_subtype(f, bool_atom, falsy);
    });
}

#[test]
fn null_in_falsy_mixed() {
    fixture(|f| {
        let null = f.null();
        let falsy = f.mixed_falsy();
        assert_atomic_subtype(f, null, falsy);
    });
}

#[test]
fn null_not_in_truthy_mixed() {
    fixture(|f| {
        let null = f.null();
        let truthy = f.mixed_truthy();
        assert_atomic_not_subtype(f, null, truthy);
    });
}

#[test]
fn lit_int_truthiness() {
    fixture(|f| {
        let zero = f.t_lit_int(0);
        let forty_two = f.t_lit_int(42);
        let minus_one = f.t_lit_int(-1);
        let truthy = f.mixed_truthy();
        let falsy = f.mixed_falsy();
        assert_atomic_subtype(f, zero, falsy);
        assert_atomic_not_subtype(f, zero, truthy);
        assert_atomic_subtype(f, forty_two, truthy);
        assert_atomic_subtype(f, minus_one, truthy);
        assert_atomic_not_subtype(f, forty_two, falsy);
    });
}

#[test]
fn int_general_truthiness_undetermined() {
    fixture(|f| {
        let int = f.t_int();
        let truthy = f.mixed_truthy();
        let falsy = f.mixed_falsy();
        assert_atomic_not_subtype(f, int, truthy);
        assert_atomic_not_subtype(f, int, falsy);
    });
}

#[test]
fn positive_int_in_truthy_mixed() {
    fixture(|f| {
        let positive = f.t_positive_int();
        let negative = f.t_negative_int();
        let truthy = f.mixed_truthy();
        assert_atomic_subtype(f, positive, truthy);
        assert_atomic_subtype(f, negative, truthy);
    });
}

#[test]
fn lit_float_truthiness() {
    fixture(|f| {
        let zero = f.t_lit_float(0.0);
        let positive = f.t_lit_float(1.5);
        let negative = f.t_lit_float(-2.5);
        let truthy = f.mixed_truthy();
        let falsy = f.mixed_falsy();
        assert_atomic_subtype(f, zero, falsy);
        assert_atomic_subtype(f, positive, truthy);
        assert_atomic_subtype(f, negative, truthy);
    });
}

#[test]
fn float_general_truthiness_undetermined() {
    fixture(|f| {
        let float = f.t_float();
        let truthy = f.mixed_truthy();
        let falsy = f.mixed_falsy();
        assert_atomic_not_subtype(f, float, truthy);
        assert_atomic_not_subtype(f, float, falsy);
    });
}

#[test]
fn lit_string_truthiness() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        let zero = f.t_lit_string("0");
        let hi = f.t_lit_string("hi");
        let false_word = f.t_lit_string("false");
        let truthy = f.mixed_truthy();
        let falsy = f.mixed_falsy();
        assert_atomic_subtype(f, empty, falsy);
        assert_atomic_subtype(f, zero, falsy);
        assert_atomic_subtype(f, hi, truthy);
        assert_atomic_subtype(f, false_word, truthy);
        assert_atomic_not_subtype(f, hi, falsy);
        assert_atomic_not_subtype(f, empty, truthy);
    });
}

#[test]
fn truthy_string_in_truthy_mixed() {
    fixture(|f| {
        let truthy_string = f.t_truthy_string();
        let truthy = f.mixed_truthy();
        assert_atomic_subtype(f, truthy_string, truthy);
    });
}

#[test]
fn general_string_truthiness_undetermined() {
    fixture(|f| {
        let string = f.t_string();
        let truthy = f.mixed_truthy();
        let falsy = f.mixed_falsy();
        assert_atomic_not_subtype(f, string, truthy);
        assert_atomic_not_subtype(f, string, falsy);
    });
}

#[test]
fn objects_are_truthy() {
    fixture(|f| {
        let object = f.t_object_any();
        let foo = f.t_named("Foo");
        let truthy = f.mixed_truthy();
        let falsy = f.mixed_falsy();
        assert_atomic_subtype(f, object, truthy);
        assert_atomic_subtype(f, foo, truthy);
        assert_atomic_not_subtype(f, object, falsy);
    });
}

#[test]
fn resources_are_truthy() {
    fixture(|f| {
        let resource = f.t_resource();
        let open_resource = f.t_open_resource();
        let truthy = f.mixed_truthy();
        assert_atomic_subtype(f, resource, truthy);
        assert_atomic_subtype(f, open_resource, truthy);
    });
}

#[test]
fn class_like_strings_are_truthy() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let literal = f.t_lit_class_string("Foo");
        let truthy = f.mixed_truthy();
        assert_atomic_subtype(f, class_string, truthy);
        assert_atomic_subtype(f, literal, truthy);
    });
}

#[test]
fn empty_array_is_falsy() {
    fixture(|f| {
        let empty_array = f.t_empty_array();
        let falsy = f.mixed_falsy();
        let truthy = f.mixed_truthy();
        assert_atomic_subtype(f, empty_array, falsy);
        assert_atomic_not_subtype(f, empty_array, truthy);
    });
}

#[test]
fn non_empty_list_is_truthy() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let list = f.t_list(int_type, true);
        let truthy = f.mixed_truthy();
        assert_atomic_subtype(f, list, truthy);
    });
}

#[test]
fn general_list_truthiness_undetermined() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let list = f.t_list(int_type, false);
        let truthy = f.mixed_truthy();
        let falsy = f.mixed_falsy();
        assert_atomic_not_subtype(f, list, truthy);
        assert_atomic_not_subtype(f, list, falsy);
    });
}
