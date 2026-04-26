mod comparator_common;

use comparator_common::*;

#[test]
fn class_string_reflexive() {
    assert_atomic_subtype(&t_class_string(), &t_class_string());
}

#[test]
fn interface_string_reflexive() {
    assert_atomic_subtype(&t_interface_string(), &t_interface_string());
}

#[test]
fn enum_string_reflexive() {
    assert_atomic_subtype(&t_enum_string(), &t_enum_string());
}

#[test]
fn trait_string_reflexive() {
    assert_atomic_subtype(&t_trait_string(), &t_trait_string());
}

#[test]
fn lit_class_string_reflexive() {
    for n in ["Foo", "App\\Bar", "Vendor\\Pkg\\X"] {
        assert_atomic_subtype(&t_lit_class_string(n), &t_lit_class_string(n));
    }
}

#[test]
fn lit_class_string_in_class_string() {
    for n in ["Foo", "Bar", "App\\Service"] {
        assert_atomic_subtype(&t_lit_class_string(n), &t_class_string());
    }
}

#[test]
fn class_string_not_in_lit_class_string() {
    assert_atomic_not_subtype(&t_class_string(), &t_lit_class_string("Foo"));
}

#[test]
fn class_string_in_string() {
    assert_atomic_subtype(&t_class_string(), &t_string());
}

#[test]
fn interface_string_in_string() {
    assert_atomic_subtype(&t_interface_string(), &t_string());
}

#[test]
fn enum_string_in_string() {
    assert_atomic_subtype(&t_enum_string(), &t_string());
}

#[test]
fn trait_string_in_string() {
    assert_atomic_subtype(&t_trait_string(), &t_string());
}

#[test]
fn class_string_in_array_key() {
    assert_atomic_subtype(&t_class_string(), &t_array_key());
}

#[test]
fn class_string_in_scalar() {
    assert_atomic_subtype(&t_class_string(), &t_scalar());
}

#[test]
fn class_string_not_in_int() {
    assert_atomic_not_subtype(&t_class_string(), &t_int());
}

#[test]
fn class_string_not_in_numeric() {
    assert_atomic_not_subtype(&t_class_string(), &t_numeric());
}

#[test]
fn lit_class_string_in_string() {
    assert_atomic_subtype(&t_lit_class_string("Foo"), &t_string());
}

#[test]
fn lit_class_string_in_array_key() {
    assert_atomic_subtype(&t_lit_class_string("Foo"), &t_array_key());
}

#[test]
fn distinct_lit_class_strings_disjoint() {
    assert_atomic_not_subtype(&t_lit_class_string("Foo"), &t_lit_class_string("Bar"));
}

#[test]
fn many_lit_class_strings_in_class_string() {
    for i in 0..30 {
        let n = format!("Class_{i}");
        assert_atomic_subtype(&t_lit_class_string(&n), &t_class_string());
    }
}
