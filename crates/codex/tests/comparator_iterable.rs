mod comparator_common;

use comparator_common::*;

#[test]
fn iterable_reflexive() {
    let it = t_iterable(u(t_int()), u(t_int()));
    assert_atomic_subtype(&it.clone(), &it);
}

#[test]
fn list_in_iterable_int_int() {
    assert_atomic_subtype(&t_list(u(t_int()), false), &t_iterable(u(t_int()), u(t_int())));
}

#[test]
fn list_in_iterable_with_array_key() {
    assert_atomic_subtype(&t_list(u(t_int()), false), &t_iterable(u(t_array_key()), u(t_int())));
}

#[test]
fn keyed_in_iterable_string_value() {
    assert_atomic_subtype(&t_keyed_unsealed(u(t_string()), u(t_int()), false), &t_iterable(u(t_string()), u(t_int())));
}

#[test]
fn list_with_lit_in_iterable_general() {
    assert_atomic_subtype(&t_list(u(t_lit_int(5)), false), &t_iterable(u(t_int()), u(t_int())));
}

#[test]
fn iterable_not_in_list() {
    assert_atomic_not_subtype(&t_iterable(u(t_int()), u(t_int())), &t_list(u(t_int()), false));
}

#[test]
fn iterable_not_in_keyed() {
    assert_atomic_not_subtype(
        &t_iterable(u(t_string()), u(t_int())),
        &t_keyed_unsealed(u(t_string()), u(t_int()), false),
    );
}

#[test]
fn iterable_value_covariance() {
    let cb = empty_codebase();
    let lit = t_iterable(u(t_int()), u(t_lit_int(5)));
    let general = t_iterable(u(t_int()), u(t_int()));
    assert!(atomic_is_contained(&lit, &general, &cb));
    assert!(!atomic_is_contained(&general, &lit, &cb));
}

#[test]
fn iterable_disjoint_value() {
    assert_atomic_not_subtype(&t_iterable(u(t_int()), u(t_int())), &t_iterable(u(t_int()), u(t_string())));
}

#[test]
fn iterable_in_mixed() {
    assert_atomic_subtype(&t_iterable(u(t_int()), u(t_int())), &mixed());
}

#[test]
fn never_in_iterable() {
    assert_atomic_subtype(&never(), &t_iterable(u(t_int()), u(t_int())));
}

#[test]
fn empty_array_in_iterable() {
    assert_atomic_subtype(&t_empty_array(), &t_iterable(u(t_array_key()), u(mixed())));
}
