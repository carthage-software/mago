mod common;

use common::*;

#[test]
fn pp_int_int() {
    fixture(|f| {
        let int = f.t_int();
        assert_atomic_subtype(f, int, int);
    });
}

#[test]
fn pp_string_string() {
    fixture(|f| {
        let string = f.t_string();
        assert_atomic_subtype(f, string, string);
    });
}

#[test]
fn pp_float_float() {
    fixture(|f| {
        let float = f.t_float();
        assert_atomic_subtype(f, float, float);
    });
}

#[test]
fn pp_bool_bool() {
    fixture(|f| {
        let bool_atom = f.t_bool();
        assert_atomic_subtype(f, bool_atom, bool_atom);
    });
}

#[test]
fn pp_true_true() {
    fixture(|f| {
        let true_atom = f.t_true();
        assert_atomic_subtype(f, true_atom, true_atom);
    });
}

#[test]
fn pp_false_false() {
    fixture(|f| {
        let false_atom = f.t_false();
        assert_atomic_subtype(f, false_atom, false_atom);
    });
}

#[test]
fn pp_null_null() {
    fixture(|f| {
        let null = f.null();
        assert_atomic_subtype(f, null, null);
    });
}

#[test]
fn pp_void_void() {
    fixture(|f| {
        let void = f.void();
        assert_atomic_subtype(f, void, void);
    });
}

#[test]
fn pp_never_never() {
    fixture(|f| {
        let never = f.never();
        assert_atomic_subtype(f, never, never);
    });
}

#[test]
fn pp_mixed_mixed() {
    fixture(|f| {
        let mixed = f.mixed();
        assert_atomic_subtype(f, mixed, mixed);
    });
}

#[test]
fn pp_object_object() {
    fixture(|f| {
        let object = f.t_object_any();
        assert_atomic_subtype(f, object, object);
    });
}

#[test]
fn pp_resource_resource() {
    fixture(|f| {
        let resource = f.t_resource();
        assert_atomic_subtype(f, resource, resource);
    });
}

#[test]
fn pp_array_key_array_key() {
    fixture(|f| {
        let array_key = f.t_array_key();
        assert_atomic_subtype(f, array_key, array_key);
    });
}

#[test]
fn pp_numeric_numeric() {
    fixture(|f| {
        let numeric = f.t_numeric();
        assert_atomic_subtype(f, numeric, numeric);
    });
}

#[test]
fn pp_scalar_scalar() {
    fixture(|f| {
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, scalar, scalar);
    });
}

#[test]
fn pp_true_in_bool() {
    fixture(|f| {
        let true_atom = f.t_true();
        let bool_atom = f.t_bool();
        assert_atomic_subtype(f, true_atom, bool_atom);
    });
}

#[test]
fn pp_false_in_bool() {
    fixture(|f| {
        let false_atom = f.t_false();
        let bool_atom = f.t_bool();
        assert_atomic_subtype(f, false_atom, bool_atom);
    });
}

#[test]
fn pp_bool_not_in_true() {
    fixture(|f| {
        let bool_atom = f.t_bool();
        let true_atom = f.t_true();
        assert_atomic_not_subtype(f, bool_atom, true_atom);
    });
}

#[test]
fn pp_bool_not_in_false() {
    fixture(|f| {
        let bool_atom = f.t_bool();
        let false_atom = f.t_false();
        assert_atomic_not_subtype(f, bool_atom, false_atom);
    });
}

#[test]
fn pp_true_not_in_false() {
    fixture(|f| {
        let true_atom = f.t_true();
        let false_atom = f.t_false();
        assert_atomic_not_subtype(f, true_atom, false_atom);
    });
}

#[test]
fn pp_false_not_in_true() {
    fixture(|f| {
        let false_atom = f.t_false();
        let true_atom = f.t_true();
        assert_atomic_not_subtype(f, false_atom, true_atom);
    });
}

#[test]
fn pp_lit_in_int_0() {
    fixture(|f| {
        let literal = f.t_lit_int(0);
        let int = f.t_int();
        assert_atomic_subtype(f, literal, int);
    });
}

#[test]
fn pp_lit_in_int_1() {
    fixture(|f| {
        let literal = f.t_lit_int(1);
        let int = f.t_int();
        assert_atomic_subtype(f, literal, int);
    });
}

