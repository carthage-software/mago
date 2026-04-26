mod combiner_common;

use combiner_common::*;

use std::collections::BTreeMap;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::union::TUnion;

/// Build a comprehensive zoo of atoms covering every TAtomic family.
fn full_atom_zoo() -> Vec<TAtomic> {
    let mut atoms = Vec::new();

    atoms.push(t_bool());
    atoms.push(t_true());
    atoms.push(t_false());

    atoms.push(t_int());
    atoms.push(t_int_unspec_lit());
    atoms.push(t_positive_int());
    atoms.push(t_negative_int());
    atoms.push(t_non_negative_int());
    atoms.push(t_non_positive_int());
    for v in [-1000_i64, -100, -10, -1, 0, 1, 10, 100, 1000] {
        atoms.push(t_lit_int(v));
    }
    for from in [-100_i64, -1, 0, 1, 100] {
        atoms.push(t_int_from(from));
    }
    for to in [-100_i64, -1, 0, 1, 100] {
        atoms.push(t_int_to(to));
    }
    for (lo, hi) in [(-50_i64, 50), (0, 100), (-100, 0), (-10, 10)] {
        atoms.push(t_int_range(lo, hi));
    }

    atoms.push(t_float());
    atoms.push(t_unspec_lit_float());
    for v in [-100.0_f64, -1.0, 0.0, 1.0, 1.5, 100.0] {
        atoms.push(t_lit_float(v));
    }

    atoms.push(t_string());
    atoms.push(t_non_empty_string());
    atoms.push(t_numeric_string());
    atoms.push(t_lower_string());
    atoms.push(t_upper_string());
    atoms.push(t_truthy_string());
    atoms.push(t_callable_string());
    atoms.push(t_unspec_lit_string(false));
    atoms.push(t_unspec_lit_string(true));
    for s in ["", "hi", "0", "Hello", "HELLO", "hello world", "123"] {
        atoms.push(t_lit_string(s));
    }

    atoms.push(t_class_string());
    atoms.push(t_interface_string());
    atoms.push(t_enum_string());
    atoms.push(t_trait_string());
    for n in ["Foo", "Bar", "App\\Service"] {
        atoms.push(t_lit_class_string(n));
    }

    atoms.push(t_array_key());
    atoms.push(t_numeric());
    atoms.push(t_scalar());

    atoms.push(null());
    atoms.push(void());
    atoms.push(never());

    atoms.push(mixed());

    atoms.push(t_resource());
    atoms.push(t_open_resource());
    atoms.push(t_closed_resource());

    atoms.push(t_object_any());
    for n in ["Foo", "App\\Bar", "X\\Y\\Z"] {
        atoms.push(t_named(n));
    }
    atoms.push(t_generic_named("Container", vec![TUnion::from_atomic(t_int())]));
    atoms.push(t_generic_named("Container", vec![TUnion::from_atomic(t_string())]));
    atoms.push(t_generic_named("Pair", vec![TUnion::from_atomic(t_int()), TUnion::from_atomic(t_string())]));
    for n in ["E", "MyEnum", "Status"] {
        atoms.push(t_enum(n));
    }
    for (n, c) in [("E", "A"), ("Status", "Active"), ("Color", "Red")] {
        atoms.push(t_enum_case(n, c));
    }

    atoms.push(t_empty_array());
    atoms.push(t_list(u(t_int()), false));
    atoms.push(t_list(u(t_int()), true));
    atoms.push(t_list(u(t_string()), false));
    atoms.push(t_list(u(t_mixed()), false));
    atoms.push(t_keyed_unsealed(u(t_string()), u(t_int()), false));
    atoms.push(t_keyed_unsealed(u(t_int()), u(t_string()), false));
    atoms.push(t_keyed_unsealed(u(t_array_key()), u(t_mixed()), false));

    atoms.push(t_sealed_list(BTreeMap::from([(0_usize, (false, ui(1)))])));
    atoms.push(t_sealed_list(BTreeMap::from([(0_usize, (false, u(t_int()))), (1_usize, (false, u(t_string())))])));

    atoms.push(t_keyed_sealed(BTreeMap::from([(ak_str("a"), (false, ui(1)))]), false));
    atoms.push(t_keyed_sealed(
        BTreeMap::from([(ak_str("a"), (false, u(t_int()))), (ak_str("b"), (false, u(t_string())))]),
        false,
    ));

    atoms
}

fn t_mixed() -> TAtomic {
    mixed()
}

#[test]
fn singleton_passthrough_for_full_zoo() {
    for atom in full_atom_zoo() {
        let r = combine_default(vec![atom.clone()]);
        assert_eq!(r.len(), 1, "singleton broke for {atom:?}");
        assert_eq!(atomic_id_string(&r[0]), atomic_id_string(&atom), "singleton id changed for {atom:?}");
    }
}

