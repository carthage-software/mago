mod common;

use common::*;

use mago_oracle::ty::Atom;
use mago_oracle::ty::AtomKind;

fn full_atom_zoo<'arena>(f: &mut Fixture<'_, 'arena>) -> Vec<Atom<'arena>> {
    let mut atoms = vec![
        f.t_bool(),
        f.t_true(),
        f.t_false(),
        f.t_int(),
        f.t_int_unspec_lit(),
        f.t_positive_int(),
        f.t_negative_int(),
        f.t_non_negative_int(),
        f.t_non_positive_int(),
    ];
    for value in [-1000i64, -100, -10, -1, 0, 1, 10, 100, 1000] {
        atoms.push(f.t_lit_int(value));
    }
    for from in [-100i64, -1, 0, 1, 100] {
        atoms.push(f.t_int_from(from));
    }
    for to in [-100i64, -1, 0, 1, 100] {
        atoms.push(f.t_int_to(to));
    }
    for (low, high) in [(-50i64, 50), (0, 100), (-100, 0), (-10, 10)] {
        atoms.push(f.t_int_range(low, high));
    }

    atoms.push(f.t_float());
    atoms.push(f.t_unspec_lit_float());
    for value in [-100.0f64, -1.0, 0.0, 1.0, 1.5, 100.0] {
        atoms.push(f.t_lit_float(value));
    }

    atoms.push(f.t_string());
    atoms.push(f.t_non_empty_string());
    atoms.push(f.t_numeric_string());
    atoms.push(f.t_lower_string());
    atoms.push(f.t_upper_string());
    atoms.push(f.t_truthy_string());
    atoms.push(f.t_callable_string());
    atoms.push(f.t_unspec_lit_string(false));
    atoms.push(f.t_unspec_lit_string(true));
    for value in ["", "hi", "0", "Hello", "HELLO", "hello world", "123"] {
        atoms.push(f.t_lit_string(value));
    }

    atoms.push(f.t_class_string());
    atoms.push(f.t_interface_string());
    atoms.push(f.t_enum_string());
    atoms.push(f.t_trait_string());
    for name in ["Foo", "Bar", "App\\Service"] {
        atoms.push(f.t_lit_class_string(name));
    }

    atoms.push(f.t_array_key());
    atoms.push(f.t_numeric());
    atoms.push(f.t_scalar());

    atoms.push(f.null());
    atoms.push(f.void());
    atoms.push(f.never());

    atoms.push(f.mixed());

    atoms.push(f.t_resource());
    atoms.push(f.t_open_resource());
    atoms.push(f.t_closed_resource());

    atoms.push(f.t_object_any());
    for name in ["Foo", "App\\Bar", "X\\Y\\Z"] {
        atoms.push(f.t_named(name));
    }
    for name in ["E", "MyEnum", "Status"] {
        atoms.push(f.t_enum(name));
    }
    for (name, case) in [("E", "A"), ("Status", "Active"), ("Color", "Red")] {
        atoms.push(f.t_enum_case(name, case));
    }

    atoms.push(f.t_empty_array());

    atoms
}

fn is_mixed(atom: Atom<'_>) -> bool {
    atom.kind() == AtomKind::Mixed
}

#[test]
fn singleton_passthrough_for_full_zoo() {
    fixture(|f| {
        for atom in full_atom_zoo(f) {
            let result = combine_default(f, vec![atom]);
            assert_eq!(result.len(), 1, "singleton broke for {atom:?}");
            assert_eq!(result[0], atom, "singleton id changed for {atom:?}");
        }
    });
}

#[test]
fn self_idempotency_basic() {
    fixture(|f| {
        for atom in full_atom_zoo(f) {
            for n in [2usize, 3, 5, 10] {
                let result = combine_default(f, vec![atom; n]);
                assert_eq!(result.len(), 1, "self-idempotency broke for {atom:?} (n={n})");
                assert_eq!(result[0], atom);
            }
        }
    });
}

#[test]
fn double_input_matches_single_for_zoo() {
    fixture(|f| {
        for atom in full_atom_zoo(f) {
            let single = combine_default(f, vec![atom]);
            let double = combine_default(f, vec![atom, atom]);
            assert_multiset_eq(&single, &double);
        }
    });
}

#[test]
fn never_is_absorbed_by_every_non_void_zoo_atom() {
    fixture(|f| {
        let never = f.never();
        let void = f.void();
        for atom in full_atom_zoo(f) {
            if atom == never || atom == void {
                continue;
            }
            let result = combine_default(f, vec![atom, never]);
            let result_reversed = combine_default(f, vec![never, atom]);
            assert!(result.iter().all(|candidate| *candidate != never), "never leaked through with {atom:?}");
            assert!(
                result_reversed.iter().all(|candidate| *candidate != never),
                "never leaked through (rev) with {atom:?}"
            );
        }
    });
}

#[test]
fn mixed_dominates_every_zoo_atom() {
    fixture(|f| {
        let mixed = f.mixed();
        for atom in full_atom_zoo(f) {
            let result = combine_default(f, vec![atom, mixed]);
            let result_reversed = combine_default(f, vec![mixed, atom]);
            assert_eq!(result.len(), 1, "mixed didn't dominate {atom:?} (forward)");
            assert_eq!(result_reversed.len(), 1, "mixed didn't dominate {atom:?} (reverse)");
            assert!(is_mixed(result[0]), "{atom:?} | mixed didn't yield Mixed");
            assert!(is_mixed(result_reversed[0]), "{atom:?} | mixed didn't yield Mixed (rev)");
        }
    });
}

#[test]
fn lit_int_absorbed_by_int_for_many_values() {
    fixture(|f| {
        let values: Vec<i64> = (-50..50).collect();
        for value in values {
            assert_combines_to(f, vec![f.t_int(), f.t_lit_int(value)], vec![f.t_int()]);
            assert_combines_to(f, vec![f.t_lit_int(value), f.t_int()], vec![f.t_int()]);
        }
    });
}

#[test]
fn lit_string_absorbed_by_string_for_many_values() {
    fixture(|f| {
        let strings: Vec<String> = (0..50).map(|i| format!("test_{i}")).collect();
        for s in &strings {
            let literal = f.t_lit_string(s);
            assert_combines_to(f, vec![f.t_string(), literal], vec![f.t_string()]);
            assert_combines_to(f, vec![literal, f.t_string()], vec![f.t_string()]);
        }
    });
}

#[test]
fn lit_float_absorbed_by_float_for_many_values() {
    fixture(|f| {
        for i in 0..30 {
            let value = f64::from(i) * 0.5;
            assert_combines_to(f, vec![f.t_float(), f.t_lit_float(value)], vec![f.t_float()]);
            assert_combines_to(f, vec![f.t_lit_float(value), f.t_float()], vec![f.t_float()]);
        }
    });
}

#[test]
fn order_independence_for_non_asymmetric_pairs() {
    fixture(|f| {
        let stable = vec![
            f.t_int(),
            f.t_string(),
            f.t_float(),
            f.t_bool(),
            f.t_named("Foo"),
            f.t_named("Bar"),
            f.t_object_any(),
            f.t_resource(),
            f.t_open_resource(),
            f.t_closed_resource(),
            f.null(),
        ];

        for (i, a) in stable.iter().enumerate() {
            for b in &stable[i..] {
                let forward = combine_default(f, vec![*a, *b]);
                let backward = combine_default(f, vec![*b, *a]);
                assert_multiset_eq(&forward, &backward);
            }
        }
    });
}