#[test]
fn pp_lit_in_int_neg() {
    fixture(|f| {
        let literal = f.t_lit_int(-1);
        let int = f.t_int();
        assert_atomic_subtype(f, literal, int);
    });
}

#[test]
fn pp_int_not_in_lit() {
    fixture(|f| {
        let int = f.t_int();
        let literal = f.t_lit_int(5);
        assert_atomic_not_subtype(f, int, literal);
    });
}

#[test]
fn pp_lit_disjoint() {
    fixture(|f| {
        let one = f.t_lit_int(1);
        let two = f.t_lit_int(2);
        assert_atomic_not_subtype(f, one, two);
    });
}

#[test]
fn pp_lit_eq() {
    fixture(|f| {
        let five = f.t_lit_int(5);
        assert_atomic_subtype(f, five, five);
    });
}

#[test]
fn pp_pos_in_int() {
    fixture(|f| {
        let positive = f.t_positive_int();
        let int = f.t_int();
        assert_atomic_subtype(f, positive, int);
    });
}

#[test]
fn pp_neg_in_int() {
    fixture(|f| {
        let negative = f.t_negative_int();
        let int = f.t_int();
        assert_atomic_subtype(f, negative, int);
    });
}

#[test]
fn pp_nn_in_int() {
    fixture(|f| {
        let non_negative = f.t_non_negative_int();
        let int = f.t_int();
        assert_atomic_subtype(f, non_negative, int);
    });
}

#[test]
fn pp_np_in_int() {
    fixture(|f| {
        let non_positive = f.t_non_positive_int();
        let int = f.t_int();
        assert_atomic_subtype(f, non_positive, int);
    });
}

#[test]
fn pp_int_not_pos() {
    fixture(|f| {
        let int = f.t_int();
        let positive = f.t_positive_int();
        assert_atomic_not_subtype(f, int, positive);
    });
}

#[test]
fn pp_int_not_neg() {
    fixture(|f| {
        let int = f.t_int();
        let negative = f.t_negative_int();
        assert_atomic_not_subtype(f, int, negative);
    });
}

#[test]
fn pp_pos_in_nn() {
    fixture(|f| {
        let positive = f.t_positive_int();
        let non_negative = f.t_non_negative_int();
        assert_atomic_subtype(f, positive, non_negative);
    });
}

#[test]
fn pp_neg_in_np() {
    fixture(|f| {
        let negative = f.t_negative_int();
        let non_positive = f.t_non_positive_int();
        assert_atomic_subtype(f, negative, non_positive);
    });
}

#[test]
fn pp_pos_neg_disjoint() {
    fixture(|f| {
        let positive = f.t_positive_int();
        let negative = f.t_negative_int();
        assert_atomic_not_subtype(f, positive, negative);
    });
}

#[test]
fn pp_range_05_in_range_010() {
    fixture(|f| {
        let narrow = f.t_int_range(0, 5);
        let wide = f.t_int_range(0, 10);
        assert_atomic_subtype(f, narrow, wide);
    });
}

#[test]
fn pp_range_010_not_in_range_05() {
    fixture(|f| {
        let wide = f.t_int_range(0, 10);
        let narrow = f.t_int_range(0, 5);
        assert_atomic_not_subtype(f, wide, narrow);
    });
}

#[test]
fn pp_range_05_in_pos() {
    fixture(|f| {
        let range = f.t_int_range(0, 5);
        let positive = f.t_positive_int();
        assert_atomic_not_subtype(f, range, positive);
    });
}

#[test]
fn pp_range_15_in_pos() {
    fixture(|f| {
        let range = f.t_int_range(1, 5);
        let positive = f.t_positive_int();
        assert_atomic_subtype(f, range, positive);
    });
}

#[test]
fn pp_lit5_in_range010() {
    fixture(|f| {
        let five = f.t_lit_int(5);
        let range = f.t_int_range(0, 10);
        assert_atomic_subtype(f, five, range);
    });
}

#[test]
fn pp_lit15_not_in_range010() {
    fixture(|f| {
        let fifteen = f.t_lit_int(15);
        let range = f.t_int_range(0, 10);
        assert_atomic_not_subtype(f, fifteen, range);
    });
}

#[test]
fn pp_from5_in_pos() {
    fixture(|f| {
        let from_five = f.t_int_from(5);
        let positive = f.t_positive_int();
        assert_atomic_subtype(f, from_five, positive);
    });
}

