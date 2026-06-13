mod common;

use common::*;

#[test]
fn callable_reflexive() {
    fixture(|f| {
        let callable = f.t_callable_mixed();
        assert_atomic_subtype(f, callable, callable);
    });
}

#[test]
fn closure_reflexive() {
    fixture(|f| {
        let closure = f.t_closure_mixed();
        assert_atomic_subtype(f, closure, closure);
    });
}

#[test]
fn closure_in_callable() {
    fixture(|f| {
        let closure = f.t_closure_mixed();
        let callable = f.t_callable_mixed();
        assert_atomic_subtype(f, closure, callable);
    });
}

#[test]
fn callable_not_in_closure() {
    fixture(|f| {
        let callable = f.t_callable_mixed();
        let closure = f.t_closure_mixed();
        assert_atomic_not_subtype(f, callable, closure);
    });
}

#[test]
fn callable_string_in_callable() {
    fixture(|f| {
        let callable_string = f.t_callable_string();
        let callable = f.t_callable_mixed();
        assert_atomic_subtype(f, callable_string, callable);
    });
}

#[test]
fn callable_not_in_callable_string() {
    fixture(|f| {
        let callable = f.t_callable_mixed();
        let callable_string = f.t_callable_string();
        assert_atomic_not_subtype(f, callable, callable_string);
    });
}

#[test]
fn callable_not_in_int() {
    fixture(|f| {
        let callable = f.t_callable_mixed();
        let int = f.t_int();
        assert_atomic_not_subtype(f, callable, int);
    });
}

#[test]
fn int_not_in_callable() {
    fixture(|f| {
        let int = f.t_int();
        let callable = f.t_callable_mixed();
        assert_atomic_not_subtype(f, int, callable);
    });
}

#[test]
fn callable_in_mixed() {
    fixture(|f| {
        let callable = f.t_callable_mixed();
        let mixed = f.mixed();
        assert_atomic_subtype(f, callable, mixed);
    });
}

#[test]
fn never_in_callable() {
    fixture(|f| {
        let never = f.never();
        let callable = f.t_callable_mixed();
        assert_atomic_subtype(f, never, callable);
    });
}

#[test]
fn callable_not_in_object() {
    fixture(|f| {
        let callable = f.t_callable_mixed();
        let object = f.t_object_any();
        assert_atomic_not_subtype(f, callable, object);
    });
}
