mod comparator_common;

use comparator_common::*;

#[test]
fn pp_int_int() {
    assert_atomic_subtype(&t_int(), &t_int());
}

#[test]
fn pp_string_string() {
    assert_atomic_subtype(&t_string(), &t_string());
}

#[test]
fn pp_float_float() {
    assert_atomic_subtype(&t_float(), &t_float());
}

#[test]
fn pp_bool_bool() {
    assert_atomic_subtype(&t_bool(), &t_bool());
}

#[test]
fn pp_true_true() {
    assert_atomic_subtype(&t_true(), &t_true());
}

#[test]
fn pp_false_false() {
    assert_atomic_subtype(&t_false(), &t_false());
}

#[test]
fn pp_null_null() {
    assert_atomic_subtype(&null(), &null());
}

#[test]
fn pp_void_void() {
    assert_atomic_subtype(&void(), &void());
}

#[test]
fn pp_never_never() {
    assert_atomic_subtype(&never(), &never());
}

#[test]
fn pp_mixed_mixed() {
    assert_atomic_subtype(&mixed(), &mixed());
}

#[test]
fn pp_object_object() {
    assert_atomic_subtype(&t_object_any(), &t_object_any());
}

#[test]
fn pp_resource_resource() {
    assert_atomic_subtype(&t_resource(), &t_resource());
}

#[test]
fn pp_array_key_array_key() {
    assert_atomic_subtype(&t_array_key(), &t_array_key());
}

#[test]
fn pp_numeric_numeric() {
    assert_atomic_subtype(&t_numeric(), &t_numeric());
}

#[test]
fn pp_scalar_scalar() {
    assert_atomic_subtype(&t_scalar(), &t_scalar());
}

#[test]
fn pp_true_in_bool() {
    assert_atomic_subtype(&t_true(), &t_bool());
}

#[test]
fn pp_false_in_bool() {
    assert_atomic_subtype(&t_false(), &t_bool());
}

#[test]
fn pp_bool_not_in_true() {
    assert_atomic_not_subtype(&t_bool(), &t_true());
}

#[test]
fn pp_bool_not_in_false() {
    assert_atomic_not_subtype(&t_bool(), &t_false());
}

#[test]
fn pp_true_not_in_false() {
    assert_atomic_not_subtype(&t_true(), &t_false());
}

#[test]
fn pp_false_not_in_true() {
    assert_atomic_not_subtype(&t_false(), &t_true());
}

#[test]
fn pp_lit_in_int_0() {
    assert_atomic_subtype(&t_lit_int(0), &t_int());
}

#[test]
fn pp_lit_in_int_1() {
    assert_atomic_subtype(&t_lit_int(1), &t_int());
}

#[test]
fn pp_lit_in_int_neg() {
    assert_atomic_subtype(&t_lit_int(-1), &t_int());
}

#[test]
fn pp_int_not_in_lit() {
    assert_atomic_not_subtype(&t_int(), &t_lit_int(5));
}

#[test]
fn pp_lit_disjoint() {
    assert_atomic_not_subtype(&t_lit_int(1), &t_lit_int(2));
}

#[test]
fn pp_lit_eq() {
    assert_atomic_subtype(&t_lit_int(5), &t_lit_int(5));
}

#[test]
fn pp_pos_in_int() {
    assert_atomic_subtype(&t_positive_int(), &t_int());
}

#[test]
fn pp_neg_in_int() {
    assert_atomic_subtype(&t_negative_int(), &t_int());
}

#[test]
fn pp_nn_in_int() {
    assert_atomic_subtype(&t_non_negative_int(), &t_int());
}

#[test]
fn pp_np_in_int() {
    assert_atomic_subtype(&t_non_positive_int(), &t_int());
}

#[test]
fn pp_int_not_pos() {
    assert_atomic_not_subtype(&t_int(), &t_positive_int());
}

#[test]
fn pp_int_not_neg() {
    assert_atomic_not_subtype(&t_int(), &t_negative_int());
}

#[test]
fn pp_pos_in_nn() {
    assert_atomic_subtype(&t_positive_int(), &t_non_negative_int());
}

#[test]
fn pp_neg_in_np() {
    assert_atomic_subtype(&t_negative_int(), &t_non_positive_int());
}