#[test]
fn pp_from0_not_in_pos() {
    fixture(|f| {
        let from_zero = f.t_int_from(0);
        let positive = f.t_positive_int();
        assert_atomic_not_subtype(f, from_zero, positive);
    });
}

#[test]
fn pp_from0_in_nn() {
    fixture(|f| {
        let from_zero = f.t_int_from(0);
        let non_negative = f.t_non_negative_int();
        assert_atomic_subtype(f, from_zero, non_negative);
    });
}

#[test]
fn pp_to_neg1_in_neg() {
    fixture(|f| {
        let to_minus_one = f.t_int_to(-1);
        let negative = f.t_negative_int();
        assert_atomic_subtype(f, to_minus_one, negative);
    });
}

#[test]
fn pp_to0_in_np() {
    fixture(|f| {
        let to_zero = f.t_int_to(0);
        let non_positive = f.t_non_positive_int();
        assert_atomic_subtype(f, to_zero, non_positive);
    });
}

#[test]
fn pp_unspec_lit_in_int() {
    fixture(|f| {
        let literal_int = f.t_int_unspec_lit();
        let int = f.t_int();
        assert_atomic_subtype(f, literal_int, int);
    });
}

#[test]
fn pp_int_not_in_unspec_lit() {
    fixture(|f| {
        let int = f.t_int();
        let literal_int = f.t_int_unspec_lit();
        assert_atomic_not_subtype(f, int, literal_int);
    });
}

#[test]
fn pp_lit5_in_unspec_lit() {
    fixture(|f| {
        let five = f.t_lit_int(5);
        let literal_int = f.t_int_unspec_lit();
        assert_atomic_subtype(f, five, literal_int);
    });
}

#[test]
fn pp_lit_str_in_str() {
    fixture(|f| {
        let literal = f.t_lit_string("hi");
        let string = f.t_string();
        assert_atomic_subtype(f, literal, string);
    });
}

#[test]
fn pp_str_not_in_lit() {
    fixture(|f| {
        let string = f.t_string();
        let literal = f.t_lit_string("hi");
        assert_atomic_not_subtype(f, string, literal);
    });
}

#[test]
fn pp_lit_str_disjoint() {
    fixture(|f| {
        let left = f.t_lit_string("a");
        let right = f.t_lit_string("b");
        assert_atomic_not_subtype(f, left, right);
    });
}

#[test]
fn pp_lit_str_eq() {
    fixture(|f| {
        let literal = f.t_lit_string("hi");
        assert_atomic_subtype(f, literal, literal);
    });
}

#[test]
fn pp_non_empty_in_str() {
    fixture(|f| {
        let non_empty = f.t_non_empty_string();
        let string = f.t_string();
        assert_atomic_subtype(f, non_empty, string);
    });
}

#[test]
fn pp_str_not_in_non_empty() {
    fixture(|f| {
        let string = f.t_string();
        let non_empty = f.t_non_empty_string();
        assert_atomic_not_subtype(f, string, non_empty);
    });
}

#[test]
fn pp_empty_lit_not_in_non_empty() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        let non_empty = f.t_non_empty_string();
        assert_atomic_not_subtype(f, empty, non_empty);
    });
}

#[test]
fn pp_a_lit_in_non_empty() {
    fixture(|f| {
        let literal = f.t_lit_string("a");
        let non_empty = f.t_non_empty_string();
        assert_atomic_subtype(f, literal, non_empty);
    });
}

#[test]
fn pp_truthy_in_str() {
    fixture(|f| {
        let truthy = f.t_truthy_string();
        let string = f.t_string();
        assert_atomic_subtype(f, truthy, string);
    });
}

#[test]
fn pp_truthy_in_non_empty() {
    fixture(|f| {
        let truthy = f.t_truthy_string();
        let non_empty = f.t_non_empty_string();
        assert_atomic_subtype(f, truthy, non_empty);
    });
}

#[test]
fn pp_lit_hi_in_truthy() {
    fixture(|f| {
        let literal = f.t_lit_string("hi");
        let truthy = f.t_truthy_string();
        assert_atomic_subtype(f, literal, truthy);
    });
}

#[test]
fn pp_lit_0_not_in_truthy() {
    fixture(|f| {
        let zero = f.t_lit_string("0");
        let truthy = f.t_truthy_string();
        assert_atomic_not_subtype(f, zero, truthy);
    });
}

#[test]
fn pp_lit_empty_not_in_truthy() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        let truthy = f.t_truthy_string();
        assert_atomic_not_subtype(f, empty, truthy);
    });
}

