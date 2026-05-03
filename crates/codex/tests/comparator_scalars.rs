#![allow(clippy::approx_constant)]

mod comparator_common;

use comparator_common::*;

#[test]
fn reflexivity_all_scalars() {
    for atom in scalar_zoo() {
        assert_atomic_subtype(&atom, &atom);
    }
}

#[test]
fn int_in_int() {
    assert_atomic_subtype(&t_int(), &t_int());
}

#[test]
fn lit_int_in_int() {
    for v in [-1000i64, -1, 0, 1, 100, 1_000_000] {
        assert_atomic_subtype(&t_lit_int(v), &t_int());
    }
}

#[test]
fn int_not_in_lit_int() {
    for v in [-1, 0, 1, 100] {
        assert_atomic_not_subtype(&t_int(), &t_lit_int(v));
    }
}

#[test]
fn distinct_lit_ints_disjoint() {
    for (a, b) in [(0i64, 1), (-1, 1), (10, 100), (1, 2)] {
        assert_atomic_not_subtype(&t_lit_int(a), &t_lit_int(b));
        assert_atomic_not_subtype(&t_lit_int(b), &t_lit_int(a));
    }
}

#[test]
fn equal_lit_ints_subtype() {
    for v in [-100i64, -1, 0, 1, 100] {
        assert_atomic_subtype(&t_lit_int(v), &t_lit_int(v));
    }
}

#[test]
fn float_in_float() {
    assert_atomic_subtype(&t_float(), &t_float());
}

#[test]
fn lit_float_in_float() {
    for v in [-3.14f64, 0.0, 1.5, 100.0] {
        assert_atomic_subtype(&t_lit_float(v), &t_float());
    }
}

#[test]
fn float_not_in_lit_float() {
    for v in [0.0f64, 1.5, -3.14] {
        assert_atomic_not_subtype(&t_float(), &t_lit_float(v));
    }
}

#[test]
fn distinct_lit_floats_disjoint() {
    for (a, b) in [(0.0f64, 1.0), (1.5, 2.5), (-1.0, 1.0)] {
        assert_atomic_not_subtype(&t_lit_float(a), &t_lit_float(b));
    }
}

#[test]
fn string_in_string() {
    assert_atomic_subtype(&t_string(), &t_string());
}

#[test]
fn lit_string_in_string() {
    for s in ["", "hi", "0", "123", "Hello World"] {
        assert_atomic_subtype(&t_lit_string(s), &t_string());
    }
}

#[test]
fn string_not_in_lit_string() {
    for s in ["", "hi", "abc"] {
        assert_atomic_not_subtype(&t_string(), &t_lit_string(s));
    }
}

#[test]
fn distinct_lit_strings_disjoint() {
    for (a, b) in [("a", "b"), ("hi", "hello"), ("", "x")] {
        assert_atomic_not_subtype(&t_lit_string(a), &t_lit_string(b));
    }
}

#[test]
fn true_in_bool() {
    assert_atomic_subtype(&t_true(), &t_bool());
}

#[test]
fn false_in_bool() {
    assert_atomic_subtype(&t_false(), &t_bool());
}

#[test]
fn bool_not_in_true() {
    assert_atomic_not_subtype(&t_bool(), &t_true());
}

#[test]
fn bool_not_in_false() {
    assert_atomic_not_subtype(&t_bool(), &t_false());
}

#[test]
fn true_not_in_false() {
    assert_atomic_not_subtype(&t_true(), &t_false());
}

#[test]
fn false_not_in_true() {
    assert_atomic_not_subtype(&t_false(), &t_true());
}

#[test]
fn int_in_numeric() {
    assert_atomic_subtype(&t_int(), &t_numeric());
}

#[test]
fn float_in_numeric() {
    assert_atomic_subtype(&t_float(), &t_numeric());
}

#[test]
fn numeric_string_in_numeric() {
    assert_atomic_subtype(&t_numeric_string(), &t_numeric());
}

#[test]
fn string_not_in_numeric() {
    assert_atomic_not_subtype(&t_string(), &t_numeric());
}

