mod common;

use common::*;

#[test]
fn float_reflexive() {
    fixture(|f| {
        let float = f.t_float();
        assert_atomic_subtype(f, float, float);
    });
}

#[test]
fn lit_float_reflexive() {
    fixture(|f| {
        for value in [-100.0f64, -1.5, 0.0, 1.5, 1e10] {
            let literal = f.t_lit_float(value);
            assert_atomic_subtype(f, literal, literal);
        }
    });
}

#[test]
fn lit_in_float() {
    fixture(|f| {
        let float = f.t_float();
        for value in [-3.25f64, 0.0, 1.5, 100.0, 1e10] {
            let literal = f.t_lit_float(value);
            assert_atomic_subtype(f, literal, float);
        }
    });
}

#[test]
fn float_not_in_lit() {
    fixture(|f| {
        let float = f.t_float();
        for value in [0.0f64, 1.5, -3.25] {
            let literal = f.t_lit_float(value);
            assert_atomic_not_subtype(f, float, literal);
        }
    });
}

#[test]
fn distinct_lits_disjoint() {
    fixture(|f| {
        for (a, b) in [(0.0f64, 1.0), (1.5, 2.5), (-1.0, 1.0), (3.25, -3.25)] {
            let a_literal = f.t_lit_float(a);
            let b_literal = f.t_lit_float(b);
            assert_atomic_not_subtype(f, a_literal, b_literal);
            assert_atomic_not_subtype(f, b_literal, a_literal);
        }
    });
}

#[test]
fn unspec_lit_in_float() {
    fixture(|f| {
        let unspecified = f.t_unspec_lit_float();
        let float = f.t_float();
        assert_atomic_subtype(f, unspecified, float);
    });
}

#[test]
fn lit_in_unspec_lit() {
    fixture(|f| {
        let unspecified = f.t_unspec_lit_float();
        for value in [0.0f64, 1.5, -3.25] {
            let literal = f.t_lit_float(value);
            assert_atomic_subtype(f, literal, unspecified);
        }
    });
}

#[test]
fn float_in_numeric() {
    fixture(|f| {
        let float = f.t_float();
        let numeric = f.t_numeric();
        assert_atomic_subtype(f, float, numeric);
    });
}

#[test]
fn lit_float_in_numeric() {
    fixture(|f| {
        let numeric = f.t_numeric();
        for value in [0.0f64, 1.5, -3.25] {
            let literal = f.t_lit_float(value);
            assert_atomic_subtype(f, literal, numeric);
        }
    });
}

#[test]
fn float_in_scalar() {
    fixture(|f| {
        let float = f.t_float();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, float, scalar);
    });
}

#[test]
fn float_not_in_int() {
    fixture(|f| {
        let float = f.t_float();
        let int = f.t_int();
        assert_atomic_not_subtype(f, float, int);
    });
}

#[test]
fn float_not_in_string() {
    fixture(|f| {
        let float = f.t_float();
        let string = f.t_string();
        assert_atomic_not_subtype(f, float, string);
    });
}

#[test]
fn float_not_in_bool() {
    fixture(|f| {
        let float = f.t_float();
        let boolean = f.t_bool();
        assert_atomic_not_subtype(f, float, boolean);
    });
}

#[test]
fn float_not_in_array_key() {
    fixture(|f| {
        let float = f.t_float();
        let array_key = f.t_array_key();
        assert_atomic_not_subtype(f, float, array_key);
    });
}

#[test]
fn many_lit_in_float() {
    fixture(|f| {
        let float = f.t_float();
        for index in 0..200 {
            let value = f64::from(index).mul_add(0.5, -50.0);
            let literal = f.t_lit_float(value);
            assert_atomic_subtype(f, literal, float);
        }
    });
}