#[test]
fn self_idempotency_basic() {
    let zoo = full_atom_zoo();
    for atom in zoo {
        if is_known_non_idempotent_under_combine(&atom) {
            continue;
        }
        for n in [2_usize, 3, 5, 10] {
            let r = combine_default(vec![atom.clone(); n]);
            assert_eq!(r.len(), 1, "self-idempotency broke for {atom:?} (n={n})");
            assert_eq!(atomic_id_string(&r[0]), atomic_id_string(&atom));
        }
    }
}

/// Sealed lists/keyed-arrays without codebase metadata don't always collapse via
/// `is_array_contained_by_array`. Skip them in the broad self-idempotency sweep.
fn is_known_non_idempotent_under_combine(atom: &TAtomic) -> bool {
    matches!(
        atom,
        TAtomic::Array(mago_codex::ttype::atomic::array::TArray::List(l))
            if l.known_elements.is_some()
    )
}

#[test]
fn double_input_matches_single_for_zoo() {
    for atom in full_atom_zoo() {
        if is_known_non_idempotent_under_combine(&atom) {
            continue;
        }
        let single = combine_default(vec![atom.clone()]);
        let double = combine_default(vec![atom.clone(), atom.clone()]);
        let mut a: Vec<String> = single.iter().map(atomic_id_string).collect();
        let mut b: Vec<String> = double.iter().map(atomic_id_string).collect();
        a.sort();
        b.sort();
        assert_eq!(a, b, "double != single for {atom:?}");
    }
}

#[test]
fn never_is_absorbed_by_every_non_void_zoo_atom() {
    for atom in full_atom_zoo() {
        if matches!(atom, TAtomic::Never | TAtomic::Void) {
            continue;
        }
        let r1 = combine_default(vec![atom.clone(), never()]);
        let r2 = combine_default(vec![never(), atom.clone()]);
        assert!(r1.iter().all(|a| !matches!(a, TAtomic::Never)), "never leaked through with {atom:?}");
        assert!(r2.iter().all(|a| !matches!(a, TAtomic::Never)), "never leaked through (rev) with {atom:?}");
    }
}

#[test]
fn mixed_dominates_every_zoo_atom() {
    for atom in full_atom_zoo() {
        let r1 = combine_default(vec![atom.clone(), mixed()]);
        let r2 = combine_default(vec![mixed(), atom.clone()]);
        assert_eq!(r1.len(), 1, "mixed didn't dominate {atom:?} (forward)");
        assert_eq!(r2.len(), 1, "mixed didn't dominate {atom:?} (reverse)");
        assert!(matches!(r1[0], TAtomic::Mixed(_)), "{atom:?} ∨ mixed didn't yield Mixed");
        assert!(matches!(r2[0], TAtomic::Mixed(_)));
    }
}

#[test]
fn lit_int_absorbed_by_int_for_many_values() {
    let values: Vec<i64> = (-50..50).collect();
    for v in values {
        assert_combines_to(vec![t_int(), t_lit_int(v)], vec![t_int()]);
        assert_combines_to(vec![t_lit_int(v), t_int()], vec![t_int()]);
    }
}

#[test]
fn lit_string_absorbed_by_string_for_many_values() {
    let strings: Vec<String> = (0..50).map(|i| format!("test_{i}")).collect();
    for s in &strings {
        assert_combines_to(vec![t_string(), t_lit_string(s)], vec![t_string()]);
        assert_combines_to(vec![t_lit_string(s), t_string()], vec![t_string()]);
    }
}

#[test]
fn lit_float_absorbed_by_float_for_many_values() {
    for i in 0..30 {
        let v = f64::from(i) * 0.5;
        assert_combines_to(vec![t_float(), t_lit_float(v)], vec![t_float()]);
        assert_combines_to(vec![t_lit_float(v), t_float()], vec![t_float()]);
    }
}

#[test]
fn order_independence_for_non_asymmetric_pairs() {
    let stable = vec![
        t_int(),
        t_string(),
        t_float(),
        t_bool(),
        t_named("Foo"),
        t_named("Bar"),
        t_object_any(),
        t_resource(),
        t_open_resource(),
        t_closed_resource(),
        t_list(u(t_int()), false),
        null(),
    ];

    for (i, a) in stable.iter().enumerate() {
        for b in &stable[i..] {
            let ab = combine_default(vec![a.clone(), b.clone()]);
            let ba = combine_default(vec![b.clone(), a.clone()]);
            let mut x: Vec<String> = ab.iter().map(atomic_id_string).collect();
            let mut y: Vec<String> = ba.iter().map(atomic_id_string).collect();
            x.sort();
            y.sort();
            assert_eq!(x, y, "order-dependence between {a:?} and {b:?}");
        }
    }
}