#[test]
fn numeric_lit_string_in_numeric() {
    for s in ["0", "1", "-1", "123", "1.5", "1e10"] {
        assert_atomic_subtype(&t_lit_string(s), &t_numeric());
    }
}

#[test]
fn non_numeric_lit_string_not_in_numeric() {
    for s in ["abc", "hi", "12abc", "abc123"] {
        assert_atomic_not_subtype(&t_lit_string(s), &t_numeric());
    }
}

#[test]
fn lit_int_in_numeric() {
    for v in [-1000i64, -1, 0, 1, 100] {
        assert_atomic_subtype(&t_lit_int(v), &t_numeric());
    }
}

#[test]
fn lit_float_in_numeric() {
    for v in [-1.5f64, 0.0, 1.5] {
        assert_atomic_subtype(&t_lit_float(v), &t_numeric());
    }
}

#[test]
fn bool_not_in_numeric() {
    assert_atomic_not_subtype(&t_bool(), &t_numeric());
}

#[test]
fn int_in_array_key() {
    assert_atomic_subtype(&t_int(), &t_array_key());
}

#[test]
fn string_in_array_key() {
    assert_atomic_subtype(&t_string(), &t_array_key());
}

#[test]
fn lit_int_in_array_key() {
    for v in [-100i64, 0, 1, 100] {
        assert_atomic_subtype(&t_lit_int(v), &t_array_key());
    }
}

#[test]
fn lit_string_in_array_key() {
    for s in ["", "x", "abc"] {
        assert_atomic_subtype(&t_lit_string(s), &t_array_key());
    }
}

#[test]
fn float_not_in_array_key() {
    assert_atomic_not_subtype(&t_float(), &t_array_key());
}

#[test]
fn bool_not_in_array_key() {
    assert_atomic_not_subtype(&t_bool(), &t_array_key());
}

#[test]
fn class_like_string_in_array_key() {
    assert_atomic_subtype(&t_class_string(), &t_array_key());
    assert_atomic_subtype(&t_interface_string(), &t_array_key());
    assert_atomic_subtype(&t_enum_string(), &t_array_key());
    assert_atomic_subtype(&t_trait_string(), &t_array_key());
}

#[test]
fn class_string_in_string() {
    assert_atomic_subtype(&t_class_string(), &t_string());
    assert_atomic_subtype(&t_interface_string(), &t_string());
    assert_atomic_subtype(&t_enum_string(), &t_string());
}

#[test]
fn lit_class_string_in_class_string() {
    assert_atomic_subtype(&t_lit_class_string("Foo"), &t_class_string());
}

#[test]
fn lit_class_string_in_string() {
    assert_atomic_subtype(&t_lit_class_string("Foo"), &t_string());
}

#[test]
fn int_in_scalar() {
    assert_atomic_subtype(&t_int(), &t_scalar());
}

#[test]
fn string_in_scalar() {
    assert_atomic_subtype(&t_string(), &t_scalar());
}

#[test]
fn float_in_scalar() {
    assert_atomic_subtype(&t_float(), &t_scalar());
}

#[test]
fn bool_in_scalar() {
    assert_atomic_subtype(&t_bool(), &t_scalar());
}

#[test]
fn true_in_scalar() {
    assert_atomic_subtype(&t_true(), &t_scalar());
}

#[test]
fn false_in_scalar() {
    assert_atomic_subtype(&t_false(), &t_scalar());
}

#[test]
fn numeric_in_scalar() {
    assert_atomic_subtype(&t_numeric(), &t_scalar());
}

#[test]
fn array_key_in_scalar() {
    assert_atomic_subtype(&t_array_key(), &t_scalar());
}

#[test]
fn class_string_in_scalar() {
    assert_atomic_subtype(&t_class_string(), &t_scalar());
}

#[test]
fn lit_int_in_scalar() {
    for v in [-100i64, 0, 100] {
        assert_atomic_subtype(&t_lit_int(v), &t_scalar());
    }
}

#[test]
fn lit_string_in_scalar() {
    for s in ["", "hi", "0"] {
        assert_atomic_subtype(&t_lit_string(s), &t_scalar());
    }
}

#[test]
fn lit_float_in_scalar() {
    assert_atomic_subtype(&t_lit_float(1.5), &t_scalar());
}

