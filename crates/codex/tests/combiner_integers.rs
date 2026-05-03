mod combiner_common;

use combiner_common::*;

use mago_codex::ttype::atomic::TAtomic;

#[test]
fn idempotent_unspec() {
    for n in 1..=10 {
        assert_self_idempotent(t_int(), n);
    }
}

#[test]
fn idempotent_unspec_literal() {
    for n in 1..=10 {
        assert_self_idempotent(t_int_unspec_lit(), n);
    }
}

#[test]
fn idempotent_literal() {
    for v in [-1_000_000i64, -100, -1, 0, 1, 100, 1_000_000, i64::MIN + 1, i64::MAX - 1] {
        for n in 1..=10 {
            assert_self_idempotent(t_lit_int(v), n);
        }
    }
}

#[test]
fn idempotent_from() {
    for v in [-100i64, -1, 0, 1, 100, i64::MIN + 1] {
        for n in 1..=5 {
            assert_self_idempotent(t_int_from(v), n);
        }
    }
}

#[test]
fn idempotent_to() {
    for v in [-100i64, -1, 0, 1, 100, i64::MAX - 1] {
        for n in 1..=5 {
            assert_self_idempotent(t_int_to(v), n);
        }
    }
}

#[test]
fn idempotent_range() {
    for (lo, hi) in [(-100i64, 100), (-1, 1), (i64::MIN + 1, i64::MAX - 1)] {
        for n in 1..=5 {
            assert_self_idempotent(t_int_range(lo, hi), n);
        }
    }
}

#[test]
fn singleton_range_normalises_to_literal() {
    for v in [-100i64, -1, 0, 1, 42, 100] {
        let result = combine_default(vec![t_int_range(v, v), t_int_range(v, v)]);
        assert_eq!(result.len(), 1);
        assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_lit_int(v)));
    }
}

#[test]
fn idempotent_named_ranges() {
    for atom in [t_positive_int(), t_negative_int(), t_non_negative_int(), t_non_positive_int()] {
        for n in 1..=10 {
            assert_self_idempotent(atom.clone(), n);
        }
    }
}

#[test]
fn unspecified_absorbs_literals() {
    for v in [-100i64, -1, 0, 1, 100, 12345] {
        assert_combines_to(vec![t_int(), t_lit_int(v)], vec![t_int()]);
        assert_combines_to(vec![t_lit_int(v), t_int()], vec![t_int()]);
    }
}

#[test]
fn unspecified_absorbs_ranges() {
    for (lo, hi) in [(0i64, 10), (-5, 5), (i64::MIN + 1, 0), (0, i64::MAX - 1)] {
        assert_combines_to(vec![t_int(), t_int_range(lo, hi)], vec![t_int()]);
        assert_combines_to(vec![t_int_range(lo, hi), t_int()], vec![t_int()]);
    }
}

#[test]
fn unspecified_absorbs_from() {
    for v in [-100i64, -1, 0, 1, 100] {
        assert_combines_to(vec![t_int(), t_int_from(v)], vec![t_int()]);
        assert_combines_to(vec![t_int_from(v), t_int()], vec![t_int()]);
    }
}

#[test]
fn unspecified_absorbs_to() {
    for v in [-100i64, -1, 0, 1, 100] {
        assert_combines_to(vec![t_int(), t_int_to(v)], vec![t_int()]);
        assert_combines_to(vec![t_int_to(v), t_int()], vec![t_int()]);
    }
}

#[test]
fn unspecified_absorbs_named() {
    for atom in [t_positive_int(), t_negative_int(), t_non_negative_int(), t_non_positive_int()] {
        assert_combines_to(vec![t_int(), atom.clone()], vec![t_int()]);
        assert_combines_to(vec![atom, t_int()], vec![t_int()]);
    }
}

#[test]
fn unspecified_absorbs_unspecified_literal() {
    assert_combines_to(vec![t_int(), t_int_unspec_lit()], vec![t_int()]);
    assert_combines_to(vec![t_int_unspec_lit(), t_int()], vec![t_int()]);
}

#[test]
fn two_distinct_literals_kept() {
    for (a, b) in [(0i64, 1), (-1, 1), (-100, 100), (10, 20)] {
        let result = combine_default(vec![t_lit_int(a), t_lit_int(b)]);
        assert_eq!(result.len(), 2, "{a} ∨ {b}");
    }
}

#[test]
fn n_distinct_literals_kept() {
    for n in [3usize, 5, 10, 32, 64] {
        let inputs: Vec<TAtomic> = (0..n).map(|i| t_lit_int(i as i64)).collect();
        let result = combine_default(inputs);
        assert_eq!(result.len(), n, "n={n}");
    }
}