#[test]
fn pp_pos_neg_disjoint() {
    assert_atomic_not_subtype(&t_positive_int(), &t_negative_int());
}

#[test]
fn pp_range_05_in_range_010() {
    assert_atomic_subtype(&t_int_range(0, 5), &t_int_range(0, 10));
}

#[test]
fn pp_range_010_not_in_range_05() {
    assert_atomic_not_subtype(&t_int_range(0, 10), &t_int_range(0, 5));
}

#[test]
fn pp_range_05_in_pos() {
    assert_atomic_not_subtype(&t_int_range(0, 5), &t_positive_int());
}

#[test]
fn pp_range_15_in_pos() {
    assert_atomic_subtype(&t_int_range(1, 5), &t_positive_int());
}

#[test]
fn pp_lit5_in_range010() {
    assert_atomic_subtype(&t_lit_int(5), &t_int_range(0, 10));
}

#[test]
fn pp_lit15_not_in_range010() {
    assert_atomic_not_subtype(&t_lit_int(15), &t_int_range(0, 10));
}

#[test]
fn pp_from5_in_pos() {
    assert_atomic_subtype(&t_int_from(5), &t_positive_int());
}

#[test]
fn pp_from0_not_in_pos() {
    assert_atomic_not_subtype(&t_int_from(0), &t_positive_int());
}

#[test]
fn pp_from0_in_nn() {
    assert_atomic_subtype(&t_int_from(0), &t_non_negative_int());
}

#[test]
fn pp_to_neg1_in_neg() {
    assert_atomic_subtype(&t_int_to(-1), &t_negative_int());
}

#[test]
fn pp_to0_in_np() {
    assert_atomic_subtype(&t_int_to(0), &t_non_positive_int());
}

#[test]
fn pp_unspec_lit_in_int() {
    assert_atomic_subtype(&t_int_unspec_lit(), &t_int());
}

#[test]
fn pp_int_not_in_unspec_lit() {
    assert_atomic_not_subtype(&t_int(), &t_int_unspec_lit());
}

#[test]
fn pp_lit5_in_unspec_lit() {
    assert_atomic_subtype(&t_lit_int(5), &t_int_unspec_lit());
}

#[test]
fn pp_lit_str_in_str() {
    assert_atomic_subtype(&t_lit_string("hi"), &t_string());
}

#[test]
fn pp_str_not_in_lit() {
    assert_atomic_not_subtype(&t_string(), &t_lit_string("hi"));
}

#[test]
fn pp_lit_str_disjoint() {
    assert_atomic_not_subtype(&t_lit_string("a"), &t_lit_string("b"));
}

#[test]
fn pp_lit_str_eq() {
    assert_atomic_subtype(&t_lit_string("hi"), &t_lit_string("hi"));
}

#[test]
fn pp_non_empty_in_str() {
    assert_atomic_subtype(&t_non_empty_string(), &t_string());
}

#[test]
fn pp_str_not_in_non_empty() {
    assert_atomic_not_subtype(&t_string(), &t_non_empty_string());
}

#[test]
fn pp_empty_lit_not_in_non_empty() {
    assert_atomic_not_subtype(&t_lit_string(""), &t_non_empty_string());
}

#[test]
fn pp_a_lit_in_non_empty() {
    assert_atomic_subtype(&t_lit_string("a"), &t_non_empty_string());
}

#[test]
fn pp_truthy_in_str() {
    assert_atomic_subtype(&t_truthy_string(), &t_string());
}

#[test]
fn pp_truthy_in_non_empty() {
    assert_atomic_subtype(&t_truthy_string(), &t_non_empty_string());
}

#[test]
fn pp_lit_hi_in_truthy() {
    assert_atomic_subtype(&t_lit_string("hi"), &t_truthy_string());
}

#[test]
fn pp_lit_0_not_in_truthy() {
    assert_atomic_not_subtype(&t_lit_string("0"), &t_truthy_string());
}

#[test]
fn pp_lit_empty_not_in_truthy() {
    assert_atomic_not_subtype(&t_lit_string(""), &t_truthy_string());
}

#[test]
fn pp_lower_in_str() {
    assert_atomic_subtype(&t_lower_string(), &t_string());
}

