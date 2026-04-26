mod comparator_common;

use comparator_common::*;

#[test]
fn resource_reflexive() {
    assert_atomic_subtype(&t_resource(), &t_resource());
}

#[test]
fn open_reflexive() {
    assert_atomic_subtype(&t_open_resource(), &t_open_resource());
}

#[test]
fn closed_reflexive() {
    assert_atomic_subtype(&t_closed_resource(), &t_closed_resource());
}

#[test]
fn open_in_resource() {
    assert_atomic_subtype(&t_open_resource(), &t_resource());
}

#[test]
fn closed_in_resource() {
    assert_atomic_subtype(&t_closed_resource(), &t_resource());
}

#[test]
fn resource_not_in_open() {
    assert_atomic_not_subtype(&t_resource(), &t_open_resource());
}

#[test]
fn resource_not_in_closed() {
    assert_atomic_not_subtype(&t_resource(), &t_closed_resource());
}

#[test]
fn open_not_in_closed() {
    assert_atomic_not_subtype(&t_open_resource(), &t_closed_resource());
}

#[test]
fn closed_not_in_open() {
    assert_atomic_not_subtype(&t_closed_resource(), &t_open_resource());
}

#[test]
fn resource_not_in_int() {
    assert_atomic_not_subtype(&t_resource(), &t_int());
}

#[test]
fn resource_not_in_string() {
    assert_atomic_not_subtype(&t_resource(), &t_string());
}

#[test]
fn resource_not_in_object() {
    assert_atomic_not_subtype(&t_resource(), &t_object_any());
}

#[test]
fn resource_not_in_array() {
    assert_atomic_not_subtype(&t_resource(), &t_empty_array());
}

#[test]
fn resource_in_mixed() {
    assert_atomic_subtype(&t_resource(), &mixed());
    assert_atomic_subtype(&t_open_resource(), &mixed());
    assert_atomic_subtype(&t_closed_resource(), &mixed());
}

#[test]
fn never_in_resource() {
    assert_atomic_subtype(&never(), &t_resource());
    assert_atomic_subtype(&never(), &t_open_resource());
    assert_atomic_subtype(&never(), &t_closed_resource());
}