#[test]
fn literals_with_duplicates_collapse() {
    for v in [0i64, 1, -1, 42] {
        let inputs: Vec<TAtomic> = vec![t_lit_int(v); 10];
        assert_combines_to(inputs, vec![t_lit_int(v)]);
    }
}

#[test]
fn duplicates_in_mixed_literals_dedup() {
    let inputs = vec![t_lit_int(1), t_lit_int(2), t_lit_int(1), t_lit_int(2), t_lit_int(3)];
    let result = combine_default(inputs);
    assert_eq!(result.len(), 3);
}

#[test]
fn overlapping_ranges_merge() {
    let result = combine_default(vec![t_int_range(0, 10), t_int_range(5, 15)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_range(0, 15)));
}

#[test]
fn adjacent_ranges_merge() {
    let result = combine_default(vec![t_int_range(0, 10), t_int_range(11, 20)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_range(0, 20)));
}

#[test]
fn disjoint_ranges_kept_apart() {
    let result = combine_default(vec![t_int_range(0, 10), t_int_range(20, 30)]);
    assert_eq!(result.len(), 2);
}

#[test]
fn equal_ranges_collapse() {
    for (lo, hi) in [(0i64, 10), (-5, 5), (-100, 100)] {
        assert_combines_to(vec![t_int_range(lo, hi); 5], vec![t_int_range(lo, hi)]);
    }
}

#[test]
fn nested_ranges_merge_to_outer() {
    let result = combine_default(vec![t_int_range(0, 100), t_int_range(10, 20)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_range(0, 100)));
}

#[test]
fn many_ranges_merge_chain() {
    let result = combine_default(vec![t_int_range(0, 10), t_int_range(11, 20), t_int_range(21, 30)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_range(0, 30)));
}

#[test]
fn range_absorbs_literal_inside() {
    for v in 0..=10i64 {
        assert_combines_to(vec![t_int_range(0, 10), t_lit_int(v)], vec![t_int_range(0, 10)]);
        assert_combines_to(vec![t_lit_int(v), t_int_range(0, 10)], vec![t_int_range(0, 10)]);
    }
}

#[test]
fn range_keeps_literal_outside() {
    let result = combine_default(vec![t_int_range(0, 10), t_lit_int(20)]);
    assert_eq!(result.len(), 2);
}

#[test]
fn range_extends_with_adjacent_literal() {
    let result = combine_default(vec![t_int_range(0, 10), t_lit_int(11)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_range(0, 11)));
}

#[test]
fn literal_extends_lower_with_adjacent_range() {
    let result = combine_default(vec![t_lit_int(-1), t_int_range(0, 10)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_range(-1, 10)));
}

#[test]
fn from_absorbs_literal_above() {
    for v in [0i64, 1, 5, 10, 100, 1_000_000] {
        assert_combines_to(vec![t_int_from(0), t_lit_int(v)], vec![t_int_from(0)]);
        assert_combines_to(vec![t_lit_int(v), t_int_from(0)], vec![t_int_from(0)]);
    }
}

#[test]
fn from_with_literal_one_below_extends() {
    for n in [0i64, 5, 100, -1] {
        let result = combine_default(vec![t_int_from(n), t_lit_int(n - 1)]);
        assert_eq!(result.len(), 1);
        assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_from(n - 1)));
    }
}

#[test]
fn from_keeps_literal_far_below() {
    for v in [-2i64, -5, -100, -1_000_000] {
        let result = combine_default(vec![t_int_from(0), t_lit_int(v)]);
        assert_eq!(result.len(), 2, "From(0) ∨ Literal({v})");
    }
}

#[test]
fn to_absorbs_literal_below() {
    for v in [-100i64, -1, 0] {
        assert_combines_to(vec![t_int_to(0), t_lit_int(v)], vec![t_int_to(0)]);
        assert_combines_to(vec![t_lit_int(v), t_int_to(0)], vec![t_int_to(0)]);
    }
}

#[test]
fn to_with_literal_one_above_extends() {
    for n in [0i64, 5, -1, -100] {
        let result = combine_default(vec![t_int_to(n), t_lit_int(n + 1)]);
        assert_eq!(result.len(), 1);
        assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_to(n + 1)));
    }
}

#[test]
fn to_keeps_literal_far_above() {
    for v in [2i64, 5, 100, 1_000_000] {
        let result = combine_default(vec![t_int_to(0), t_lit_int(v)]);
        assert_eq!(result.len(), 2, "To(0) ∨ Literal({v})");
    }
}

