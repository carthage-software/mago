mod combiner_common;

use combiner_common::*;

use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::union::TUnion;

#[test]
fn int_absorbs_every_literal_in_minus_500_to_500() {
    for v in -500..=500i64 {
        assert_combines_to(vec![t_int(), t_lit_int(v)], vec![t_int()]);
        assert_combines_to(vec![t_lit_int(v), t_int()], vec![t_int()]);
    }
}

#[test]
fn unspec_int_absorbs_lit_unspec_lit() {
    assert_combines_to(vec![t_int(), t_int_unspec_lit()], vec![t_int()]);
    assert_combines_to(vec![t_int_unspec_lit(), t_int()], vec![t_int()]);
}

#[test]
fn lit_int_self_dedup_for_many_values() {
    for v in -50..=50i64 {
        for n in 2..=5 {
            let result = combine_default(vec![t_lit_int(v); n]);
            assert_eq!(result.len(), 1);
        }
    }
}

#[test]
fn distinct_lit_pairs_kept_apart() {
    for a in -10..=10i64 {
        for b in -10..=10i64 {
            if a == b {
                continue;
            }
            let result = combine_default(vec![t_lit_int(a), t_lit_int(b)]);
            assert_eq!(result.len(), 2, "{a} ∨ {b}");
        }
    }
}

#[test]
fn ranges_with_overlaps_collapse() {
    let cases = [
        ((0i64, 10), (5, 15)),
        ((0, 10), (10, 20)),
        ((0, 10), (11, 20)),
        ((0, 10), (12, 20)),
        ((-5, 5), (-3, 3)),
        ((0, 100), (50, 60)),
    ];
    for ((a_lo, a_hi), (b_lo, b_hi)) in cases {
        let result = combine_default(vec![t_int_range(a_lo, a_hi), t_int_range(b_lo, b_hi)]);
        let merged = (a_lo.min(b_lo), a_hi.max(b_hi));
        let adj_or_overlap = a_hi.saturating_add(1) >= b_lo && b_hi.saturating_add(1) >= a_lo;
        if adj_or_overlap {
            assert_eq!(result.len(), 1);
            assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_range(merged.0, merged.1)));
        } else {
            assert_eq!(result.len(), 2);
        }
    }
}

#[test]
fn from_n_with_lit_n_minus_1_extends_for_many_n() {
    for n in -50..=50i64 {
        let result = combine_default(vec![t_int_from(n), t_lit_int(n - 1)]);
        assert_eq!(result.len(), 1);
        assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_from(n - 1)));
    }
}

#[test]
fn to_n_with_lit_n_plus_1_extends_for_many_n() {
    for n in -50..=50i64 {
        let result = combine_default(vec![t_int_to(n), t_lit_int(n + 1)]);
        assert_eq!(result.len(), 1);
        assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_to(n + 1)));
    }
}

#[test]
fn string_absorbs_many_literals() {
    for i in 0..200 {
        let s = format!("test_{i}");
        assert_combines_to(vec![t_string(), t_lit_string(&s)], vec![t_string()]);
        assert_combines_to(vec![t_lit_string(&s), t_string()], vec![t_string()]);
    }
}

#[test]
fn lit_string_self_dedup_many_values() {
    for i in 0..50 {
        let s = format!("v_{i}");
        for n in 2..=5 {
            let result = combine_default(vec![t_lit_string(&s); n]);
            assert_eq!(result.len(), 1);
        }
    }
}

#[test]
fn distinct_lit_string_pairs_kept_apart() {
    for i in 0..30 {
        for j in (i + 1)..30 {
            let a = format!("a_{i}");
            let b = format!("b_{j}");
            let result = combine_default(vec![t_lit_string(&a), t_lit_string(&b)]);
            assert_eq!(result.len(), 2);
        }
    }
}

#[test]
fn float_absorbs_many_literals() {
    for i in 0..200 {
        let v = f64::from(i).mul_add(0.5, -50.0);
        assert_combines_to(vec![t_float(), t_lit_float(v)], vec![t_float()]);
        assert_combines_to(vec![t_lit_float(v), t_float()], vec![t_float()]);
    }
}

