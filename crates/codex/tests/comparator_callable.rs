mod comparator_common;

use comparator_common::*;

#[test]
fn callable_reflexive() {
    assert_atomic_subtype(&t_callable_mixed(), &t_callable_mixed());
}

#[test]
fn closure_reflexive() {
    assert_atomic_subtype(&t_closure_mixed(), &t_closure_mixed());
}

#[test]
fn closure_in_callable() {
    assert_atomic_subtype(&t_closure_mixed(), &t_callable_mixed());
}

#[test]
fn callable_not_in_closure() {
    assert_atomic_not_subtype(&t_callable_mixed(), &t_closure_mixed());
}

#[test]
fn callable_string_in_callable() {
    assert_atomic_subtype(&t_callable_string(), &t_callable_mixed());
}

#[test]
fn callable_not_in_callable_string() {
    assert_atomic_not_subtype(&t_callable_mixed(), &t_callable_string());
}

#[test]
fn callable_not_in_int() {
    assert_atomic_not_subtype(&t_callable_mixed(), &t_int());
}

#[test]
fn int_not_in_callable() {
    assert_atomic_not_subtype(&t_int(), &t_callable_mixed());
}

#[test]
fn callable_in_mixed() {
    assert_atomic_subtype(&t_callable_mixed(), &mixed());
}

#[test]
fn never_in_callable() {
    assert_atomic_subtype(&never(), &t_callable_mixed());
}

#[test]
fn callable_not_in_object() {
    assert_atomic_not_subtype(&t_callable_mixed(), &t_object_any());
}