#[test]
fn pp_upper_in_str() {
    assert_atomic_subtype(&t_upper_string(), &t_string());
}

#[test]
fn pp_lower_not_in_upper() {
    assert_atomic_not_subtype(&t_lower_string(), &t_upper_string());
}

#[test]
fn pp_lit_hi_in_lower() {
    assert_atomic_subtype(&t_lit_string("hi"), &t_lower_string());
}

#[test]
fn pp_lit_upper_hi_not_in_lower() {
    assert_atomic_not_subtype(&t_lit_string("HI"), &t_lower_string());
}

#[test]
fn pp_lit_upper_hi_in_upper() {
    assert_atomic_subtype(&t_lit_string("HI"), &t_upper_string());
}

#[test]
fn pp_numeric_in_str() {
    assert_atomic_subtype(&t_numeric_string(), &t_string());
}

#[test]
fn pp_str_not_in_numeric() {
    assert_atomic_not_subtype(&t_string(), &t_numeric_string());
}

#[test]
fn pp_lit_123_in_numeric() {
    assert_atomic_subtype(&t_lit_string("123"), &t_numeric_string());
}

#[test]
fn pp_lit_abc_not_in_numeric() {
    assert_atomic_not_subtype(&t_lit_string("abc"), &t_numeric_string());
}

#[test]
fn pp_int_in_numeric() {
    assert_atomic_subtype(&t_int(), &t_numeric());
}

#[test]
fn pp_float_in_numeric() {
    assert_atomic_subtype(&t_float(), &t_numeric());
}

#[test]
fn pp_numstr_in_numeric() {
    assert_atomic_subtype(&t_numeric_string(), &t_numeric());
}

#[test]
fn pp_str_not_in_numeric_atom() {
    assert_atomic_not_subtype(&t_string(), &t_numeric());
}

#[test]
fn pp_bool_not_in_numeric() {
    assert_atomic_not_subtype(&t_bool(), &t_numeric());
}

#[test]
fn pp_int_in_array_key() {
    assert_atomic_subtype(&t_int(), &t_array_key());
}

#[test]
fn pp_str_in_array_key() {
    assert_atomic_subtype(&t_string(), &t_array_key());
}

#[test]
fn pp_float_not_in_array_key() {
    assert_atomic_not_subtype(&t_float(), &t_array_key());
}

#[test]
fn pp_array_key_not_in_int() {
    assert_atomic_not_subtype(&t_array_key(), &t_int());
}

#[test]
fn pp_int_in_scalar() {
    assert_atomic_subtype(&t_int(), &t_scalar());
}

#[test]
fn pp_str_in_scalar() {
    assert_atomic_subtype(&t_string(), &t_scalar());
}

#[test]
fn pp_float_in_scalar() {
    assert_atomic_subtype(&t_float(), &t_scalar());
}

#[test]
fn pp_bool_in_scalar() {
    assert_atomic_subtype(&t_bool(), &t_scalar());
}

#[test]
fn pp_numeric_in_scalar() {
    assert_atomic_subtype(&t_numeric(), &t_scalar());
}

#[test]
fn pp_array_key_in_scalar() {
    assert_atomic_subtype(&t_array_key(), &t_scalar());
}

#[test]
fn pp_class_in_scalar() {
    assert_atomic_subtype(&t_class_string(), &t_scalar());
}

#[test]
fn pp_null_not_in_scalar() {
    assert_atomic_not_subtype(&null(), &t_scalar());
}

#[test]
fn pp_object_not_in_scalar() {
    assert_atomic_not_subtype(&t_object_any(), &t_scalar());
}

#[test]
fn pp_class_in_str() {
    assert_atomic_subtype(&t_class_string(), &t_string());
}

#[test]
fn pp_class_in_array_key() {
    assert_atomic_subtype(&t_class_string(), &t_array_key());
}

#[test]
fn pp_class_not_in_int() {
    assert_atomic_not_subtype(&t_class_string(), &t_int());
}

#[test]
fn pp_lit_class_in_class() {
    assert_atomic_subtype(&t_lit_class_string("Foo"), &t_class_string());
}

#[test]
fn pp_class_not_in_lit_class() {
    assert_atomic_not_subtype(&t_class_string(), &t_lit_class_string("Foo"));
}

