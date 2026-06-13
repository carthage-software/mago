mod common;

use common::*;

#[test]
fn smoke_int_int() {
    fixture(|f| {
        let int = f.t_int();
        let int_type = f.u(int);
        assert_subtype(f, int_type, int_type);
    });
}

#[test]
fn smoke_lit_int_in_int() {
    fixture(|f| {
        let literal = f.t_lit_int(5);
        let int = f.t_int();
        let literal_type = f.u(literal);
        let int_type = f.u(int);
        assert_subtype(f, literal_type, int_type);
    });
}

#[test]
fn smoke_int_not_in_string() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let int_type = f.u(int);
        let string_type = f.u(string);
        assert_not_subtype(f, int_type, string_type);
    });
}

#[test]
fn smoke_int_in_mixed() {
    fixture(|f| {
        let int = f.t_int();
        let mixed = f.mixed();
        let int_type = f.u(int);
        let mixed_type = f.u(mixed);
        assert_subtype(f, int_type, mixed_type);
    });
}

#[test]
fn smoke_never_in_anything() {
    fixture(|f| {
        let never = f.never();
        let int = f.t_int();
        let string = f.t_string();
        let mixed = f.mixed();
        let never_type = f.u(never);
        let int_type = f.u(int);
        let string_type = f.u(string);
        let mixed_type = f.u(mixed);
        assert_subtype(f, never_type, int_type);
        assert_subtype(f, never_type, string_type);
        assert_subtype(f, never_type, mixed_type);
    });
}
