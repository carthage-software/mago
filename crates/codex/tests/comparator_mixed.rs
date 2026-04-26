mod comparator_common;

use comparator_common::*;

#[test]
fn mixed_reflexive() {
    assert_atomic_subtype(&mixed(), &mixed());
}

#[test]
fn truthy_mixed_reflexive() {
    assert_atomic_subtype(&mixed_truthy(), &mixed_truthy());
}

#[test]
fn falsy_mixed_reflexive() {
    assert_atomic_subtype(&mixed_falsy(), &mixed_falsy());
}

#[test]
fn nonnull_mixed_reflexive() {
    assert_atomic_subtype(&mixed_nonnull(), &mixed_nonnull());
}

#[test]
fn truthy_mixed_in_mixed() {
    assert_atomic_subtype(&mixed_truthy(), &mixed());
}

#[test]
fn falsy_mixed_in_mixed() {
    assert_atomic_subtype(&mixed_falsy(), &mixed());
}

#[test]
fn nonnull_mixed_in_mixed() {
    assert_atomic_subtype(&mixed_nonnull(), &mixed());
}

#[test]
fn int_in_mixed() {
    assert_atomic_subtype(&t_int(), &mixed());
}

#[test]
fn string_in_mixed() {
    assert_atomic_subtype(&t_string(), &mixed());
}

#[test]
fn float_in_mixed() {
    assert_atomic_subtype(&t_float(), &mixed());
}

#[test]
fn bool_in_mixed() {
    assert_atomic_subtype(&t_bool(), &mixed());
}

#[test]
fn null_in_mixed() {
    assert_atomic_subtype(&null(), &mixed());
}

#[test]
fn void_in_mixed() {
    assert_atomic_subtype(&void(), &mixed());
}

#[test]
fn object_in_mixed() {
    assert_atomic_subtype(&t_object_any(), &mixed());
    assert_atomic_subtype(&t_named("Foo"), &mixed());
}

#[test]
fn array_in_mixed() {
    assert_atomic_subtype(&t_empty_array(), &mixed());
    assert_atomic_subtype(&t_list(u(t_int()), false), &mixed());
}

#[test]
fn resource_in_mixed() {
    assert_atomic_subtype(&t_resource(), &mixed());
}

#[test]
fn mixed_not_in_int() {
    assert_atomic_not_subtype(&mixed(), &t_int());
}

#[test]
fn mixed_not_in_string() {
    assert_atomic_not_subtype(&mixed(), &t_string());
}

#[test]
fn mixed_not_in_float() {
    assert_atomic_not_subtype(&mixed(), &t_float());
}

#[test]
fn mixed_not_in_bool() {
    assert_atomic_not_subtype(&mixed(), &t_bool());
}

#[test]
fn mixed_not_in_null() {
    assert_atomic_not_subtype(&mixed(), &null());
}

#[test]
fn mixed_not_in_object() {
    assert_atomic_not_subtype(&mixed(), &t_object_any());
}

#[test]
fn mixed_not_in_array() {
    assert_atomic_not_subtype(&mixed(), &t_empty_array());
}

#[test]
fn never_in_mixed_variants() {
    assert_atomic_subtype(&never(), &mixed());
    assert_atomic_subtype(&never(), &mixed_truthy());
    assert_atomic_subtype(&never(), &mixed_falsy());
    assert_atomic_subtype(&never(), &mixed_nonnull());
}
