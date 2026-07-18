mod common;

use common::*;

#[test]
fn true_reflexive() {
    fixture(|f| {
        let true_atom = f.t_true();
        assert_atomic_subtype(f, true_atom, true_atom);
    });
}

#[test]
fn false_reflexive() {
    fixture(|f| {
        let false_atom = f.t_false();
        assert_atomic_subtype(f, false_atom, false_atom);
    });
}

#[test]
fn bool_reflexive() {
    fixture(|f| {
        let boolean = f.t_bool();
        assert_atomic_subtype(f, boolean, boolean);
    });
}

#[test]
fn true_in_bool() {
    fixture(|f| {
        let true_atom = f.t_true();
        let boolean = f.t_bool();
        assert_atomic_subtype(f, true_atom, boolean);
    });
}

#[test]
fn false_in_bool() {
    fixture(|f| {
        let false_atom = f.t_false();
        let boolean = f.t_bool();
        assert_atomic_subtype(f, false_atom, boolean);
    });
}

#[test]
fn bool_not_in_true() {
    fixture(|f| {
        let boolean = f.t_bool();
        let true_atom = f.t_true();
        assert_atomic_not_subtype(f, boolean, true_atom);
    });
}

#[test]
fn bool_not_in_false() {
    fixture(|f| {
        let boolean = f.t_bool();
        let false_atom = f.t_false();
        assert_atomic_not_subtype(f, boolean, false_atom);
    });
}

#[test]
fn true_not_in_false() {
    fixture(|f| {
        let true_atom = f.t_true();
        let false_atom = f.t_false();
        assert_atomic_not_subtype(f, true_atom, false_atom);
    });
}

#[test]
fn false_not_in_true() {
    fixture(|f| {
        let false_atom = f.t_false();
        let true_atom = f.t_true();
        assert_atomic_not_subtype(f, false_atom, true_atom);
    });
}

#[test]
fn bool_in_scalar() {
    fixture(|f| {
        let boolean = f.t_bool();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, boolean, scalar);
    });
}

#[test]
fn true_in_scalar() {
    fixture(|f| {
        let true_atom = f.t_true();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, true_atom, scalar);
    });
}

#[test]
fn false_in_scalar() {
    fixture(|f| {
        let false_atom = f.t_false();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, false_atom, scalar);
    });
}

#[test]
fn scalar_not_in_bool() {
    fixture(|f| {
        let scalar = f.t_scalar();
        let boolean = f.t_bool();
        assert_atomic_not_subtype(f, scalar, boolean);
    });
}

#[test]
fn bool_not_in_int() {
    fixture(|f| {
        let boolean = f.t_bool();
        let int = f.t_int();
        assert_atomic_not_subtype(f, boolean, int);
    });
}

#[test]
fn bool_not_in_string() {
    fixture(|f| {
        let boolean = f.t_bool();
        let string = f.t_string();
        assert_atomic_not_subtype(f, boolean, string);
    });
}

#[test]
fn bool_not_in_float() {
    fixture(|f| {
        let boolean = f.t_bool();
        let float = f.t_float();
        assert_atomic_not_subtype(f, boolean, float);
    });
}

#[test]
fn bool_not_in_numeric() {
    fixture(|f| {
        let boolean = f.t_bool();
        let numeric = f.t_numeric();
        assert_atomic_not_subtype(f, boolean, numeric);
    });
}

#[test]
fn bool_not_in_array_key() {
    fixture(|f| {
        let boolean = f.t_bool();
        let array_key = f.t_array_key();
        assert_atomic_not_subtype(f, boolean, array_key);
    });
}

#[test]
fn bool_not_in_object() {
    fixture(|f| {
        let boolean = f.t_bool();
        let object = f.t_object_any();
        assert_atomic_not_subtype(f, boolean, object);
    });
}

#[test]
fn bool_not_in_null() {
    fixture(|f| {
        let boolean = f.t_bool();
        let null = f.null();
        assert_atomic_not_subtype(f, boolean, null);
    });
}