#[test]
fn pp_lit_float_in_float() {
    assert_atomic_subtype(&t_lit_float(1.5), &t_float());
}

#[test]
fn pp_float_not_in_lit_float() {
    assert_atomic_not_subtype(&t_float(), &t_lit_float(1.5));
}

#[test]
fn pp_lit_floats_disjoint() {
    assert_atomic_not_subtype(&t_lit_float(1.0), &t_lit_float(2.0));
}

#[test]
fn pp_lit_float_in_numeric() {
    assert_atomic_subtype(&t_lit_float(1.5), &t_numeric());
}

#[test]
fn pp_float_not_in_int() {
    assert_atomic_not_subtype(&t_float(), &t_int());
}

#[test]
fn pp_int_in_float() {
    assert_atomic_subtype(&t_int(), &t_float());
}

#[test]
fn pp_lit_int_in_float() {
    assert_atomic_subtype(&t_lit_int(5), &t_float());
}

#[test]
fn pp_open_in_resource() {
    assert_atomic_subtype(&t_open_resource(), &t_resource());
}

#[test]
fn pp_closed_in_resource() {
    assert_atomic_subtype(&t_closed_resource(), &t_resource());
}

#[test]
fn pp_resource_not_in_open() {
    assert_atomic_not_subtype(&t_resource(), &t_open_resource());
}

#[test]
fn pp_open_not_in_closed() {
    assert_atomic_not_subtype(&t_open_resource(), &t_closed_resource());
}

#[test]
fn pp_int_in_mixed() {
    assert_atomic_subtype(&t_int(), &mixed());
}

#[test]
fn pp_str_in_mixed() {
    assert_atomic_subtype(&t_string(), &mixed());
}

#[test]
fn pp_float_in_mixed() {
    assert_atomic_subtype(&t_float(), &mixed());
}

#[test]
fn pp_bool_in_mixed() {
    assert_atomic_subtype(&t_bool(), &mixed());
}

#[test]
fn pp_null_in_mixed() {
    assert_atomic_subtype(&null(), &mixed());
}

#[test]
fn pp_void_in_mixed() {
    assert_atomic_subtype(&void(), &mixed());
}

#[test]
fn pp_object_in_mixed() {
    assert_atomic_subtype(&t_object_any(), &mixed());
}

#[test]
fn pp_resource_in_mixed() {
    assert_atomic_subtype(&t_resource(), &mixed());
}

#[test]
fn pp_array_in_mixed() {
    assert_atomic_subtype(&t_empty_array(), &mixed());
}

#[test]
fn pp_mixed_not_in_int() {
    assert_atomic_not_subtype(&mixed(), &t_int());
}

#[test]
fn pp_mixed_not_in_str() {
    assert_atomic_not_subtype(&mixed(), &t_string());
}

#[test]
fn pp_mixed_not_in_float() {
    assert_atomic_not_subtype(&mixed(), &t_float());
}

#[test]
fn pp_mixed_not_in_bool() {
    assert_atomic_not_subtype(&mixed(), &t_bool());
}

#[test]
fn pp_mixed_not_in_null() {
    assert_atomic_not_subtype(&mixed(), &null());
}

#[test]
fn pp_mixed_not_in_object() {
    assert_atomic_not_subtype(&mixed(), &t_object_any());
}

#[test]
fn pp_never_in_int() {
    assert_atomic_subtype(&never(), &t_int());
}

#[test]
fn pp_never_in_str() {
    assert_atomic_subtype(&never(), &t_string());
}

#[test]
fn pp_never_in_float() {
    assert_atomic_subtype(&never(), &t_float());
}

#[test]
fn pp_never_in_bool() {
    assert_atomic_subtype(&never(), &t_bool());
}

#[test]
fn pp_never_in_null() {
    assert_atomic_subtype(&never(), &null());
}

#[test]
fn pp_never_in_void() {
    assert_atomic_subtype(&never(), &void());
}

#[test]
fn pp_never_in_object() {
    assert_atomic_subtype(&never(), &t_object_any());
}

#[test]
fn pp_never_in_resource() {
    assert_atomic_subtype(&never(), &t_resource());
}

#[test]
fn pp_never_in_array() {
    assert_atomic_subtype(&never(), &t_empty_array());
}

