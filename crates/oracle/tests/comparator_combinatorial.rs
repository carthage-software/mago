mod common;

use common::*;

use mago_oracle::ty::Atom;

fn full_zoo<'arena>(f: &mut Fixture<'_, 'arena>) -> Vec<Atom<'arena>> {
    vec![
        f.t_bool(),
        f.t_true(),
        f.t_false(),
        f.t_int(),
        f.t_lit_int(0),
        f.t_lit_int(42),
        f.t_positive_int(),
        f.t_negative_int(),
        f.t_int_range(0, 10),
        f.t_float(),
        f.t_lit_float(1.5),
        f.t_string(),
        f.t_lit_string("hi"),
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
        f.null(),
        f.void(),
        f.t_resource(),
        f.t_open_resource(),
        f.t_closed_resource(),
        f.t_object_any(),
        f.t_named("Foo"),
        f.t_enum("E"),
        f.t_enum_case("E", "A"),
        f.t_empty_array(),
    ]
}

#[test]
fn every_lit_int_in_int() {
    fixture(|f| {
        let int = f.t_int();
        for value in -500..=500i64 {
            let literal = f.t_lit_int(value);
            assert_atomic_subtype(f, literal, int);
        }
    });
}

#[test]
fn no_distinct_lit_ints_subtype() {
    fixture(|f| {
        for a in -20..=20i64 {
            for b in -20..=20i64 {
                if a == b {
                    continue;
                }
                let a_literal = f.t_lit_int(a);
                let b_literal = f.t_lit_int(b);
                assert_atomic_not_subtype(f, a_literal, b_literal);
            }
        }
    });
}

#[test]
fn every_positive_lit_in_positive() {
    fixture(|f| {
        let positive = f.t_positive_int();
        for value in 1..=200i64 {
            let literal = f.t_lit_int(value);
            assert_atomic_subtype(f, literal, positive);
        }
    });
}

#[test]
fn every_zero_or_positive_in_non_negative() {
    fixture(|f| {
        let non_negative = f.t_non_negative_int();
        for value in 0..=200i64 {
            let literal = f.t_lit_int(value);
            assert_atomic_subtype(f, literal, non_negative);
        }
    });
}

#[test]
fn every_negative_lit_in_negative() {
    fixture(|f| {
        let negative = f.t_negative_int();
        for value in -200..=-1i64 {
            let literal = f.t_lit_int(value);
            assert_atomic_subtype(f, literal, negative);
        }
    });
}

#[test]
fn every_zero_or_negative_in_non_positive() {
    fixture(|f| {
        let non_positive = f.t_non_positive_int();
        for value in -200..=0i64 {
            let literal = f.t_lit_int(value);
            assert_atomic_subtype(f, literal, non_positive);
        }
    });
}

#[test]
fn lit_in_range_inclusive() {
    fixture(|f| {
        for low in [-50i64, 0, 50] {
            for value in (low + 1)..(low + 30) {
                let literal = f.t_lit_int(value);
                let range = f.t_int_range(low, low + 29);
                assert_atomic_subtype(f, literal, range);
            }
        }
    });
}

#[test]
fn lit_in_from() {
    fixture(|f| {
        for from in [-10i64, 0, 5, 100] {
            for value in from..(from + 50) {
                let literal = f.t_lit_int(value);
                let range = f.t_int_from(from);
                assert_atomic_subtype(f, literal, range);
            }
        }
    });
}

#[test]
fn lit_below_from_not_subtype() {
    fixture(|f| {
        for from in [0i64, 5, 100] {
            for value in (from - 50)..from {
                let literal = f.t_lit_int(value);
                let range = f.t_int_from(from);
                assert_atomic_not_subtype(f, literal, range);
            }
        }
    });
}

#[test]
fn lit_in_to() {
    fixture(|f| {
        for to in [-50i64, 0, 50] {
            for value in (to - 30)..=to {
                let literal = f.t_lit_int(value);
                let range = f.t_int_to(to);
                assert_atomic_subtype(f, literal, range);
            }
        }
    });
}

#[test]
fn every_lit_str_in_string() {
    fixture(|f| {
        let string = f.t_string();
        for index in 0..200 {
            let value = format!("test_{index}");
            let literal = f.t_lit_string(&value);
            assert_atomic_subtype(f, literal, string);
        }
    });
}

#[test]
fn every_lit_str_eq_self() {
    fixture(|f| {
        for index in 0..100 {
            let value = format!("v_{index}");
            let literal = f.t_lit_string(&value);
            assert_atomic_subtype(f, literal, literal);
        }
    });
}

#[test]
fn no_distinct_lit_strs_subtype() {
    fixture(|f| {
        let values: Vec<_> = (0..30).map(|index| format!("a_{index}")).collect();
        for a in &values {
            for b in &values {
                if a == b {
                    continue;
                }
                let a_literal = f.t_lit_string(a);
                let b_literal = f.t_lit_string(b);
                assert_atomic_not_subtype(f, a_literal, b_literal);
            }
        }
    });
}

#[test]
fn every_lit_float_in_float() {
    fixture(|f| {
        let float = f.t_float();
        for index in 0..200 {
            let value = f64::from(index).mul_add(0.5, -50.0);
            let literal = f.t_lit_float(value);
            assert_atomic_subtype(f, literal, float);
        }
    });
}

#[test]
fn every_atom_in_mixed() {
    fixture(|f| {
        let mixed = f.mixed();
        for atom in full_zoo(f) {
            assert_atomic_subtype(f, atom, mixed);
        }
    });
}

#[test]
fn never_in_every_atom() {
    fixture(|f| {
        let never = f.never();
        for atom in full_zoo(f) {
            assert_atomic_subtype(f, never, atom);
        }
    });
}

#[test]
fn every_atom_eq_self() {
    fixture(|f| {
        for atom in full_zoo(f) {
            assert_atomic_subtype(f, atom, atom);
        }
    });
}

#[test]
fn nullable_int_contains_every_lit() {
    fixture(|f| {
        let int = f.t_int();
        let null = f.null();
        let nullable = f.u_many(vec![int, null]);
        for value in -50..=50i64 {
            let literal = f.ui(value);
            assert_subtype(f, literal, nullable);
        }
    });
}

#[test]
fn int_or_str_contains_every_lit() {
    fixture(|f| {
        let int = f.t_int();
        let string = f.t_string();
        let union = f.u_many(vec![int, string]);
        for value in -20..=20i64 {
            let literal = f.ui(value);
            assert_subtype(f, literal, union);
        }
        for value in ["a", "b", "c", "hi", "hello"] {
            let literal = f.us(value);
            assert_subtype(f, literal, union);
        }
    });
}
