mod comparator_common;

use comparator_common::*;
use mago_codex::ttype::atomic::TAtomic;

#[test]
fn every_lit_int_in_int() {
    for v in -500..=500i64 {
        assert_atomic_subtype(&t_lit_int(v), &t_int());
    }
}

#[test]
fn no_distinct_lit_ints_subtype() {
    for a in -20..=20i64 {
        for b in -20..=20i64 {
            if a == b {
                continue;
            }
            assert_atomic_not_subtype(&t_lit_int(a), &t_lit_int(b));
        }
    }
}

#[test]
fn every_positive_lit_in_positive() {
    for v in 1..=200i64 {
        assert_atomic_subtype(&t_lit_int(v), &t_positive_int());
    }
}

#[test]
fn every_zero_or_positive_in_non_negative() {
    for v in 0..=200i64 {
        assert_atomic_subtype(&t_lit_int(v), &t_non_negative_int());
    }
}

#[test]
fn every_negative_lit_in_negative() {
    for v in -200..=-1i64 {
        assert_atomic_subtype(&t_lit_int(v), &t_negative_int());
    }
}

#[test]
fn every_zero_or_negative_in_non_positive() {
    for v in -200..=0i64 {
        assert_atomic_subtype(&t_lit_int(v), &t_non_positive_int());
    }
}

#[test]
fn lit_in_range_inclusive() {
    for lo in [-50i64, 0, 50] {
        for v in (lo + 1)..(lo + 30) {
            assert_atomic_subtype(&t_lit_int(v), &t_int_range(lo, lo + 29));
        }
    }
}

#[test]
fn lit_in_from() {
    for n in [-10i64, 0, 5, 100] {
        for v in n..(n + 50) {
            assert_atomic_subtype(&t_lit_int(v), &t_int_from(n));
        }
    }
}

#[test]
fn lit_below_from_not_subtype() {
    for n in [0i64, 5, 100] {
        for v in (n - 50)..n {
            assert_atomic_not_subtype(&t_lit_int(v), &t_int_from(n));
        }
    }
}

#[test]
fn lit_in_to() {
    for n in [-50i64, 0, 50] {
        for v in (n - 30)..=n {
            assert_atomic_subtype(&t_lit_int(v), &t_int_to(n));
        }
    }
}

#[test]
fn every_lit_str_in_string() {
    for i in 0..200 {
        let s = format!("test_{i}");
        assert_atomic_subtype(&t_lit_string(&s), &t_string());
    }
}

#[test]
fn every_lit_str_eq_self() {
    for i in 0..100 {
        let s = format!("v_{i}");
        assert_atomic_subtype(&t_lit_string(&s), &t_lit_string(&s));
    }
}

#[test]
fn no_distinct_lit_strs_subtype() {
    let strs: Vec<_> = (0..30).map(|i| format!("a_{i}")).collect();
    for a in &strs {
        for b in &strs {
            if a == b {
                continue;
            }
            assert_atomic_not_subtype(&t_lit_string(a), &t_lit_string(b));
        }
    }
}

#[test]
fn every_lit_float_in_float() {
    for i in 0..200 {
        let v = f64::from(i).mul_add(0.5, -50.0);
        assert_atomic_subtype(&t_lit_float(v), &t_float());
    }
}

#[test]
fn every_atom_in_mixed() {
    let atoms = full_zoo();
    for a in atoms {
        assert_atomic_subtype(&a, &mixed());
    }
}

#[test]
fn never_in_every_atom() {
    for a in full_zoo() {
        assert_atomic_subtype(&never(), &a);
    }
}

#[test]
fn every_atom_eq_self() {
    for a in full_zoo() {
        assert_atomic_subtype(&a, &a);
    }
}

#[test]
fn list_lit_int_in_list_int_for_many_values() {
    for v in -50..=50i64 {
        assert_atomic_subtype(&t_list(u(t_lit_int(v)), false), &t_list(u(t_int()), false));
    }
}

#[test]
fn list_lit_string_in_list_string_for_many_values() {
    for i in 0..50 {
        let s = format!("s_{i}");
        assert_atomic_subtype(&t_list(u(t_lit_string(&s)), false), &t_list(u(t_string()), false));
    }
}

#[test]
fn nullable_int_contains_every_lit() {
    let nullable = u_many(vec![t_int(), null()]);
    for v in -50..=50i64 {
        assert_subtype(&u(t_lit_int(v)), &nullable);
    }
}

#[test]
fn int_or_str_contains_every_lit() {
    let union = u_many(vec![t_int(), t_string()]);
    for v in -20..=20i64 {
        assert_subtype(&u(t_lit_int(v)), &union);
    }
    for s in ["a", "b", "c", "hi", "hello"] {
        assert_subtype(&u(t_lit_string(s)), &union);
    }
}

fn full_zoo() -> Vec<TAtomic> {
    vec![
        t_bool(),
        t_true(),
        t_false(),
        t_int(),
        t_lit_int(0),
        t_lit_int(42),
        t_positive_int(),
        t_negative_int(),
        t_int_range(0, 10),
        t_float(),
        t_lit_float(1.5),
        t_string(),
        t_lit_string("hi"),
        t_non_empty_string(),
        t_numeric_string(),
        t_lower_string(),
        t_upper_string(),
        t_truthy_string(),
        t_class_string(),
        t_interface_string(),
        t_enum_string(),
        t_lit_class_string("Foo"),
        t_array_key(),
        t_numeric(),
        t_scalar(),
        null(),
        void(),
        t_resource(),
        t_open_resource(),
        t_closed_resource(),
        t_object_any(),
        t_named("Foo"),
        t_enum("E"),
        t_enum_case("E", "A"),
        t_empty_array(),
        t_list(u(t_int()), false),
        t_list(u(t_int()), true),
        t_keyed_unsealed(u(t_string()), u(t_int()), false),
        t_iterable(u(t_int()), u(t_int())),
    ]
}
