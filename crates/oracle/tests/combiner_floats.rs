mod common;

use common::*;

#[test]
fn idempotent_general_float() {
    fixture(|f| {
        for n in 1..=10 {
            assert_self_idempotent(f, f.t_float(), n);
        }
    });
}

#[test]
fn idempotent_unspec_literal() {
    fixture(|f| {
        for n in 1..=10 {
            assert_self_idempotent(f, f.t_unspec_lit_float(), n);
        }
    });
}

#[test]
fn idempotent_specific_literal() {
    fixture(|f| {
        for v in [-3.25f64, -1.0, 0.0, 1.0, 1.5, 1e10, 1e-10, 0.5] {
            for n in 1..=8 {
                assert_self_idempotent(f, f.t_lit_float(v), n);
            }
        }
    });
}

#[test]
fn float_absorbs_literal_either_order() {
    fixture(|f| {
        for v in [-100.0f64, 0.0, 1.5, 100.0] {
            assert_combines_to(f, vec![f.t_float(), f.t_lit_float(v)], vec![f.t_float()]);
            assert_combines_to(f, vec![f.t_lit_float(v), f.t_float()], vec![f.t_float()]);
        }
    });
}

#[test]
fn float_absorbs_unspec_literal_either_order() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_float(), f.t_unspec_lit_float()]);
        assert!(result.contains(&f.t_float()), "expected float in {result:?}");
    });
}

#[test]
fn float_absorbs_many_literals() {
    fixture(|f| {
        let mut inputs = vec![f.t_float()];
        for i in 0..30 {
            inputs.push(f.t_lit_float(f64::from(i)));
        }
        assert_combines_to(f, inputs, vec![f.t_float()]);
    });
}

#[test]
fn two_distinct_literals_kept() {
    fixture(|f| {
        for (a, b) in [(0.0f64, 1.0), (-1.0, 1.0), (1.5, 2.5), (-3.25, 3.25)] {
            let result = combine_default(f, vec![f.t_lit_float(a), f.t_lit_float(b)]);
            assert_eq!(result.len(), 2, "{a} | {b}");
        }
    });
}

#[test]
fn n_distinct_literals_kept() {
    fixture(|f| {
        for n in [3usize, 5, 10, 50] {
            let inputs: Vec<_> = (0..n).map(|i| f.t_lit_float(i as f64 + 0.5)).collect();
            let result = combine_default(f, inputs);
            assert_eq!(result.len(), n);
        }
    });
}

#[test]
fn duplicate_literal_floats_collapse() {
    fixture(|f| {
        for v in [0.0f64, 1.5, -3.25, 1e10] {
            for n in 1..=10 {
                assert_combines_to(f, vec![f.t_lit_float(v); n], vec![f.t_lit_float(v)]);
            }
        }
    });
}

#[test]
fn many_distinct_literals_exceed_threshold_generalise() {
    fixture(|f| {
        let n = 200usize;
        let inputs: Vec<_> = (0..n).map(|i| f.t_lit_float(i as f64)).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], f.t_float());
    });
}

#[test]
fn under_threshold_keeps_literals() {
    fixture(|f| {
        let n = 100usize;
        let inputs: Vec<_> = (0..n).map(|i| f.t_lit_float(i as f64)).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result.len(), n);
    });
}

#[test]
fn custom_low_threshold_generalises_quickly() {
    fixture(|f| {
        let inputs: Vec<_> = (0..20usize).map(|i| f.t_lit_float(i as f64)).collect();
        let result = combine_with_float_threshold(f, inputs, 5);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], f.t_float());
    });
}

#[test]
fn negative_zero_collapses_with_zero() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_lit_float(-0.0), f.t_lit_float(0.0)]);
        assert!(result.len() <= 2);
    });
}

#[test]
fn float_int_kept_separate() {
    fixture(|f| {
        let mut result = combine_default(f, vec![f.t_float(), f.t_int()]);
        result.sort();
        let mut expected = vec![f.t_float(), f.t_int()];
        expected.sort();
        assert_eq!(result, expected);
    });
}

#[test]
fn lit_float_lit_int_kept_separate() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_lit_float(1.5), f.t_lit_int(1)]);
        assert_eq!(result.len(), 2);
    });
}
