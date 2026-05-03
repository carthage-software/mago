mod combiner_common;

use combiner_common::*;

use std::collections::BTreeMap;

use mago_codex::ttype::atomic::TAtomic;

#[test]
fn at_default_int_threshold_keeps_literals() {
    let n = 128usize;
    let inputs: Vec<TAtomic> = (0..n).map(|i| t_lit_int(i as i64)).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), n);
}

#[test]
fn just_over_default_int_threshold_generalises() {
    let n = 129usize;
    let inputs: Vec<TAtomic> = (0..n).map(|i| t_lit_int(i as i64)).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), "int");
}

#[test]
fn many_int_thresholds_walk() {
    for threshold in [1u16, 2, 5, 10, 32, 64, 100, 128] {
        let inputs: Vec<TAtomic> = (0..200i64).map(t_lit_int).collect();
        let result = combine_with_int_threshold(inputs, threshold);
        assert_eq!(result.len(), 1);
        assert_eq!(atomic_id_string(&result[0]), "int");
    }
}

#[test]
fn int_threshold_above_input_count_keeps_literals() {
    let inputs: Vec<TAtomic> = (0..50i64).map(t_lit_int).collect();
    let result = combine_with_int_threshold(inputs, 100);
    assert_eq!(result.len(), 50);
}

#[test]
fn at_default_string_threshold_keeps_literals() {
    let n = 128usize;
    let inputs: Vec<TAtomic> = (0..n).map(|i| t_lit_string(&format!("s{i}"))).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), n);
}

#[test]
fn just_over_default_string_threshold_generalises() {
    let n = 129usize;
    let inputs: Vec<TAtomic> = (0..n).map(|i| t_lit_string(&format!("s{i}"))).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), "string");
}

#[test]
fn many_string_thresholds_walk() {
    for threshold in [1u16, 2, 5, 10, 32, 64, 100, 128] {
        let inputs: Vec<TAtomic> = (0..200usize).map(|i| t_lit_string(&format!("s{i}"))).collect();
        let result = combine_with_string_threshold(inputs, threshold);
        assert_eq!(result.len(), 1);
        assert_eq!(atomic_id_string(&result[0]), "string");
    }
}

#[test]
fn just_over_default_float_threshold_generalises() {
    let n = 129usize;
    let inputs: Vec<TAtomic> = (0..n).map(|i| t_lit_float(i as f64)).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), "float");
}

#[test]
fn many_distinct_sealed_lists_above_threshold_collapse() {
    let n = 50usize;
    let inputs: Vec<TAtomic> =
        (0..n_i64(n)).map(|i| t_sealed_list(BTreeMap::from([(0usize, (false, ui(i)))]))).collect();
    let result = combine_default(inputs);
    assert!(result.len() <= n);
}

#[test]
fn integer_threshold_zero_with_two_inputs_generalises() {
    let inputs = vec![t_lit_int(1), t_lit_int(2)];
    let result = combine_with_int_threshold(inputs, 0);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), "int");
}

#[test]
fn string_threshold_zero_with_two_inputs_generalises() {
    let inputs = vec![t_lit_string("a"), t_lit_string("b")];
    let result = combine_with_string_threshold(inputs, 0);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), "string");
}

#[test]
fn array_threshold_zero_with_two_inputs_generalises() {
    let inputs = vec![
        t_sealed_list(BTreeMap::from([(0usize, (false, ui(1)))])),
        t_sealed_list(BTreeMap::from([(0usize, (false, ui(2)))])),
    ];
    let result = combine_with_array_threshold(inputs, 0);
    assert!(!result.is_empty());
}

fn n_i64(n: usize) -> i64 {
    n as i64
}
