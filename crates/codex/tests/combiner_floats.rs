#![allow(clippy::approx_constant)]

mod combiner_common;

use combiner_common::*;

use mago_codex::ttype::atomic::TAtomic;

#[test]
fn idempotent_general_float() {
    for n in 1..=10 {
        assert_self_idempotent(t_float(), n);
    }
}

#[test]
fn idempotent_unspec_literal() {
    for n in 1..=10 {
        assert_self_idempotent(t_unspec_lit_float(), n);
    }
}

#[test]
fn idempotent_specific_literal() {
    for v in [-3.14_f64, -1.0, 0.0, 1.0, 1.5, 1e10, 1e-10, 0.5] {
        for n in 1..=8 {
            assert_self_idempotent(t_lit_float(v), n);
        }
    }
}

#[test]
fn float_absorbs_literal_either_order() {
    for v in [-100.0_f64, 0.0, 1.5, 100.0] {
        assert_combines_to(vec![t_float(), t_lit_float(v)], vec![t_float()]);
        assert_combines_to(vec![t_lit_float(v), t_float()], vec![t_float()]);
    }
}

#[test]
fn float_absorbs_unspec_literal_either_order() {
    let result = combine_default(vec![t_float(), t_unspec_lit_float()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert!(ids.contains(&"float".to_string()), "expected float in {ids:?}");
}

#[test]
fn float_absorbs_many_literals() {
    let mut inputs = vec![t_float()];
    for i in 0..30 {
        inputs.push(t_lit_float(f64::from(i)));
    }
    assert_combines_to(inputs, vec![t_float()]);
}

#[test]
fn two_distinct_literals_kept() {
    for (a, b) in [(0.0_f64, 1.0), (-1.0, 1.0), (1.5, 2.5), (-3.14, 3.14)] {
        let result = combine_default(vec![t_lit_float(a), t_lit_float(b)]);
        assert_eq!(result.len(), 2, "{a} ∨ {b}");
    }
}

#[test]
fn n_distinct_literals_kept() {
    for n in [3_usize, 5, 10, 50] {
        let inputs: Vec<TAtomic> = (0..n).map(|i| t_lit_float(i as f64 + 0.5)).collect();
        let result = combine_default(inputs);
        assert_eq!(result.len(), n);
    }
}

#[test]
fn duplicate_literal_floats_collapse() {
    for v in [0.0_f64, 1.5, -3.14, 1e10] {
        for n in 1..=10 {
            assert_combines_to(vec![t_lit_float(v); n], vec![t_lit_float(v)]);
        }
    }
}

#[test]
fn many_distinct_literals_exceed_threshold_generalise() {
    let n = 200_usize;
    let inputs: Vec<TAtomic> = (0..n).map(|i| t_lit_float(i as f64)).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), "float");
}

#[test]
fn under_threshold_keeps_literals() {
    let n = 100_usize;
    let inputs: Vec<TAtomic> = (0..n).map(|i| t_lit_float(i as f64)).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), n);
}

#[test]
fn custom_low_threshold_generalises_quickly() {
    let inputs: Vec<TAtomic> = (0..20_usize).map(|i| t_lit_float(i as f64)).collect();
    let result = combine_with_string_threshold(inputs, 5);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), "float");
}

#[test]
fn negative_zero_collapses_with_zero() {
    let result = combine_default(vec![t_lit_float(-0.0), t_lit_float(0.0)]);
    assert!(result.len() <= 2);
}

#[test]
fn float_int_kept_separate() {
    let result = combine_default(vec![t_float(), t_int()]);
    let mut ids: Vec<String> = result.iter().map(atomic_id_string).collect();
    ids.sort();
    assert_eq!(ids, vec!["float", "int"]);
}

#[test]
fn lit_float_lit_int_kept_separate() {
    let result = combine_default(vec![t_lit_float(1.5), t_lit_int(1)]);
    assert_eq!(result.len(), 2);
}