#[test]
fn from_and_to_with_overlap_become_unspecified() {
    let result = combine_default(vec![t_int_from(0), t_int_to(0)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int()));
}

#[test]
fn from_and_to_adjacent_become_unspecified() {
    let result = combine_default(vec![t_int_from(1), t_int_to(0)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int()));
}

#[test]
fn from_and_to_disjoint_kept_apart() {
    let result = combine_default(vec![t_int_from(10), t_int_to(0)]);
    assert_eq!(result.len(), 2);
}

#[test]
fn from_lo_overrides_higher_from() {
    assert_combines_to(vec![t_int_from(0), t_int_from(5)], vec![t_int_from(0)]);
    assert_combines_to(vec![t_int_from(5), t_int_from(0)], vec![t_int_from(0)]);
}

#[test]
fn to_hi_overrides_lower_to() {
    assert_combines_to(vec![t_int_to(100), t_int_to(50)], vec![t_int_to(100)]);
    assert_combines_to(vec![t_int_to(50), t_int_to(100)], vec![t_int_to(100)]);
}

#[test]
fn positive_int_is_from_1() {
    let positive = t_positive_int();
    let result = combine_default(vec![positive.clone(), t_int_from(1)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&positive));
}

#[test]
fn non_negative_int_is_from_0() {
    let nn = t_non_negative_int();
    let result = combine_default(vec![nn.clone(), t_int_from(0)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&nn));
}

#[test]
fn negative_int_is_to_minus_1() {
    let neg = t_negative_int();
    let result = combine_default(vec![neg.clone(), t_int_to(-1)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&neg));
}

#[test]
fn non_positive_int_is_to_0() {
    let np = t_non_positive_int();
    let result = combine_default(vec![np.clone(), t_int_to(0)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&np));
}

#[test]
fn positive_or_negative_kept_apart() {
    let result = combine_default(vec![t_positive_int(), t_negative_int()]);
    assert_eq!(result.len(), 2);
}

#[test]
fn non_negative_or_non_positive_become_unspecified() {
    let result = combine_default(vec![t_non_negative_int(), t_non_positive_int()]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int()));
}

#[test]
fn positive_absorbs_positive_literal() {
    for v in [1i64, 5, 100, i64::MAX - 1] {
        assert_combines_to(vec![t_positive_int(), t_lit_int(v)], vec![t_positive_int()]);
        assert_combines_to(vec![t_lit_int(v), t_positive_int()], vec![t_positive_int()]);
    }
}

#[test]
fn positive_extends_with_zero_to_non_negative() {
    let result = combine_default(vec![t_positive_int(), t_lit_int(0)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_non_negative_int()));
}

#[test]
fn unspecified_literal_keeps_with_actual_literal() {
    let result = combine_default(vec![t_int_unspec_lit(), t_lit_int(5)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_unspec_lit()));
}

#[test]
fn unspecified_literal_with_range_keeps_both() {
    let result = combine_default(vec![t_int_unspec_lit(), t_int_range(0, 10)]);
    assert_eq!(result.len(), 2);
}

#[test]
fn unspecified_literal_with_unspecified_collapses_to_unspecified() {
    assert_combines_to(vec![t_int_unspec_lit(), t_int()], vec![t_int()]);
    assert_combines_to(vec![t_int(), t_int_unspec_lit()], vec![t_int()]);
}

#[test]
fn positive_or_zero_collapses_to_non_negative() {
    let result = combine_default(vec![t_positive_int(), t_lit_int(0)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_non_negative_int()));
}

#[test]
fn negative_or_zero_collapses_to_non_positive() {
    let result = combine_default(vec![t_negative_int(), t_lit_int(0)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_non_positive_int()));
}

#[test]
fn small_range_and_literal_extend() {
    let result = combine_default(vec![t_int_range(0, 5), t_lit_int(6), t_lit_int(7)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_range(0, 7)));
}

#[test]
fn many_ranges_consecutive_merge() {
    let inputs: Vec<TAtomic> = (0..5).map(|i| t_int_range(i * 2, i * 2 + 1)).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_range(0, 9)));
}

#[test]
fn many_disjoint_ranges_kept_apart() {
    let inputs = vec![t_int_range(0, 5), t_int_range(10, 15), t_int_range(20, 25)];
    let result = combine_default(inputs);
    assert_eq!(result.len(), 3);
}

