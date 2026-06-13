mod common;

use common::*;

#[test]
fn null_reflexive() {
    fixture(|f| {
        let null = f.null();
        assert_atomic_subtype(f, null, null);
    });
}

#[test]
fn void_reflexive() {
    fixture(|f| {
        let void = f.void();
        assert_atomic_subtype(f, void, void);
    });
}

#[test]
fn never_reflexive() {
    fixture(|f| {
        let never = f.never();
        assert_atomic_subtype(f, never, never);
    });
}

#[test]
fn never_in_int() {
    fixture(|f| {
        let never = f.never();
        let int = f.t_int();
        assert_atomic_subtype(f, never, int);
    });
}

#[test]
fn never_in_string() {
    fixture(|f| {
        let never = f.never();
        let string = f.t_string();
        assert_atomic_subtype(f, never, string);
    });
}

#[test]
fn never_in_float() {
    fixture(|f| {
        let never = f.never();
        let float = f.t_float();
        assert_atomic_subtype(f, never, float);
    });
}

#[test]
fn never_in_bool() {
    fixture(|f| {
        let never = f.never();
        let bool_atom = f.t_bool();
        assert_atomic_subtype(f, never, bool_atom);
    });
}

#[test]
fn never_in_null() {
    fixture(|f| {
        let never = f.never();
        let null = f.null();
        assert_atomic_subtype(f, never, null);
    });
}

#[test]
fn never_in_void() {
    fixture(|f| {
        let never = f.never();
        let void = f.void();
        assert_atomic_subtype(f, never, void);
    });
}

#[test]
fn never_in_object() {
    fixture(|f| {
        let never = f.never();
        let object = f.t_object_any();
        let foo = f.t_named("Foo");
        assert_atomic_subtype(f, never, object);
        assert_atomic_subtype(f, never, foo);
    });
}

#[test]
fn never_in_array() {
    fixture(|f| {
        let never = f.never();
        let empty_array = f.t_empty_array();
        assert_atomic_subtype(f, never, empty_array);
    });
}

#[test]
fn never_in_resource() {
    fixture(|f| {
        let never = f.never();
        let resource = f.t_resource();
        assert_atomic_subtype(f, never, resource);
    });
}

#[test]
fn never_in_mixed() {
    fixture(|f| {
        let never = f.never();
        let mixed = f.mixed();
        assert_atomic_subtype(f, never, mixed);
    });
}

#[test]
fn never_in_scalar() {
    fixture(|f| {
        let never = f.never();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, never, scalar);
    });
}

#[test]
fn never_in_array_key() {
    fixture(|f| {
        let never = f.never();
        let array_key = f.t_array_key();
        assert_atomic_subtype(f, never, array_key);
    });
}

#[test]
fn never_in_numeric() {
    fixture(|f| {
        let never = f.never();
        let numeric = f.t_numeric();
        assert_atomic_subtype(f, never, numeric);
    });
}

#[test]
fn anything_not_in_never() {
    fixture(|f| {
        let never = f.never();
        for atom in [
            f.t_int(),
            f.t_string(),
            f.t_float(),
            f.t_bool(),
            f.null(),
            f.void(),
            f.t_object_any(),
            f.t_resource(),
            f.mixed(),
            f.t_scalar(),
            f.t_array_key(),
            f.t_numeric(),
        ] {
            assert_atomic_not_subtype(f, atom, never);
        }
    });
}

#[test]
fn null_not_in_int() {
    fixture(|f| {
        let null = f.null();
        let int = f.t_int();
        assert_atomic_not_subtype(f, null, int);
    });
}

#[test]
fn null_not_in_string() {
    fixture(|f| {
        let null = f.null();
        let string = f.t_string();
        assert_atomic_not_subtype(f, null, string);
    });
}

#[test]
fn null_not_in_float() {
    fixture(|f| {
        let null = f.null();
        let float = f.t_float();
        assert_atomic_not_subtype(f, null, float);
    });
}

#[test]
fn null_not_in_bool() {
    fixture(|f| {
        let null = f.null();
        let bool_atom = f.t_bool();
        assert_atomic_not_subtype(f, null, bool_atom);
    });
}

#[test]
fn null_not_in_object() {
    fixture(|f| {
        let null = f.null();
        let object = f.t_object_any();
        assert_atomic_not_subtype(f, null, object);
    });
}

#[test]
fn null_not_in_array() {
    fixture(|f| {
        let null = f.null();
        let empty_array = f.t_empty_array();
        assert_atomic_not_subtype(f, null, empty_array);
    });
}

#[test]
fn null_in_mixed() {
    fixture(|f| {
        let null = f.null();
        let mixed = f.mixed();
        assert_atomic_subtype(f, null, mixed);
    });
}

#[test]
fn null_not_in_scalar() {
    fixture(|f| {
        let null = f.null();
        let scalar = f.t_scalar();
        assert_atomic_not_subtype(f, null, scalar);
    });
}

#[test]
fn void_not_in_int() {
    fixture(|f| {
        let void = f.void();
        let int = f.t_int();
        assert_atomic_not_subtype(f, void, int);
    });
}

#[test]
fn void_not_in_string() {
    fixture(|f| {
        let void = f.void();
        let string = f.t_string();
        assert_atomic_not_subtype(f, void, string);
    });
}

#[test]
fn void_in_mixed() {
    fixture(|f| {
        let void = f.void();
        let mixed = f.mixed();
        assert_atomic_subtype(f, void, mixed);
    });
}

#[test]
fn void_in_null() {
    fixture(|f| {
        let void = f.void();
        let null = f.null();
        assert_atomic_subtype(f, void, null);
    });
}

#[test]
fn null_in_void() {
    fixture(|f| {
        let null = f.null();
        let void = f.void();
        assert_atomic_subtype(f, null, void);
    });
}

#[test]
fn null_not_in_never() {
    fixture(|f| {
        let null = f.null();
        let never = f.never();
        assert_atomic_not_subtype(f, null, never);
    });
}

#[test]
fn void_not_in_never() {
    fixture(|f| {
        let void = f.void();
        let never = f.never();
        assert_atomic_not_subtype(f, void, never);
    });
}