#[test]
fn pp_lower_in_str() {
    fixture(|f| {
        let lower = f.t_lower_string();
        let string = f.t_string();
        assert_atomic_subtype(f, lower, string);
    });
}

#[test]
fn pp_upper_in_str() {
    fixture(|f| {
        let upper = f.t_upper_string();
        let string = f.t_string();
        assert_atomic_subtype(f, upper, string);
    });
}

#[test]
fn pp_lower_not_in_upper() {
    fixture(|f| {
        let lower = f.t_lower_string();
        let upper = f.t_upper_string();
        assert_atomic_not_subtype(f, lower, upper);
    });
}

#[test]
fn pp_lit_hi_in_lower() {
    fixture(|f| {
        let literal = f.t_lit_string("hi");
        let lower = f.t_lower_string();
        assert_atomic_subtype(f, literal, lower);
    });
}

#[test]
fn pp_lit_upper_hi_not_in_lower() {
    fixture(|f| {
        let literal = f.t_lit_string("HI");
        let lower = f.t_lower_string();
        assert_atomic_not_subtype(f, literal, lower);
    });
}

#[test]
fn pp_lit_upper_hi_in_upper() {
    fixture(|f| {
        let literal = f.t_lit_string("HI");
        let upper = f.t_upper_string();
        assert_atomic_subtype(f, literal, upper);
    });
}

#[test]
fn pp_numeric_in_str() {
    fixture(|f| {
        let numeric_string = f.t_numeric_string();
        let string = f.t_string();
        assert_atomic_subtype(f, numeric_string, string);
    });
}

#[test]
fn pp_str_not_in_numeric() {
    fixture(|f| {
        let string = f.t_string();
        let numeric_string = f.t_numeric_string();
        assert_atomic_not_subtype(f, string, numeric_string);
    });
}

#[test]
fn pp_lit_123_in_numeric() {
    fixture(|f| {
        let literal = f.t_lit_string("123");
        let numeric_string = f.t_numeric_string();
        assert_atomic_subtype(f, literal, numeric_string);
    });
}

#[test]
fn pp_lit_abc_not_in_numeric() {
    fixture(|f| {
        let literal = f.t_lit_string("abc");
        let numeric_string = f.t_numeric_string();
        assert_atomic_not_subtype(f, literal, numeric_string);
    });
}

#[test]
fn pp_int_in_numeric() {
    fixture(|f| {
        let int = f.t_int();
        let numeric = f.t_numeric();
        assert_atomic_subtype(f, int, numeric);
    });
}

#[test]
fn pp_float_in_numeric() {
    fixture(|f| {
        let float = f.t_float();
        let numeric = f.t_numeric();
        assert_atomic_subtype(f, float, numeric);
    });
}

#[test]
fn pp_numstr_in_numeric() {
    fixture(|f| {
        let numeric_string = f.t_numeric_string();
        let numeric = f.t_numeric();
        assert_atomic_subtype(f, numeric_string, numeric);
    });
}

#[test]
fn pp_str_not_in_numeric_atom() {
    fixture(|f| {
        let string = f.t_string();
        let numeric = f.t_numeric();
        assert_atomic_not_subtype(f, string, numeric);
    });
}

#[test]
fn pp_bool_not_in_numeric() {
    fixture(|f| {
        let bool_atom = f.t_bool();
        let numeric = f.t_numeric();
        assert_atomic_not_subtype(f, bool_atom, numeric);
    });
}

#[test]
fn pp_int_in_array_key() {
    fixture(|f| {
        let int = f.t_int();
        let array_key = f.t_array_key();
        assert_atomic_subtype(f, int, array_key);
    });
}

#[test]
fn pp_str_in_array_key() {
    fixture(|f| {
        let string = f.t_string();
        let array_key = f.t_array_key();
        assert_atomic_subtype(f, string, array_key);
    });
}

#[test]
fn pp_float_not_in_array_key() {
    fixture(|f| {
        let float = f.t_float();
        let array_key = f.t_array_key();
        assert_atomic_not_subtype(f, float, array_key);
    });
}

#[test]
fn pp_array_key_not_in_int() {
    fixture(|f| {
        let array_key = f.t_array_key();
        let int = f.t_int();
        assert_atomic_not_subtype(f, array_key, int);
    });
}

