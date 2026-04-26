mod comparator_common;

use comparator_common::*;

#[test]
fn true_reflexive() {
    assert_atomic_subtype(&t_true(), &t_true());
}

#[test]
fn false_reflexive() {
    assert_atomic_subtype(&t_false(), &t_false());
}

#[test]
fn bool_reflexive() {
    assert_atomic_subtype(&t_bool(), &t_bool());
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
fn scalar_not_in_bool() {
    assert_atomic_not_subtype(&t_scalar(), &t_bool());
}

#[test]
fn bool_not_in_int() {
    assert_atomic_not_subtype(&t_bool(), &t_int());
}

#[test]
fn bool_not_in_string() {
    assert_atomic_not_subtype(&t_bool(), &t_string());
}

#[test]
fn bool_not_in_float() {
    assert_atomic_not_subtype(&t_bool(), &t_float());
}

#[test]
fn bool_not_in_numeric() {
    assert_atomic_not_subtype(&t_bool(), &t_numeric());
}

#[test]
fn bool_not_in_array_key() {
    assert_atomic_not_subtype(&t_bool(), &t_array_key());
}

#[test]
fn bool_not_in_object() {
    assert_atomic_not_subtype(&t_bool(), &t_object_any());
}

#[test]
fn bool_not_in_null() {
    assert_atomic_not_subtype(&t_bool(), &null());
}