#[test]
fn pp_never_in_mixed() {
    assert_atomic_subtype(&never(), &mixed());
}

#[test]
fn pp_int_not_in_never() {
    assert_atomic_not_subtype(&t_int(), &never());
}

#[test]
fn pp_str_not_in_never() {
    assert_atomic_not_subtype(&t_string(), &never());
}

#[test]
fn pp_null_not_in_never() {
    assert_atomic_not_subtype(&null(), &never());
}

#[test]
fn pp_object_not_in_never() {
    assert_atomic_not_subtype(&t_object_any(), &never());
}

#[test]
fn pp_int_str_disjoint() {
    assert_atomic_not_subtype(&t_int(), &t_string());
    assert_atomic_not_subtype(&t_string(), &t_int());
}

#[test]
fn pp_int_bool_disjoint() {
    assert_atomic_not_subtype(&t_int(), &t_bool());
    assert_atomic_not_subtype(&t_bool(), &t_int());
}

#[test]
fn pp_str_bool_disjoint() {
    assert_atomic_not_subtype(&t_string(), &t_bool());
    assert_atomic_not_subtype(&t_bool(), &t_string());
}

#[test]
fn pp_int_null_disjoint() {
    assert_atomic_not_subtype(&t_int(), &null());
    assert_atomic_not_subtype(&null(), &t_int());
}

#[test]
fn pp_str_null_disjoint() {
    assert_atomic_not_subtype(&t_string(), &null());
    assert_atomic_not_subtype(&null(), &t_string());
}

#[test]
fn pp_int_object_disjoint() {
    assert_atomic_not_subtype(&t_int(), &t_object_any());
    assert_atomic_not_subtype(&t_object_any(), &t_int());
}

#[test]
fn pp_int_array_disjoint() {
    assert_atomic_not_subtype(&t_int(), &t_empty_array());
    assert_atomic_not_subtype(&t_empty_array(), &t_int());
}

#[test]
fn pp_int_resource_disjoint() {
    assert_atomic_not_subtype(&t_int(), &t_resource());
    assert_atomic_not_subtype(&t_resource(), &t_int());
}

#[test]
fn pp_object_array_disjoint() {
    assert_atomic_not_subtype(&t_object_any(), &t_empty_array());
    assert_atomic_not_subtype(&t_empty_array(), &t_object_any());
}

#[test]
fn pp_object_resource_disjoint() {
    assert_atomic_not_subtype(&t_object_any(), &t_resource());
    assert_atomic_not_subtype(&t_resource(), &t_object_any());
}

#[test]
fn pp_array_resource_disjoint() {
    assert_atomic_not_subtype(&t_empty_array(), &t_resource());
    assert_atomic_not_subtype(&t_resource(), &t_empty_array());
}

#[test]
fn pp_empty_in_list() {
    assert_atomic_subtype(&t_empty_array(), &t_list(u(t_int()), false));
}

#[test]
fn pp_empty_not_in_ne_list() {
    assert_atomic_not_subtype(&t_empty_array(), &t_list(u(t_int()), true));
}

#[test]
fn pp_list_int_int() {
    assert_atomic_subtype(&t_list(u(t_int()), false), &t_list(u(t_int()), false));
}

#[test]
fn pp_ne_list_in_list() {
    assert_atomic_subtype(&t_list(u(t_int()), true), &t_list(u(t_int()), false));
}

#[test]
fn pp_list_not_in_ne_list() {
    assert_atomic_not_subtype(&t_list(u(t_int()), false), &t_list(u(t_int()), true));
}

#[test]
fn pp_list_int_in_list_scalar() {
    assert_atomic_subtype(&t_list(u(t_int()), false), &t_list(u(t_scalar()), false));
}

#[test]
fn pp_list_int_in_list_mixed() {
    assert_atomic_subtype(&t_list(u(t_int()), false), &t_list(u(mixed()), false));
}

#[test]
fn pp_list_lit_in_list_int() {
    assert_atomic_subtype(&t_list(u(t_lit_int(5)), false), &t_list(u(t_int()), false));
}

#[test]
fn pp_list_int_str_disjoint() {
    assert_atomic_not_subtype(&t_list(u(t_int()), false), &t_list(u(t_string()), false));
}