#[test]
fn pp_int_in_scalar() {
    fixture(|f| {
        let int = f.t_int();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, int, scalar);
    });
}

#[test]
fn pp_str_in_scalar() {
    fixture(|f| {
        let string = f.t_string();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, string, scalar);
    });
}

#[test]
fn pp_float_in_scalar() {
    fixture(|f| {
        let float = f.t_float();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, float, scalar);
    });
}

#[test]
fn pp_bool_in_scalar() {
    fixture(|f| {
        let bool_atom = f.t_bool();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, bool_atom, scalar);
    });
}

#[test]
fn pp_numeric_in_scalar() {
    fixture(|f| {
        let numeric = f.t_numeric();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, numeric, scalar);
    });
}

#[test]
fn pp_array_key_in_scalar() {
    fixture(|f| {
        let array_key = f.t_array_key();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, array_key, scalar);
    });
}

#[test]
fn pp_class_in_scalar() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, class_string, scalar);
    });
}

#[test]
fn pp_null_not_in_scalar() {
    fixture(|f| {
        let null = f.null();
        let scalar = f.t_scalar();
        assert_atomic_not_subtype(f, null, scalar);
    });
}

#[test]
fn pp_object_not_in_scalar() {
    fixture(|f| {
        let object = f.t_object_any();
        let scalar = f.t_scalar();
        assert_atomic_not_subtype(f, object, scalar);
    });
}

#[test]
fn pp_class_in_str() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let string = f.t_string();
        assert_atomic_subtype(f, class_string, string);
    });
}

#[test]
fn pp_class_in_array_key() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let array_key = f.t_array_key();
        assert_atomic_subtype(f, class_string, array_key);
    });
}

#[test]
fn pp_class_not_in_int() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let int = f.t_int();
        assert_atomic_not_subtype(f, class_string, int);
    });
}

#[test]
fn pp_lit_class_in_class() {
    fixture(|f| {
        let literal = f.t_lit_class_string("Foo");
        let class_string = f.t_class_string();
        assert_atomic_subtype(f, literal, class_string);
    });
}

#[test]
fn pp_class_not_in_lit_class() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let literal = f.t_lit_class_string("Foo");
        assert_atomic_not_subtype(f, class_string, literal);
    });
}

#[test]
fn pp_lit_float_in_float() {
    fixture(|f| {
        let literal = f.t_lit_float(1.5);
        let float = f.t_float();
        assert_atomic_subtype(f, literal, float);
    });
}

#[test]
fn pp_float_not_in_lit_float() {
    fixture(|f| {
        let float = f.t_float();
        let literal = f.t_lit_float(1.5);
        assert_atomic_not_subtype(f, float, literal);
    });
}

#[test]
fn pp_lit_floats_disjoint() {
    fixture(|f| {
        let one = f.t_lit_float(1.0);
        let two = f.t_lit_float(2.0);
        assert_atomic_not_subtype(f, one, two);
    });
}

#[test]
fn pp_lit_float_in_numeric() {
    fixture(|f| {
        let literal = f.t_lit_float(1.5);
        let numeric = f.t_numeric();
        assert_atomic_subtype(f, literal, numeric);
    });
}

#[test]
fn pp_float_not_in_int() {
    fixture(|f| {
        let float = f.t_float();
        let int = f.t_int();
        assert_atomic_not_subtype(f, float, int);
    });
}

#[test]
fn pp_int_not_in_float() {
    fixture(|f| {
        let int = f.t_int();
        let float = f.t_float();
        assert_atomic_not_subtype(f, int, float);
    });
}

#[test]
fn pp_lit_int_not_in_float() {
    fixture(|f| {
        let literal = f.t_lit_int(5);
        let float = f.t_float();
        assert_atomic_not_subtype(f, literal, float);
    });
}

#[test]
fn pp_open_in_resource() {
    fixture(|f| {
        let open = f.t_open_resource();
        let resource = f.t_resource();
        assert_atomic_subtype(f, open, resource);
    });
}

#[test]
fn pp_closed_in_resource() {
    fixture(|f| {
        let closed = f.t_closed_resource();
        let resource = f.t_resource();
        assert_atomic_subtype(f, closed, resource);
    });
}

#[test]
fn pp_resource_not_in_open() {
    fixture(|f| {
        let resource = f.t_resource();
        let open = f.t_open_resource();
        assert_atomic_not_subtype(f, resource, open);
    });
}

