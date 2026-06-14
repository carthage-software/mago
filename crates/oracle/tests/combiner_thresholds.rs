mod common;

use common::*;

use mago_oracle::ty::atom::payload::array::KnownElement;

#[test]
fn at_default_int_threshold_keeps_non_adjacent_literals() {
    fixture(|f| {
        let n = 128usize;
        let inputs: Vec<_> = (0..n).map(|i| f.t_lit_int((i as i64) * 10)).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result.len(), n);
    });
}

#[test]
fn just_over_default_int_threshold_generalises() {
    fixture(|f| {
        let n = 129usize;
        let inputs: Vec<_> = (0..n).map(|i| f.t_lit_int(i as i64)).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_int()]);
    });
}

#[test]
fn many_int_thresholds_walk() {
    fixture(|f| {
        for threshold in [1u16, 2, 5, 10, 32, 64, 100, 128] {
            let inputs: Vec<_> = (0..200i64).map(|value| f.t_lit_int(value)).collect();
            let result = combine_with_int_threshold(f, inputs, threshold);
            assert_eq!(result, vec![f.t_int()]);
        }
    });
}

#[test]
fn int_threshold_above_input_count_keeps_literals() {
    fixture(|f| {
        let inputs: Vec<_> = (0..50i64).map(|value| f.t_lit_int(value)).collect();
        let result = combine_with_int_threshold(f, inputs, 100);
        assert_eq!(result.len(), 50);
    });
}

#[test]
fn at_default_string_threshold_keeps_literals() {
    fixture(|f| {
        let n = 128usize;
        let inputs: Vec<_> = (0..n).map(|i| f.t_lit_string(&format!("s{i}"))).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result.len(), n);
    });
}

#[test]
fn just_over_default_string_threshold_generalises() {
    fixture(|f| {
        let n = 129usize;
        let inputs: Vec<_> = (0..n).map(|i| f.t_lit_string(&format!("s{i}"))).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_string()]);
    });
}

#[test]
fn many_string_thresholds_walk() {
    fixture(|f| {
        for threshold in [1u16, 2, 5, 10, 32, 64, 100, 128] {
            let inputs: Vec<_> = (0..200usize).map(|i| f.t_lit_string(&format!("s{i}"))).collect();
            let result = combine_with_string_threshold(f, inputs, threshold);
            assert_eq!(result, vec![f.t_string()]);
        }
    });
}

#[test]
fn just_over_default_float_threshold_generalises() {
    fixture(|f| {
        let n = 129usize;
        let inputs: Vec<_> = (0..n).map(|i| f.t_lit_float(i as f64)).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_float()]);
    });
}

#[test]
fn many_distinct_sealed_lists_above_threshold_collapse() {
    fixture(|f| {
        let sealed_lists: Vec<_> = (0..40)
            .map(|i| {
                let value = f.us(&format!("v{i}"));
                f.builder.sealed_list(&[KnownElement { index: 0, value, optional: false }], false)
            })
            .collect();
        let result = combine_with_array_threshold(f, sealed_lists, 32);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_string(), "array<array-key, mixed>");
    });
}

#[test]
fn integer_threshold_zero_with_two_inputs_generalises() {
    fixture(|f| {
        let inputs = vec![f.t_lit_int(1), f.t_lit_int(2)];
        let result = combine_with_int_threshold(f, inputs, 0);
        assert_eq!(result, vec![f.t_int()]);
    });
}

#[test]
fn string_threshold_zero_with_two_inputs_generalises() {
    fixture(|f| {
        let letter_a = f.t_lit_string("a");
        let letter_b = f.t_lit_string("b");
        let result = combine_with_string_threshold(f, vec![letter_a, letter_b], 0);
        assert_eq!(result, vec![f.t_string()]);
    });
}

#[test]
fn array_threshold_zero_with_two_inputs_generalises() {
    fixture(|f| {
        let first_element = f.us("a");
        let first_list = f.t_list(first_element, false);
        let second_element = f.us("b");
        let second_list = f.t_list(second_element, false);
        let result = combine_with_array_threshold(f, vec![first_list, second_list], 0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_string(), "array<array-key, mixed>");
    });
}
