mod common;

use common::*;

#[test]
fn iterable_reflexive() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let iterable = f.t_iterable(int_type, int_type);
        assert_atomic_subtype(f, iterable, iterable);
    });
}

#[test]
fn list_in_iterable_int_int() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let list = f.t_list(int_type, false);
        let iterable = f.t_iterable(int_type, int_type);
        assert_atomic_subtype(f, list, iterable);
    });
}

#[test]
fn list_in_iterable_with_array_key() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let array_key_type = f.u(f.t_array_key());
        let list = f.t_list(int_type, false);
        let iterable = f.t_iterable(array_key_type, int_type);
        assert_atomic_subtype(f, list, iterable);
    });
}

#[test]
fn keyed_in_iterable_string_value() {
    fixture(|f| {
        let string_type = f.u(f.t_string());
        let int_type = f.u(f.t_int());
        let keyed = f.t_keyed_unsealed(string_type, int_type, false);
        let iterable = f.t_iterable(string_type, int_type);
        assert_atomic_subtype(f, keyed, iterable);
    });
}

#[test]
fn list_with_lit_in_iterable_general() {
    fixture(|f| {
        let literal_type = f.ui(5);
        let int_type = f.u(f.t_int());
        let list = f.t_list(literal_type, false);
        let iterable = f.t_iterable(int_type, int_type);
        assert_atomic_subtype(f, list, iterable);
    });
}

#[test]
fn iterable_not_in_list() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let iterable = f.t_iterable(int_type, int_type);
        let list = f.t_list(int_type, false);
        assert_atomic_not_subtype(f, iterable, list);
    });
}

#[test]
fn iterable_not_in_keyed() {
    fixture(|f| {
        let string_type = f.u(f.t_string());
        let int_type = f.u(f.t_int());
        let iterable = f.t_iterable(string_type, int_type);
        let keyed = f.t_keyed_unsealed(string_type, int_type, false);
        assert_atomic_not_subtype(f, iterable, keyed);
    });
}

#[test]
fn iterable_value_covariance() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let literal_type = f.ui(5);
        let literal_iterable = f.t_iterable(int_type, literal_type);
        let general_iterable = f.t_iterable(int_type, int_type);
        assert!(atomic_is_contained(f, literal_iterable, general_iterable, &empty_world()));
        assert!(!atomic_is_contained(f, general_iterable, literal_iterable, &empty_world()));
    });
}

#[test]
fn iterable_disjoint_value() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let string_type = f.u(f.t_string());
        let int_iterable = f.t_iterable(int_type, int_type);
        let string_iterable = f.t_iterable(int_type, string_type);
        assert_atomic_not_subtype(f, int_iterable, string_iterable);
    });
}

#[test]
fn iterable_in_mixed() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let iterable = f.t_iterable(int_type, int_type);
        let mixed = f.mixed();
        assert_atomic_subtype(f, iterable, mixed);
    });
}

#[test]
fn never_in_iterable() {
    fixture(|f| {
        let int_type = f.u(f.t_int());
        let iterable = f.t_iterable(int_type, int_type);
        let never = f.never();
        assert_atomic_subtype(f, never, iterable);
    });
}

#[test]
fn empty_array_in_iterable() {
    fixture(|f| {
        let array_key_type = f.u(f.t_array_key());
        let mixed_type = f.u(f.mixed());
        let iterable = f.t_iterable(array_key_type, mixed_type);
        let empty_array = f.t_empty_array();
        assert_atomic_subtype(f, empty_array, iterable);
    });
}
