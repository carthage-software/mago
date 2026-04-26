mod comparator_common;

use comparator_common::*;

#[test]
fn smoke_int_int() {
    assert_subtype(&u(t_int()), &u(t_int()));
}

#[test]
fn smoke_lit_int_in_int() {
    assert_subtype(&u(t_lit_int(5)), &u(t_int()));
}

#[test]
fn smoke_int_not_in_string() {
    assert_not_subtype(&u(t_int()), &u(t_string()));
}

#[test]
fn smoke_int_in_mixed() {
    assert_subtype(&u(t_int()), &u(mixed()));
}

#[test]
fn smoke_never_in_anything() {
    assert_subtype(&u(never()), &u(t_int()));
    assert_subtype(&u(never()), &u(t_string()));
    assert_subtype(&u(never()), &u(mixed()));
}