#[test]
fn null_not_in_scalar() {
    assert_atomic_not_subtype(&null(), &t_scalar());
}

#[test]
fn object_not_in_scalar() {
    assert_atomic_not_subtype(&t_object_any(), &t_scalar());
    assert_atomic_not_subtype(&t_named("Foo"), &t_scalar());
}

#[test]
fn array_not_in_scalar() {
    assert_atomic_not_subtype(&t_empty_array(), &t_scalar());
}

#[test]
fn resource_not_in_scalar() {
    assert_atomic_not_subtype(&t_resource(), &t_scalar());
}

#[test]
fn scalar_not_in_int() {
    assert_atomic_not_subtype(&t_scalar(), &t_int());
}

#[test]
fn scalar_not_in_string() {
    assert_atomic_not_subtype(&t_scalar(), &t_string());
}

#[test]
fn scalar_not_in_numeric() {
    assert_atomic_not_subtype(&t_scalar(), &t_numeric());
}

#[test]
fn scalar_not_in_array_key() {
    assert_atomic_not_subtype(&t_scalar(), &t_array_key());
}

#[test]
fn array_key_not_in_int() {
    assert_atomic_not_subtype(&t_array_key(), &t_int());
}

#[test]
fn array_key_not_in_string() {
    assert_atomic_not_subtype(&t_array_key(), &t_string());
}

#[test]
fn numeric_not_in_int() {
    assert_atomic_not_subtype(&t_numeric(), &t_int());
}

#[test]
fn numeric_not_in_float() {
    assert_atomic_not_subtype(&t_numeric(), &t_float());
}

#[test]
fn numeric_not_in_string() {
    assert_atomic_not_subtype(&t_numeric(), &t_string());
}

#[test]
fn cross_family_disjoint_int_string() {
    assert_atomic_not_subtype(&t_int(), &t_string());
    assert_atomic_not_subtype(&t_string(), &t_int());
}

#[test]
fn int_in_float_but_not_reverse() {
    assert_atomic_subtype(&t_int(), &t_float());
    assert_atomic_subtype(&t_lit_int(5), &t_float());
    assert_atomic_not_subtype(&t_float(), &t_int());
    assert_atomic_not_subtype(&t_lit_float(1.5), &t_int());
}

#[test]
fn cross_family_disjoint_int_bool() {
    assert_atomic_not_subtype(&t_int(), &t_bool());
    assert_atomic_not_subtype(&t_bool(), &t_int());
}

#[test]
fn cross_family_disjoint_string_bool() {
    assert_atomic_not_subtype(&t_string(), &t_bool());
    assert_atomic_not_subtype(&t_bool(), &t_string());
}

#[test]
fn string_float_disjoint() {
    assert_atomic_not_subtype(&t_string(), &t_float());
    assert_atomic_not_subtype(&t_float(), &t_string());
}

#[test]
fn cross_family_disjoint_float_bool() {
    assert_atomic_not_subtype(&t_float(), &t_bool());
    assert_atomic_not_subtype(&t_bool(), &t_float());
}

#[test]
fn lit_int_lit_float_disjoint() {
    assert_atomic_not_subtype(&t_lit_int(1), &t_lit_float(1.0));
    assert_atomic_not_subtype(&t_lit_float(1.0), &t_lit_int(1));
}

fn scalar_zoo() -> Vec<mago_codex::ttype::atomic::TAtomic> {
    vec![
        t_bool(),
        t_true(),
        t_false(),
        t_int(),
        t_lit_int(0),
        t_lit_int(42),
        t_int_unspec_lit(),
        t_positive_int(),
        t_negative_int(),
        t_int_range(0, 10),
        t_float(),
        t_lit_float(1.5),
        t_unspec_lit_float(),
        t_string(),
        t_lit_string("hi"),
        t_lit_string(""),
        t_non_empty_string(),
        t_numeric_string(),
        t_lower_string(),
        t_upper_string(),
        t_truthy_string(),
        t_class_string(),
        t_interface_string(),
        t_enum_string(),
        t_lit_class_string("Foo"),
        t_array_key(),
        t_numeric(),
        t_scalar(),
    ]
}