#[test]
fn lit_float_self_dedup_many_values() {
    for i in 0..50 {
        let v = f64::from(i) * 0.25;
        for n in 2..=5 {
            assert_combines_to(vec![t_lit_float(v); n], vec![t_lit_float(v)]);
        }
    }
}

#[test]
fn all_bool_triples_collapse() {
    let bools = [t_true(), t_false(), t_bool()];
    for a in &bools {
        for b in &bools {
            for c in &bools {
                let result = combine_default(vec![a.clone(), b.clone(), c.clone()]);
                assert_eq!(result.len(), 1, "{a:?} {b:?} {c:?}");
                let id = atomic_id_string(&result[0]);
                assert!(["true", "false", "bool"].contains(&id.as_str()), "got {id}");
            }
        }
    }
}

#[test]
fn many_distinct_named_objects_kept_apart() {
    for n in [3usize, 5, 10, 20, 50] {
        let inputs: Vec<TAtomic> = (0..n).map(|i| t_named(&format!("Class{i}"))).collect();
        let result = combine_default(inputs);
        assert_eq!(result.len(), n);
    }
}

#[test]
fn same_named_with_many_copies_collapses() {
    for n in 1..=20 {
        assert_combines_to(vec![t_named("Foo"); n], vec![t_named("Foo")]);
    }
}

#[test]
fn object_any_absorbs_many_named() {
    for n in 1..=20 {
        let mut inputs = vec![t_object_any()];
        for i in 0..n {
            inputs.push(t_named(&format!("C{i}")));
        }
        assert_combines_to(inputs, vec![t_object_any()]);
    }
}

#[test]
fn generic_with_n_distinct_int_params_collapse_to_one_container() {
    for n in 1..=10 {
        let inputs: Vec<TAtomic> =
            (0..n_i64(n)).map(|i| t_generic_named("Container", vec![TUnion::from_atomic(t_lit_int(i))])).collect();
        let result = combine_default(inputs);
        assert_eq!(result.len(), 1);
    }
}

#[test]
fn generic_with_int_and_string_param_keeps_one_container() {
    let a = t_generic_named("Container", vec![TUnion::from_atomic(t_int())]);
    let b = t_generic_named("Container", vec![TUnion::from_atomic(t_string())]);
    let result = combine_default(vec![a, b]);
    assert_eq!(result.len(), 1);
}

fn n_i64(n: usize) -> i64 {
    n as i64
}

#[test]
fn many_distinct_enums_kept_apart() {
    for n in [3usize, 5, 10, 20] {
        let inputs: Vec<TAtomic> = (0..n).map(|i| t_enum(&format!("E{i}"))).collect();
        let result = combine_default(inputs);
        assert_eq!(result.len(), n);
    }
}

#[test]
fn many_distinct_enum_cases_kept_apart() {
    for n in [3usize, 5, 10] {
        let inputs: Vec<TAtomic> = (0..n).map(|i| t_enum_case("E", &format!("Case{i}"))).collect();
        let result = combine_default(inputs);
        assert_eq!(result.len(), n);
    }
}

#[test]
fn many_copies_of_simple_atoms_collapse() {
    let atoms = [
        t_int(),
        t_string(),
        t_float(),
        t_bool(),
        t_true(),
        t_false(),
        null(),
        never(),
        t_object_any(),
        t_named("Foo"),
        t_resource(),
        t_open_resource(),
        t_closed_resource(),
        t_empty_array(),
        t_list(u(t_int()), false),
        t_keyed_unsealed(u(t_string()), u(t_int()), false),
    ];
    for atom in &atoms {
        for n in [2usize, 5, 10, 20, 50, 100] {
            let result = combine_default(vec![atom.clone(); n]);
            assert_eq!(result.len(), 1, "{n} copies of {atom:?}");
        }
    }
}

