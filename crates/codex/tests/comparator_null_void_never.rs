mod comparator_common;

use comparator_common::*;

#[test]
fn null_reflexive() {
    assert_atomic_subtype(&null(), &null());
}

#[test]
fn void_reflexive() {
    assert_atomic_subtype(&void(), &void());
}

#[test]
fn never_reflexive() {
    assert_atomic_subtype(&never(), &never());
}

#[test]
fn never_in_int() {
    assert_atomic_subtype(&never(), &t_int());
}

#[test]
fn never_in_string() {
    assert_atomic_subtype(&never(), &t_string());
}

#[test]
fn never_in_float() {
    assert_atomic_subtype(&never(), &t_float());
}

#[test]
fn never_in_bool() {
    assert_atomic_subtype(&never(), &t_bool());
}

#[test]
fn never_in_null() {
    assert_atomic_subtype(&never(), &null());
}

#[test]
fn never_in_void() {
    assert_atomic_subtype(&never(), &void());
}

#[test]
fn never_in_object() {
    assert_atomic_subtype(&never(), &t_object_any());
    assert_atomic_subtype(&never(), &t_named("Foo"));
}

#[test]
fn never_in_array() {
    assert_atomic_subtype(&never(), &t_empty_array());
}

#[test]
fn never_in_resource() {
    assert_atomic_subtype(&never(), &t_resource());
}

#[test]
fn never_in_mixed() {
    assert_atomic_subtype(&never(), &mixed());
}

#[test]
fn never_in_scalar() {
    assert_atomic_subtype(&never(), &t_scalar());
}

#[test]
fn never_in_array_key() {
    assert_atomic_subtype(&never(), &t_array_key());
}

#[test]
fn never_in_numeric() {
    assert_atomic_subtype(&never(), &t_numeric());
}

#[test]
fn anything_not_in_never() {
    for atom in [
        t_int(),
        t_string(),
        t_float(),
        t_bool(),
        null(),
        void(),
        t_object_any(),
        t_resource(),
        mixed(),
        t_scalar(),
        t_array_key(),
        t_numeric(),
    ] {
        assert_atomic_not_subtype(&atom, &never());
    }
}

#[test]
fn null_not_in_int() {
    assert_atomic_not_subtype(&null(), &t_int());
}

#[test]
fn null_not_in_string() {
    assert_atomic_not_subtype(&null(), &t_string());
}

#[test]
fn null_not_in_float() {
    assert_atomic_not_subtype(&null(), &t_float());
}

#[test]
fn null_not_in_bool() {
    assert_atomic_not_subtype(&null(), &t_bool());
}

#[test]
fn null_not_in_object() {
    assert_atomic_not_subtype(&null(), &t_object_any());
}

#[test]
fn null_not_in_array() {
    assert_atomic_not_subtype(&null(), &t_empty_array());
}

#[test]
fn null_in_mixed() {
    assert_atomic_subtype(&null(), &mixed());
}

#[test]
fn null_not_in_scalar() {
    assert_atomic_not_subtype(&null(), &t_scalar());
}

#[test]
fn void_not_in_int() {
    assert_atomic_not_subtype(&void(), &t_int());
}

#[test]
fn void_not_in_string() {
    assert_atomic_not_subtype(&void(), &t_string());
}

#[test]
fn void_in_mixed() {
    assert_atomic_subtype(&void(), &mixed());
}

#[test]
fn void_not_in_null() {
    assert_atomic_not_subtype(&void(), &null());
}

#[test]
fn null_not_in_void() {
    assert_atomic_not_subtype(&null(), &void());
}

#[test]
fn null_not_in_never() {
    assert_atomic_not_subtype(&null(), &never());
}

#[test]
fn void_not_in_never() {
    assert_atomic_not_subtype(&void(), &never());
}
