mod common;

use common::*;

use mago_oracle::ty::Atom;

fn scalar_zoo<'arena>(f: &mut Fixture<'_, 'arena>) -> Vec<Atom<'arena>> {
    vec![
        f.t_bool(),
        f.t_true(),
        f.t_false(),
        f.t_int(),
        f.t_lit_int(0),
        f.t_lit_int(42),
        f.t_int_unspec_lit(),
        f.t_positive_int(),
        f.t_negative_int(),
        f.t_int_range(0, 10),
        f.t_float(),
        f.t_lit_float(1.5),
        f.t_unspec_lit_float(),
        f.t_string(),
        f.t_lit_string("hi"),
        f.t_lit_string(""),
        f.t_non_empty_string(),
        f.t_numeric_string(),
        f.t_lower_string(),
        f.t_upper_string(),
        f.t_truthy_string(),
        f.t_class_string(),
        f.t_interface_string(),
        f.t_enum_string(),
        f.t_lit_class_string("Foo"),
        f.t_array_key(),
        f.t_numeric(),
        f.t_scalar(),
    ]
}

#[test]
fn reflexivity_all_scalars() {
    fixture(|f| {
        for atom in scalar_zoo(f) {
            assert_atomic_subtype(f, atom, atom);
        }
    });
}

#[test]
fn int_in_int() {
    fixture(|f| {
        let int = f.t_int();
        assert_atomic_subtype(f, int, int);
    });
}

#[test]
fn lit_int_in_int() {
    fixture(|f| {
        let int = f.t_int();
        for value in [-1000i64, -1, 0, 1, 100, 1_000_000] {
            let literal = f.t_lit_int(value);
            assert_atomic_subtype(f, literal, int);
        }
    });
}

#[test]
fn int_not_in_lit_int() {
    fixture(|f| {
        let int = f.t_int();
        for value in [-1, 0, 1, 100] {
            let literal = f.t_lit_int(value);
            assert_atomic_not_subtype(f, int, literal);
        }
    });
}

#[test]
fn distinct_lit_ints_disjoint() {
    fixture(|f| {
        for (left_value, right_value) in [(0i64, 1), (-1, 1), (10, 100), (1, 2)] {
            let left = f.t_lit_int(left_value);
            let right = f.t_lit_int(right_value);
            assert_atomic_not_subtype(f, left, right);
            assert_atomic_not_subtype(f, right, left);
        }
    });
}

#[test]
fn equal_lit_ints_subtype() {
    fixture(|f| {
        for value in [-100i64, -1, 0, 1, 100] {
            let literal = f.t_lit_int(value);
            assert_atomic_subtype(f, literal, literal);
        }
    });
}

#[test]
fn float_in_float() {
    fixture(|f| {
        let float = f.t_float();
        assert_atomic_subtype(f, float, float);
    });
}

#[test]
fn lit_float_in_float() {
    fixture(|f| {
        let float = f.t_float();
        for value in [-3.25f64, 0.0, 1.5, 100.0] {
            let literal = f.t_lit_float(value);
            assert_atomic_subtype(f, literal, float);
        }
    });
}

#[test]
fn float_not_in_lit_float() {
    fixture(|f| {
        let float = f.t_float();
        for value in [0.0f64, 1.5, -3.25] {
            let literal = f.t_lit_float(value);
            assert_atomic_not_subtype(f, float, literal);
        }
    });
}

#[test]
fn distinct_lit_floats_disjoint() {
    fixture(|f| {
        for (left_value, right_value) in [(0.0f64, 1.0), (1.5, 2.5), (-1.0, 1.0)] {
            let left = f.t_lit_float(left_value);
            let right = f.t_lit_float(right_value);
            assert_atomic_not_subtype(f, left, right);
        }
    });
}

#[test]
fn string_in_string() {
    fixture(|f| {
        let string = f.t_string();
        assert_atomic_subtype(f, string, string);
    });
}

#[test]
fn lit_string_in_string() {
    fixture(|f| {
        let string = f.t_string();
        for text in ["", "hi", "0", "123", "Hello World"] {
            let literal = f.t_lit_string(text);
            assert_atomic_subtype(f, literal, string);
        }
    });
}