#[test]
fn three_way_stable_primitives_consistent() {
    let stable = [t_int(), t_string(), t_float(), t_bool(), null(), t_object_any(), t_named("X"), t_resource()];
    for a in stable.iter() {
        for b in stable.iter() {
            for c in stable.iter() {
                let r = combine_default(vec![a.clone(), b.clone(), c.clone()]);
                let r_rev = combine_default(vec![c.clone(), b.clone(), a.clone()]);
                let mut x: Vec<String> = r.iter().map(atomic_id_string).collect();
                let mut y: Vec<String> = r_rev.iter().map(atomic_id_string).collect();
                x.sort();
                y.sort();
                assert_eq!(x, y, "order-dependence in [a, b, c] vs [c, b, a]: {a:?} {b:?} {c:?}");
            }
        }
    }
}

#[test]
fn mixed_dominates_every_simple_atom() {
    let atoms = [
        t_int(),
        t_string(),
        t_float(),
        t_bool(),
        t_true(),
        t_false(),
        null(),
        void(),
        never(),
        t_object_any(),
        t_named("Foo"),
        t_enum("E"),
        t_resource(),
        t_empty_array(),
        t_list(u(t_int()), false),
        t_keyed_unsealed(u(t_string()), u(t_int()), false),
        t_array_key(),
        t_numeric(),
        t_scalar(),
        t_class_string(),
    ];
    for atom in &atoms {
        let r = combine_default(vec![atom.clone(), mixed()]);
        let r_rev = combine_default(vec![mixed(), atom.clone()]);
        assert_eq!(r.len(), 1);
        assert_eq!(r_rev.len(), 1);
        assert!(matches!(r[0], TAtomic::Mixed(_)));
        assert!(matches!(r_rev[0], TAtomic::Mixed(_)));
    }
}

#[test]
fn never_absorbed_by_every_non_void_atom() {
    let atoms = [
        t_int(),
        t_string(),
        t_float(),
        t_bool(),
        t_true(),
        t_false(),
        null(),
        t_object_any(),
        t_named("Foo"),
        t_enum("E"),
        t_resource(),
        t_open_resource(),
        t_closed_resource(),
        t_empty_array(),
        t_list(u(t_int()), false),
        t_keyed_unsealed(u(t_string()), u(t_int()), false),
        t_array_key(),
        t_numeric(),
        t_scalar(),
        t_class_string(),
        t_lit_int(0),
        t_lit_string("x"),
        t_lit_float(1.0),
    ];
    for atom in &atoms {
        let r = combine_default(vec![atom.clone(), never()]);
        let r_rev = combine_default(vec![never(), atom.clone()]);
        assert!(r.iter().all(|a| !matches!(a, TAtomic::Never)), "never leaked: {atom:?}");
        assert!(r_rev.iter().all(|a| !matches!(a, TAtomic::Never)), "never leaked rev: {atom:?}");
    }
}

#[test]
fn many_named_with_lits() {
    for n in 1..=10 {
        let mut inputs = vec![];
        for i in 0..n {
            inputs.push(t_named(&format!("C{i}")));
            inputs.push(t_lit_int(i as i64));
        }
        let result = combine_default(inputs);
        assert_eq!(result.len(), 2 * n);
    }
}

#[test]
fn alternating_int_string_collapses_to_two() {
    for n in 1..=20 {
        let mut inputs = Vec::new();
        for _ in 0..n {
            inputs.push(t_int());
            inputs.push(t_string());
        }
        let r = combine_default(inputs);
        assert_eq!(r.len(), 2);
    }
}

#[test]
fn alternating_named_collapses() {
    for n in 1..=15 {
        let mut inputs = Vec::new();
        for _ in 0..n {
            inputs.push(t_named("A"));
            inputs.push(t_named("B"));
        }
        let r = combine_default(inputs);
        assert_eq!(r.len(), 2);
    }
}

#[test]
fn n_copies_plus_distinct_int_kept() {
    for n in [1usize, 5, 10, 50, 100] {
        let mut inputs: Vec<TAtomic> = std::iter::repeat_with(|| t_lit_int(0)).take(n).collect();
        inputs.push(t_lit_int(1));
        let r = combine_default(inputs);
        assert_eq!(r.len(), 2, "n={n}");
    }
}