#[test]
fn pp_open_not_in_closed() {
    fixture(|f| {
        let open = f.t_open_resource();
        let closed = f.t_closed_resource();
        assert_atomic_not_subtype(f, open, closed);
    });
}

#[test]
fn pp_int_in_mixed() {
    fixture(|f| {
        let int = f.t_int();
        let mixed = f.mixed();
        assert_atomic_subtype(f, int, mixed);
    });
}

#[test]
fn pp_str_in_mixed() {
    fixture(|f| {
        let string = f.t_string();
        let mixed = f.mixed();
        assert_atomic_subtype(f, string, mixed);
    });
}

#[test]
fn pp_float_in_mixed() {
    fixture(|f| {
        let float = f.t_float();
        let mixed = f.mixed();
        assert_atomic_subtype(f, float, mixed);
    });
}

#[test]
fn pp_bool_in_mixed() {
    fixture(|f| {
        let bool_atom = f.t_bool();
        let mixed = f.mixed();
        assert_atomic_subtype(f, bool_atom, mixed);
    });
}

#[test]
fn pp_null_in_mixed() {
    fixture(|f| {
        let null = f.null();
        let mixed = f.mixed();
        assert_atomic_subtype(f, null, mixed);
    });
}

#[test]
fn pp_void_in_mixed() {
    fixture(|f| {
        let void = f.void();
        let mixed = f.mixed();
        assert_atomic_subtype(f, void, mixed);
    });
}

#[test]
fn pp_object_in_mixed() {
    fixture(|f| {
        let object = f.t_object_any();
        let mixed = f.mixed();
        assert_atomic_subtype(f, object, mixed);
    });
}

#[test]
fn pp_resource_in_mixed() {
    fixture(|f| {
        let resource = f.t_resource();
        let mixed = f.mixed();
        assert_atomic_subtype(f, resource, mixed);
    });
}

#[test]
fn pp_array_in_mixed() {
    fixture(|f| {
        let array = f.t_empty_array();
        let mixed = f.mixed();
        assert_atomic_subtype(f, array, mixed);
    });
}

#[test]
fn pp_mixed_not_in_int() {
    fixture(|f| {
        let mixed = f.mixed();
        let int = f.t_int();
        assert_atomic_not_subtype(f, mixed, int);
    });
}

#[test]
fn pp_mixed_not_in_str() {
    fixture(|f| {
        let mixed = f.mixed();
        let string = f.t_string();
        assert_atomic_not_subtype(f, mixed, string);
    });
}

#[test]
fn pp_mixed_not_in_float() {
    fixture(|f| {
        let mixed = f.mixed();
        let float = f.t_float();
        assert_atomic_not_subtype(f, mixed, float);
    });
}

#[test]
fn pp_mixed_not_in_bool() {
    fixture(|f| {
        let mixed = f.mixed();
        let bool_atom = f.t_bool();
        assert_atomic_not_subtype(f, mixed, bool_atom);
    });
}

#[test]
fn pp_mixed_not_in_null() {
    fixture(|f| {
        let mixed = f.mixed();
        let null = f.null();
        assert_atomic_not_subtype(f, mixed, null);
    });
}

#[test]
fn pp_mixed_not_in_object() {
    fixture(|f| {
        let mixed = f.mixed();
        let object = f.t_object_any();
        assert_atomic_not_subtype(f, mixed, object);
    });
}

#[test]
fn pp_never_in_int() {
    fixture(|f| {
        let never = f.never();
        let int = f.t_int();
        assert_atomic_subtype(f, never, int);
    });
}

#[test]
fn pp_never_in_str() {
    fixture(|f| {
        let never = f.never();
        let string = f.t_string();
        assert_atomic_subtype(f, never, string);
    });
}

#[test]
fn pp_never_in_float() {
    fixture(|f| {
        let never = f.never();
        let float = f.t_float();
        assert_atomic_subtype(f, never, float);
    });
}

#[test]
fn pp_never_in_bool() {
    fixture(|f| {
        let never = f.never();
        let bool_atom = f.t_bool();
        assert_atomic_subtype(f, never, bool_atom);
    });
}

#[test]
fn pp_never_in_null() {
    fixture(|f| {
        let never = f.never();
        let null = f.null();
        assert_atomic_subtype(f, never, null);
    });
}

#[test]
fn pp_never_in_void() {
    fixture(|f| {
        let never = f.never();
        let void = f.void();
        assert_atomic_subtype(f, never, void);
    });
}