#[test]
fn many_disjoint_literals_kept_apart() {
    let inputs = vec![t_lit_int(1), t_lit_int(10), t_lit_int(100), t_lit_int(1000)];
    let result = combine_default(inputs);
    assert_eq!(result.len(), 4);
}

#[test]
fn consecutive_literals_merge_to_range() {
    let inputs: Vec<TAtomic> = (1..=5i64).map(t_lit_int).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), 5);
}

#[test]
fn literal_with_from_merges_correctly() {
    let result = combine_default(vec![t_int_from(5), t_lit_int(0)]);
    assert_eq!(result.len(), 2);
}

#[test]
fn literal_4_with_from_5_merges_to_from_4() {
    let result = combine_default(vec![t_int_from(5), t_lit_int(4)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_from(4)));
}

#[test]
fn literal_n_minus_1_with_to_n_merges_to_to_n() {
    let result = combine_default(vec![t_int_to(5), t_lit_int(6)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_to(6)));
}

#[test]
fn range_at_minmax_boundaries() {
    let almost_min = i64::MIN + 1;
    let almost_max = i64::MAX - 1;

    let result = combine_default(vec![t_int_range(almost_min, almost_max)]);
    assert_eq!(result.len(), 1);

    let result = combine_default(vec![t_int_from(almost_min), t_int_to(almost_max)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int()));
}

#[test]
fn literal_min_max() {
    let min = i64::MIN;
    let max = i64::MAX;
    assert_self_idempotent(t_lit_int(min), 3);
    assert_self_idempotent(t_lit_int(max), 3);
    let result = combine_default(vec![t_lit_int(min), t_lit_int(max)]);
    assert_eq!(result.len(), 2);
}

#[test]
fn many_distinct_literals_exceed_threshold_generalise() {
    let n = 200usize;
    let inputs: Vec<TAtomic> = (0..n).map(|i| t_lit_int(i as i64)).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int()));
}

#[test]
fn under_threshold_keeps_literals() {
    let n = 100usize;
    let inputs: Vec<TAtomic> = (0..n).map(|i| t_lit_int(i as i64)).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), n);
}

#[test]
fn custom_low_threshold_generalises_quickly() {
    let n = 10usize;
    let inputs: Vec<TAtomic> = (0..n).map(|i| t_lit_int(i as i64)).collect();
    let result = combine_with_int_threshold(inputs, 5);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int()));
}

#[test]
fn repeated_same_literal_under_threshold_collapses() {
    let inputs = vec![t_lit_int(42); 200];
    let result = combine_default(inputs);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int()));
}

#[test]
fn positive_int_or_range_extends() {
    let result = combine_default(vec![t_positive_int(), t_int_range(-5, 0)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_from(-5)));
}

#[test]
fn negative_int_or_range_extends() {
    let result = combine_default(vec![t_negative_int(), t_int_range(0, 5)]);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_to(5)));
}

#[test]
fn small_subranges_merge_into_named_range() {
    let inputs: Vec<TAtomic> = (0..5).map(|i| t_int_range(2 * i + 1, 2 * i + 2)).collect();
    let result = combine_default(inputs);
    assert_eq!(result.len(), 1);
    assert_eq!(atomic_id_string(&result[0]), atomic_id_string(&t_int_range(1, 10)));
}

// Order-independence — for inputs without absorption asymmetries (no `numeric`,
// no `scalar`), integer combine should be order-independent.

#[test]
fn integer_combine_order_independent() {
    let cases: Vec<Vec<TAtomic>> = vec![
        vec![t_lit_int(1), t_lit_int(2), t_lit_int(3)],
        vec![t_lit_int(0), t_int_range(5, 10), t_int_from(20)],
        vec![t_int(), t_lit_int(0), t_int_range(5, 10)],
        vec![t_positive_int(), t_negative_int(), t_lit_int(0)],
        vec![t_int_range(0, 5), t_int_range(10, 15), t_lit_int(20)],
        vec![t_int_range(0, 5), t_int_range(6, 10), t_int_range(11, 15)],
        vec![t_int_unspec_lit(), t_lit_int(5)],
    ];
    for case in cases {
        let mut r1 = combine_default(case.clone());
        let mut reversed = case;
        reversed.reverse();
        let mut r2 = combine_default(reversed);

        let mut a: Vec<String> = r1.iter().map(atomic_id_string).collect();
        let mut b: Vec<String> = r2.iter().map(atomic_id_string).collect();
        a.sort();
        b.sort();
        assert_eq!(a, b);

        r1.sort_unstable();
        r2.sort_unstable();
    }
}