#[test]
fn string_not_in_lit_string() {
    fixture(|f| {
        let string = f.t_string();
        for text in ["", "hi", "abc"] {
            let literal = f.t_lit_string(text);
            assert_atomic_not_subtype(f, string, literal);
        }
    });
}

#[test]
fn distinct_lit_strings_disjoint() {
    fixture(|f| {
        for (left_text, right_text) in [("a", "b"), ("hi", "hello"), ("", "x")] {
            let left = f.t_lit_string(left_text);
            let right = f.t_lit_string(right_text);
            assert_atomic_not_subtype(f, left, right);
        }
    });
}

#[test]
fn true_in_bool() {
    fixture(|f| {
        let true_atom = f.t_true();
        let bool_atom = f.t_bool();
        assert_atomic_subtype(f, true_atom, bool_atom);
    });
}

#[test]
fn false_in_bool() {
    fixture(|f| {
        let false_atom = f.t_false();
        let bool_atom = f.t_bool();
        assert_atomic_subtype(f, false_atom, bool_atom);
    });
}

#[test]
fn bool_not_in_true() {
    fixture(|f| {
        let bool_atom = f.t_bool();
        let true_atom = f.t_true();
        assert_atomic_not_subtype(f, bool_atom, true_atom);
    });
}

#[test]
fn bool_not_in_false() {
    fixture(|f| {
        let bool_atom = f.t_bool();
        let false_atom = f.t_false();
        assert_atomic_not_subtype(f, bool_atom, false_atom);
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
fn int_in_numeric() {
    fixture(|f| {
        let int = f.t_int();
        let numeric = f.t_numeric();
        assert_atomic_subtype(f, int, numeric);
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
fn numeric_string_in_numeric() {
    fixture(|f| {
        let numeric_string = f.t_numeric_string();
        let numeric = f.t_numeric();
        assert_atomic_subtype(f, numeric_string, numeric);
    });
}

#[test]
fn string_not_in_numeric() {
    fixture(|f| {
        let string = f.t_string();
        let numeric = f.t_numeric();
        assert_atomic_not_subtype(f, string, numeric);
    });
}

#[test]
fn numeric_lit_string_in_numeric() {
    fixture(|f| {
        let numeric = f.t_numeric();
        for text in ["0", "1", "-1", "123", "1.5", "1e10"] {
            let literal = f.t_lit_string(text);
            assert_atomic_subtype(f, literal, numeric);
        }
    });
}

#[test]
fn non_numeric_lit_string_not_in_numeric() {
    fixture(|f| {
        let numeric = f.t_numeric();
        for text in ["abc", "hi", "12abc", "abc123"] {
            let literal = f.t_lit_string(text);
            assert_atomic_not_subtype(f, literal, numeric);
        }
    });
}

#[test]
fn lit_int_in_numeric() {
    fixture(|f| {
        let numeric = f.t_numeric();
        for value in [-1000i64, -1, 0, 1, 100] {
            let literal = f.t_lit_int(value);
            assert_atomic_subtype(f, literal, numeric);
        }
    });
}

#[test]
fn lit_float_in_numeric() {
    fixture(|f| {
        let numeric = f.t_numeric();
        for value in [-1.5f64, 0.0, 1.5] {
            let literal = f.t_lit_float(value);
            assert_atomic_subtype(f, literal, numeric);
        }
    });
}

#[test]
fn bool_not_in_numeric() {
    fixture(|f| {
        let bool_atom = f.t_bool();
        let numeric = f.t_numeric();
        assert_atomic_not_subtype(f, bool_atom, numeric);
    });
}

#[test]
fn int_in_array_key() {
    fixture(|f| {
        let int = f.t_int();
        let array_key = f.t_array_key();
        assert_atomic_subtype(f, int, array_key);
    });
}

#[test]
fn string_in_array_key() {
    fixture(|f| {
        let string = f.t_string();
        let array_key = f.t_array_key();
        assert_atomic_subtype(f, string, array_key);
    });
}

#[test]
fn lit_int_in_array_key() {
    fixture(|f| {
        let array_key = f.t_array_key();
        for value in [-100i64, 0, 1, 100] {
            let literal = f.t_lit_int(value);
            assert_atomic_subtype(f, literal, array_key);
        }
    });
}

#[test]
fn lit_string_in_array_key() {
    fixture(|f| {
        let array_key = f.t_array_key();
        for text in ["", "x", "abc"] {
            let literal = f.t_lit_string(text);
            assert_atomic_subtype(f, literal, array_key);
        }
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
fn bool_not_in_array_key() {
    fixture(|f| {
        let bool_atom = f.t_bool();
        let array_key = f.t_array_key();
        assert_atomic_not_subtype(f, bool_atom, array_key);
    });
}

#[test]
fn class_like_string_in_array_key() {
    fixture(|f| {
        let array_key = f.t_array_key();
        let class_string = f.t_class_string();
        let interface_string = f.t_interface_string();
        let enum_string = f.t_enum_string();
        let trait_string = f.t_trait_string();
        assert_atomic_subtype(f, class_string, array_key);
        assert_atomic_subtype(f, interface_string, array_key);
        assert_atomic_subtype(f, enum_string, array_key);
        assert_atomic_subtype(f, trait_string, array_key);
    });
}

#[test]
fn class_string_in_string() {
    fixture(|f| {
        let string = f.t_string();
        let class_string = f.t_class_string();
        let interface_string = f.t_interface_string();
        let enum_string = f.t_enum_string();
        assert_atomic_subtype(f, class_string, string);
        assert_atomic_subtype(f, interface_string, string);
        assert_atomic_subtype(f, enum_string, string);
    });
}

#[test]
fn lit_class_string_in_class_string() {
    fixture(|f| {
        let literal = f.t_lit_class_string("Foo");
        let class_string = f.t_class_string();
        assert_atomic_subtype(f, literal, class_string);
    });
}

#[test]
fn lit_class_string_in_string() {
    fixture(|f| {
        let literal = f.t_lit_class_string("Foo");
        let string = f.t_string();
        assert_atomic_subtype(f, literal, string);
    });
}

#[test]
fn int_in_scalar() {
    fixture(|f| {
        let int = f.t_int();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, int, scalar);
    });
}

#[test]
fn string_in_scalar() {
    fixture(|f| {
        let string = f.t_string();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, string, scalar);
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
fn bool_in_scalar() {
    fixture(|f| {
        let bool_atom = f.t_bool();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, bool_atom, scalar);
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
fn numeric_in_scalar() {
    fixture(|f| {
        let numeric = f.t_numeric();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, numeric, scalar);
    });
}

#[test]
fn array_key_in_scalar() {
    fixture(|f| {
        let array_key = f.t_array_key();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, array_key, scalar);
    });
}

#[test]
fn class_string_in_scalar() {
    fixture(|f| {
        let class_string = f.t_class_string();
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, class_string, scalar);
    });
}

#[test]
fn lit_int_in_scalar() {
    fixture(|f| {
        let scalar = f.t_scalar();
        for value in [-100i64, 0, 100] {
            let literal = f.t_lit_int(value);
            assert_atomic_subtype(f, literal, scalar);
        }
    });
}

#[test]
fn lit_string_in_scalar() {
    fixture(|f| {
        let scalar = f.t_scalar();
        for text in ["", "hi", "0"] {
            let literal = f.t_lit_string(text);
            assert_atomic_subtype(f, literal, scalar);
        }
    });
}

#[test]
fn lit_float_in_scalar() {
    fixture(|f| {
        let literal = f.t_lit_float(1.5);
        let scalar = f.t_scalar();
        assert_atomic_subtype(f, literal, scalar);
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
fn object_not_in_scalar() {
    fixture(|f| {
        let object = f.t_object_any();
        let scalar = f.t_scalar();
        assert_atomic_not_subtype(f, object, scalar);
        let foo = f.t_named("Foo");
        assert_atomic_not_subtype(f, foo, scalar);
    });
}

#[test]
fn array_not_in_scalar() {
    fixture(|f| {
        let array = f.t_empty_array();
        let scalar = f.t_scalar();
        assert_atomic_not_subtype(f, array, scalar);
    });
}

#[test]
fn resource_not_in_scalar() {
    fixture(|f| {
        let resource = f.t_resource();
        let scalar = f.t_scalar();
        assert_atomic_not_subtype(f, resource, scalar);
    });
}

#[test]
fn scalar_not_in_int() {
    fixture(|f| {
        let scalar = f.t_scalar();
        let int = f.t_int();
        assert_atomic_not_subtype(f, scalar, int);
    });
}

#[test]
fn scalar_not_in_string() {
    fixture(|f| {
        let scalar = f.t_scalar();
        let string = f.t_string();
        assert_atomic_not_subtype(f, scalar, string);
    });
}

#[test]
fn scalar_not_in_numeric() {
    fixture(|f| {
        let scalar = f.t_scalar();
        let numeric = f.t_numeric();
        assert_atomic_not_subtype(f, scalar, numeric);
    });
}

#[test]
fn scalar_not_in_array_key() {
    fixture(|f| {
        let scalar = f.t_scalar();
        let array_key = f.t_array_key();
        assert_atomic_not_subtype(f, scalar, array_key);
    });
}

#[test]
fn array_key_not_in_int() {
    fixture(|f| {
        let array_key = f.t_array_key();
        let int = f.t_int();
        assert_atomic_not_subtype(f, array_key, int);
    });
}

#[test]
fn array_key_not_in_string() {
    fixture(|f| {
        let array_key = f.t_array_key();
        let string = f.t_string();
        assert_atomic_not_subtype(f, array_key, string);
    });
}

#[test]
fn numeric_not_in_int() {
    fixture(|f| {
        let numeric = f.t_numeric();
        let int = f.t_int();
        assert_atomic_not_subtype(f, numeric, int);
    });
}

#[test]
fn numeric_not_in_float() {
    fixture(|f| {
        let numeric = f.t_numeric();
        let float = f.t_float();
        assert_atomic_not_subtype(f, numeric, float);
    });
}

#[test]
fn numeric_not_in_string() {
    fixture(|f| {
        let numeric = f.t_numeric();
        let string = f.t_string();
        assert_atomic_not_subtype(f, numeric, string);
    });
}

#[test]
fn cross_family_disjoint_int_string() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        assert_atomic_not_subtype(f, int, string);
        assert_atomic_not_subtype(f, string, int);
    });
}

#[test]
fn int_and_float_are_disjoint() {
    fixture(|f| {
        let int = f.t_int();
        let float = f.t_float();
        let lit_int = f.t_lit_int(5);
        let lit_float = f.t_lit_float(1.5);
        assert_atomic_not_subtype(f, int, float);
        assert_atomic_not_subtype(f, lit_int, float);
        assert_atomic_not_subtype(f, float, int);
        assert_atomic_not_subtype(f, lit_float, int);
    });
}

#[test]
fn cross_family_disjoint_int_bool() {
    fixture(|f| {
        let int = f.t_int();
        let bool_atom = f.t_bool();
        assert_atomic_not_subtype(f, int, bool_atom);
        assert_atomic_not_subtype(f, bool_atom, int);
    });
}

#[test]
fn cross_family_disjoint_string_bool() {
    fixture(|f| {
        let string = f.t_string();
        let bool_atom = f.t_bool();
        assert_atomic_not_subtype(f, string, bool_atom);
        assert_atomic_not_subtype(f, bool_atom, string);
    });
}

#[test]
fn string_float_disjoint() {
    fixture(|f| {
        let string = f.t_string();
        let float = f.t_float();
        assert_atomic_not_subtype(f, string, float);
        assert_atomic_not_subtype(f, float, string);
    });
}

#[test]
fn cross_family_disjoint_float_bool() {
    fixture(|f| {
        let float = f.t_float();
        let bool_atom = f.t_bool();
        assert_atomic_not_subtype(f, float, bool_atom);
        assert_atomic_not_subtype(f, bool_atom, float);
    });
}

#[test]
fn lit_int_lit_float_disjoint() {
    fixture(|f| {
        let lit_int = f.t_lit_int(1);
        let lit_float = f.t_lit_float(1.0);
        assert_atomic_not_subtype(f, lit_int, lit_float);
        assert_atomic_not_subtype(f, lit_float, lit_int);
    });
}