#[test]
fn pp_never_in_object() {
    fixture(|f| {
        let never = f.never();
        let object = f.t_object_any();
        assert_atomic_subtype(f, never, object);
    });
}

#[test]
fn pp_never_in_resource() {
    fixture(|f| {
        let never = f.never();
        let resource = f.t_resource();
        assert_atomic_subtype(f, never, resource);
    });
}

#[test]
fn pp_never_in_array() {
    fixture(|f| {
        let never = f.never();
        let array = f.t_empty_array();
        assert_atomic_subtype(f, never, array);
    });
}

#[test]
fn pp_never_in_mixed() {
    fixture(|f| {
        let never = f.never();
        let mixed = f.mixed();
        assert_atomic_subtype(f, never, mixed);
    });
}

#[test]
fn pp_int_not_in_never() {
    fixture(|f| {
        let int = f.t_int();
        let never = f.never();
        assert_atomic_not_subtype(f, int, never);
    });
}

#[test]
fn pp_str_not_in_never() {
    fixture(|f| {
        let string = f.t_string();
        let never = f.never();
        assert_atomic_not_subtype(f, string, never);
    });
}

#[test]
fn pp_null_not_in_never() {
    fixture(|f| {
        let null = f.null();
        let never = f.never();
        assert_atomic_not_subtype(f, null, never);
    });
}

#[test]
fn pp_object_not_in_never() {
    fixture(|f| {
        let object = f.t_object_any();
        let never = f.never();
        assert_atomic_not_subtype(f, object, never);
    });
}

#[test]
fn pp_int_str_disjoint() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        assert_atomic_not_subtype(f, int, string);
        assert_atomic_not_subtype(f, string, int);
    });
}

#[test]
fn pp_int_bool_disjoint() {
    fixture(|f| {
        let int = f.t_int();
        let bool_atom = f.t_bool();
        assert_atomic_not_subtype(f, int, bool_atom);
        assert_atomic_not_subtype(f, bool_atom, int);
    });
}

#[test]
fn pp_str_bool_disjoint() {
    fixture(|f| {
        let string = f.t_string();
        let bool_atom = f.t_bool();
        assert_atomic_not_subtype(f, string, bool_atom);
        assert_atomic_not_subtype(f, bool_atom, string);
    });
}

#[test]
fn pp_int_null_disjoint() {
    fixture(|f| {
        let int = f.t_int();
        let null = f.null();
        assert_atomic_not_subtype(f, int, null);
        assert_atomic_not_subtype(f, null, int);
    });
}

#[test]
fn pp_str_null_disjoint() {
    fixture(|f| {
        let string = f.t_string();
        let null = f.null();
        assert_atomic_not_subtype(f, string, null);
        assert_atomic_not_subtype(f, null, string);
    });
}

#[test]
fn pp_int_object_disjoint() {
    fixture(|f| {
        let int = f.t_int();
        let object = f.t_object_any();
        assert_atomic_not_subtype(f, int, object);
        assert_atomic_not_subtype(f, object, int);
    });
}

#[test]
fn pp_int_array_disjoint() {
    fixture(|f| {
        let int = f.t_int();
        let array = f.t_empty_array();
        assert_atomic_not_subtype(f, int, array);
        assert_atomic_not_subtype(f, array, int);
    });
}

#[test]
fn pp_int_resource_disjoint() {
    fixture(|f| {
        let int = f.t_int();
        let resource = f.t_resource();
        assert_atomic_not_subtype(f, int, resource);
        assert_atomic_not_subtype(f, resource, int);
    });
}

#[test]
fn pp_object_array_disjoint() {
    fixture(|f| {
        let object = f.t_object_any();
        let array = f.t_empty_array();
        assert_atomic_not_subtype(f, object, array);
        assert_atomic_not_subtype(f, array, object);
    });
}

#[test]
fn pp_object_resource_disjoint() {
    fixture(|f| {
        let object = f.t_object_any();
        let resource = f.t_resource();
        assert_atomic_not_subtype(f, object, resource);
        assert_atomic_not_subtype(f, resource, object);
    });
}

#[test]
fn pp_array_resource_disjoint() {
    fixture(|f| {
        let array = f.t_empty_array();
        let resource = f.t_resource();
        assert_atomic_not_subtype(f, array, resource);
        assert_atomic_not_subtype(f, resource, array);
    });
}
