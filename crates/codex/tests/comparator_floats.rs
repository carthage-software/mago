#![allow(clippy::approx_constant)]

mod comparator_common;

use comparator_common::*;

#[test]
fn float_reflexive() {
    assert_atomic_subtype(&t_float(), &t_float());
}

#[test]
fn lit_float_reflexive() {
    for v in [-100.0_f64, -1.5, 0.0, 1.5, 1e10] {
        assert_atomic_subtype(&t_lit_float(v), &t_lit_float(v));
    }
}

#[test]
fn lit_in_float() {
    for v in [-3.14_f64, 0.0, 1.5, 100.0, 1e10] {
        assert_atomic_subtype(&t_lit_float(v), &t_float());
    }
}

#[test]
fn float_not_in_lit() {
    for v in [0.0_f64, 1.5, -3.14] {
        assert_atomic_not_subtype(&t_float(), &t_lit_float(v));
    }
}

#[test]
fn distinct_lits_disjoint() {
    for (a, b) in [(0.0_f64, 1.0), (1.5, 2.5), (-1.0, 1.0), (3.14, -3.14)] {
        assert_atomic_not_subtype(&t_lit_float(a), &t_lit_float(b));
        assert_atomic_not_subtype(&t_lit_float(b), &t_lit_float(a));
    }
}

#[test]
fn unspec_lit_in_float() {
    assert_atomic_subtype(&t_unspec_lit_float(), &t_float());
}

#[test]
fn lit_in_unspec_lit() {
    for v in [0.0_f64, 1.5, -3.14] {
        assert_atomic_subtype(&t_lit_float(v), &t_unspec_lit_float());
    }
}

#[test]
fn float_in_numeric() {
    assert_atomic_subtype(&t_float(), &t_numeric());
}

#[test]
fn lit_float_in_numeric() {
    for v in [0.0_f64, 1.5, -3.14] {
        assert_atomic_subtype(&t_lit_float(v), &t_numeric());
    }
}

#[test]
fn float_in_scalar() {
    assert_atomic_subtype(&t_float(), &t_scalar());
}

#[test]
fn float_not_in_int() {
    assert_atomic_not_subtype(&t_float(), &t_int());
}

#[test]
fn float_not_in_string() {
    assert_atomic_not_subtype(&t_float(), &t_string());
}

#[test]
fn float_not_in_bool() {
    assert_atomic_not_subtype(&t_float(), &t_bool());
}

#[test]
fn float_not_in_array_key() {
    assert_atomic_not_subtype(&t_float(), &t_array_key());
}

#[test]
fn many_lit_in_float() {
    for i in 0..200 {
        let v = f64::from(i) * 0.5 - 50.0;
        assert_atomic_subtype(&t_lit_float(v), &t_float());
    }
}
