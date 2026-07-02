mod common;

use common::*;

#[test]
fn idempotent_unspec() {
    fixture(|f| {
        for n in 1..=10 {
            assert_self_idempotent(f, f.t_int(), n);
        }
    });
}

#[test]
fn idempotent_unspec_literal() {
    fixture(|f| {
        for n in 1..=10 {
            assert_self_idempotent(f, f.t_int_unspec_lit(), n);
        }
    });
}

#[test]
fn idempotent_literal() {
    fixture(|f| {
        for v in [-1_000_000i64, -100, -1, 0, 1, 100, 1_000_000, i64::MIN + 1, i64::MAX - 1] {
            for n in 1..=10 {
                assert_self_idempotent(f, f.t_lit_int(v), n);
            }
        }
    });
}

#[test]
fn idempotent_from() {
    fixture(|f| {
        for v in [-100i64, -1, 0, 1, 100, i64::MIN + 1] {
            let from = f.t_int_from(v);
            for n in 1..=5 {
                assert_self_idempotent(f, from, n);
            }
        }
    });
}

#[test]
fn idempotent_to() {
    fixture(|f| {
        for v in [-100i64, -1, 0, 1, 100, i64::MAX - 1] {
            let to = f.t_int_to(v);
            for n in 1..=5 {
                assert_self_idempotent(f, to, n);
            }
        }
    });
}

#[test]
fn idempotent_range() {
    fixture(|f| {
        for (lo, hi) in [(-100i64, 100), (-1, 1), (i64::MIN + 1, i64::MAX - 1)] {
            let range = f.t_int_range(lo, hi);
            for n in 1..=5 {
                assert_self_idempotent(f, range, n);
            }
        }
    });
}

#[test]
fn singleton_range_normalises_to_literal() {
    fixture(|f| {
        for v in [-100i64, -1, 0, 1, 42, 100] {
            let range = f.t_int_range(v, v);
            let result = combine_default(f, vec![range, range]);
            assert_eq!(result, vec![f.t_lit_int(v)]);
        }
    });
}

#[test]
fn idempotent_named_ranges() {
    fixture(|f| {
        for atom in [f.t_positive_int(), f.t_negative_int(), f.t_non_negative_int(), f.t_non_positive_int()] {
            for n in 1..=10 {
                assert_self_idempotent(f, atom, n);
            }
        }
    });
}

#[test]
fn unspecified_absorbs_literals() {
    fixture(|f| {
        for v in [-100i64, -1, 0, 1, 100, 12345] {
            assert_combines_to(f, vec![f.t_int(), f.t_lit_int(v)], vec![f.t_int()]);
            assert_combines_to(f, vec![f.t_lit_int(v), f.t_int()], vec![f.t_int()]);
        }
    });
}

#[test]
fn unspecified_absorbs_ranges() {
    fixture(|f| {
        for (lo, hi) in [(0i64, 10), (-5, 5), (i64::MIN + 1, 0), (0, i64::MAX - 1)] {
            let range = f.t_int_range(lo, hi);
            assert_combines_to(f, vec![f.t_int(), range], vec![f.t_int()]);
            assert_combines_to(f, vec![range, f.t_int()], vec![f.t_int()]);
        }
    });
}

#[test]
fn unspecified_absorbs_from() {
    fixture(|f| {
        for v in [-100i64, -1, 0, 1, 100] {
            let from = f.t_int_from(v);
            assert_combines_to(f, vec![f.t_int(), from], vec![f.t_int()]);
            assert_combines_to(f, vec![from, f.t_int()], vec![f.t_int()]);
        }
    });
}

#[test]
fn unspecified_absorbs_to() {
    fixture(|f| {
        for v in [-100i64, -1, 0, 1, 100] {
            let to = f.t_int_to(v);
            assert_combines_to(f, vec![f.t_int(), to], vec![f.t_int()]);
            assert_combines_to(f, vec![to, f.t_int()], vec![f.t_int()]);
        }
    });
}

#[test]
fn unspecified_absorbs_named() {
    fixture(|f| {
        for atom in [f.t_positive_int(), f.t_negative_int(), f.t_non_negative_int(), f.t_non_positive_int()] {
            assert_combines_to(f, vec![f.t_int(), atom], vec![f.t_int()]);
            assert_combines_to(f, vec![atom, f.t_int()], vec![f.t_int()]);
        }
    });
}

#[test]
fn unspecified_absorbs_unspecified_literal() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_int(), f.t_int_unspec_lit()], vec![f.t_int()]);
        assert_combines_to(f, vec![f.t_int_unspec_lit(), f.t_int()], vec![f.t_int()]);
    });
}

#[test]
fn two_distinct_non_adjacent_literals_kept() {
    fixture(|f| {
        for (a, b) in [(-1i64, 1), (-100, 100), (10, 20)] {
            let result = combine_default(f, vec![f.t_lit_int(a), f.t_lit_int(b)]);
            assert_eq!(result.len(), 2, "{a} | {b}");
        }
    });
}

#[test]
fn two_adjacent_literals_merge_to_range() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_lit_int(0), f.t_lit_int(1)]);
        assert_eq!(result, vec![f.t_int_range(0, 1)]);
    });
}

#[test]
fn n_consecutive_literals_merge_to_range() {
    fixture(|f| {
        let inputs: Vec<_> = (0..5i64).map(|i| f.t_lit_int(i)).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_int_range(0, 4)]);
    });
}

#[test]
fn n_non_adjacent_literals_kept() {
    fixture(|f| {
        let inputs: Vec<_> = (0..5i64).map(|i| f.t_lit_int(i * 10)).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result.len(), 5);
    });
}

#[test]
fn literals_with_duplicates_collapse() {
    fixture(|f| {
        for v in [0i64, 1, -1, 42] {
            assert_combines_to(f, vec![f.t_lit_int(v); 10], vec![f.t_lit_int(v)]);
        }
    });
}

#[test]
fn duplicates_in_mixed_literals_dedup_then_merge() {
    fixture(|f| {
        let inputs = vec![f.t_lit_int(1), f.t_lit_int(2), f.t_lit_int(1), f.t_lit_int(2), f.t_lit_int(3)];
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_int_range(1, 3)]);
    });
}

#[test]
fn overlapping_ranges_merge() {
    fixture(|f| {
        let inputs = vec![f.t_int_range(0, 10), f.t_int_range(5, 15)];
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_int_range(0, 15)]);
    });
}

#[test]
fn adjacent_ranges_merge() {
    fixture(|f| {
        let inputs = vec![f.t_int_range(0, 10), f.t_int_range(11, 20)];
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_int_range(0, 20)]);
    });
}

#[test]
fn disjoint_ranges_kept_apart() {
    fixture(|f| {
        let inputs = vec![f.t_int_range(0, 10), f.t_int_range(20, 30)];
        let result = combine_default(f, inputs);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn equal_ranges_collapse() {
    fixture(|f| {
        for (lo, hi) in [(0i64, 10), (-5, 5), (-100, 100)] {
            let range = f.t_int_range(lo, hi);
            assert_combines_to(f, vec![range; 5], vec![range]);
        }
    });
}

#[test]
fn nested_ranges_merge_to_outer() {
    fixture(|f| {
        let inputs = vec![f.t_int_range(0, 100), f.t_int_range(10, 20)];
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_int_range(0, 100)]);
    });
}

#[test]
fn many_ranges_merge_chain() {
    fixture(|f| {
        let inputs = vec![f.t_int_range(0, 10), f.t_int_range(11, 20), f.t_int_range(21, 30)];
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_int_range(0, 30)]);
    });
}

#[test]
fn range_absorbs_literal_inside() {
    fixture(|f| {
        let range = f.t_int_range(0, 10);
        for v in 0..=10i64 {
            assert_combines_to(f, vec![range, f.t_lit_int(v)], vec![range]);
            assert_combines_to(f, vec![f.t_lit_int(v), range], vec![range]);
        }
    });
}

#[test]
fn range_keeps_literal_outside() {
    fixture(|f| {
        let range = f.t_int_range(0, 10);
        let result = combine_default(f, vec![range, f.t_lit_int(20)]);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn range_extends_with_adjacent_literal() {
    fixture(|f| {
        let range = f.t_int_range(0, 10);
        let result = combine_default(f, vec![range, f.t_lit_int(11)]);
        assert_eq!(result, vec![f.t_int_range(0, 11)]);
    });
}

#[test]
fn literal_extends_lower_with_adjacent_range() {
    fixture(|f| {
        let range = f.t_int_range(0, 10);
        let result = combine_default(f, vec![f.t_lit_int(-1), range]);
        assert_eq!(result, vec![f.t_int_range(-1, 10)]);
    });
}

#[test]
fn from_absorbs_literal_above() {
    fixture(|f| {
        let from = f.t_int_from(0);
        for v in [0i64, 1, 5, 10, 100, 1_000_000] {
            assert_combines_to(f, vec![from, f.t_lit_int(v)], vec![from]);
            assert_combines_to(f, vec![f.t_lit_int(v), from], vec![from]);
        }
    });
}

#[test]
fn from_with_literal_one_below_extends() {
    fixture(|f| {
        for n in [0i64, 5, 100, -1] {
            let from = f.t_int_from(n);
            let result = combine_default(f, vec![from, f.t_lit_int(n - 1)]);
            assert_eq!(result, vec![f.t_int_from(n - 1)]);
        }
    });
}

#[test]
fn from_keeps_literal_far_below() {
    fixture(|f| {
        let from = f.t_int_from(0);
        for v in [-2i64, -5, -100, -1_000_000] {
            let result = combine_default(f, vec![from, f.t_lit_int(v)]);
            assert_eq!(result.len(), 2, "From(0) | Literal({v})");
        }
    });
}

#[test]
fn to_absorbs_literal_below() {
    fixture(|f| {
        let to = f.t_int_to(0);
        for v in [-100i64, -1, 0] {
            assert_combines_to(f, vec![to, f.t_lit_int(v)], vec![to]);
            assert_combines_to(f, vec![f.t_lit_int(v), to], vec![to]);
        }
    });
}

#[test]
fn to_with_literal_one_above_extends() {
    fixture(|f| {
        for n in [0i64, 5, -1, -100] {
            let to = f.t_int_to(n);
            let result = combine_default(f, vec![to, f.t_lit_int(n + 1)]);
            assert_eq!(result, vec![f.t_int_to(n + 1)]);
        }
    });
}

#[test]
fn to_keeps_literal_far_above() {
    fixture(|f| {
        let to = f.t_int_to(0);
        for v in [2i64, 5, 100, 1_000_000] {
            let result = combine_default(f, vec![to, f.t_lit_int(v)]);
            assert_eq!(result.len(), 2, "To(0) | Literal({v})");
        }
    });
}

#[test]
fn from_and_to_with_overlap_become_unspecified() {
    fixture(|f| {
        let inputs = vec![f.t_int_from(0), f.t_int_to(0)];
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_int()]);
    });
}

#[test]
fn from_and_to_adjacent_become_unspecified() {
    fixture(|f| {
        let inputs = vec![f.t_int_from(1), f.t_int_to(0)];
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_int()]);
    });
}

#[test]
fn from_and_to_disjoint_kept_apart() {
    fixture(|f| {
        let inputs = vec![f.t_int_from(10), f.t_int_to(0)];
        let result = combine_default(f, inputs);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn from_lo_overrides_higher_from() {
    fixture(|f| {
        let from_zero = f.t_int_from(0);
        let from_five = f.t_int_from(5);
        assert_combines_to(f, vec![from_zero, from_five], vec![from_zero]);
        assert_combines_to(f, vec![from_five, from_zero], vec![from_zero]);
    });
}

#[test]
fn to_hi_overrides_lower_to() {
    fixture(|f| {
        let to_hundred = f.t_int_to(100);
        let to_fifty = f.t_int_to(50);
        assert_combines_to(f, vec![to_hundred, to_fifty], vec![to_hundred]);
        assert_combines_to(f, vec![to_fifty, to_hundred], vec![to_hundred]);
    });
}

#[test]
fn positive_int_is_from_1() {
    fixture(|f| {
        let from = f.t_int_from(1);
        let result = combine_default(f, vec![f.t_positive_int(), from]);
        assert_eq!(result, vec![f.t_positive_int()]);
    });
}

#[test]
fn non_negative_int_is_from_0() {
    fixture(|f| {
        let from = f.t_int_from(0);
        let result = combine_default(f, vec![f.t_non_negative_int(), from]);
        assert_eq!(result, vec![f.t_non_negative_int()]);
    });
}

#[test]
fn negative_int_is_to_minus_1() {
    fixture(|f| {
        let to = f.t_int_to(-1);
        let result = combine_default(f, vec![f.t_negative_int(), to]);
        assert_eq!(result, vec![f.t_negative_int()]);
    });
}

#[test]
fn non_positive_int_is_to_0() {
    fixture(|f| {
        let to = f.t_int_to(0);
        let result = combine_default(f, vec![f.t_non_positive_int(), to]);
        assert_eq!(result, vec![f.t_non_positive_int()]);
    });
}

#[test]
fn positive_or_negative_kept_apart() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_positive_int(), f.t_negative_int()]);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn non_negative_or_non_positive_become_unspecified() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_non_negative_int(), f.t_non_positive_int()]);
        assert_eq!(result, vec![f.t_int()]);
    });
}

#[test]
fn positive_absorbs_positive_literal() {
    fixture(|f| {
        for v in [1i64, 5, 100, i64::MAX - 1] {
            assert_combines_to(f, vec![f.t_positive_int(), f.t_lit_int(v)], vec![f.t_positive_int()]);
            assert_combines_to(f, vec![f.t_lit_int(v), f.t_positive_int()], vec![f.t_positive_int()]);
        }
    });
}

#[test]
fn positive_extends_with_zero_to_non_negative() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_positive_int(), f.t_lit_int(0)]);
        assert_eq!(result, vec![f.t_non_negative_int()]);
    });
}

#[test]
fn unspecified_literal_keeps_with_actual_literal() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_int_unspec_lit(), f.t_lit_int(5)]);
        assert_eq!(result, vec![f.t_int_unspec_lit()]);
    });
}

#[test]
fn unspecified_literal_with_range_keeps_both() {
    fixture(|f| {
        let range = f.t_int_range(0, 10);
        let result = combine_default(f, vec![f.t_int_unspec_lit(), range]);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn unspecified_literal_with_unspecified_collapses_to_unspecified() {
    fixture(|f| {
        assert_combines_to(f, vec![f.t_int_unspec_lit(), f.t_int()], vec![f.t_int()]);
        assert_combines_to(f, vec![f.t_int(), f.t_int_unspec_lit()], vec![f.t_int()]);
    });
}

#[test]
fn positive_or_zero_collapses_to_non_negative() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_positive_int(), f.t_lit_int(0)]);
        assert_eq!(result, vec![f.t_non_negative_int()]);
    });
}

#[test]
fn negative_or_zero_collapses_to_non_positive() {
    fixture(|f| {
        let result = combine_default(f, vec![f.t_negative_int(), f.t_lit_int(0)]);
        assert_eq!(result, vec![f.t_non_positive_int()]);
    });
}

#[test]
fn small_range_and_literal_extend() {
    fixture(|f| {
        let range = f.t_int_range(0, 5);
        let result = combine_default(f, vec![range, f.t_lit_int(6), f.t_lit_int(7)]);
        assert_eq!(result, vec![f.t_int_range(0, 7)]);
    });
}

#[test]
fn many_ranges_consecutive_merge() {
    fixture(|f| {
        let inputs: Vec<_> = (0..5i64).map(|i| f.t_int_range(i * 2, i * 2 + 1)).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_int_range(0, 9)]);
    });
}

#[test]
fn many_disjoint_ranges_kept_apart() {
    fixture(|f| {
        let inputs = vec![f.t_int_range(0, 5), f.t_int_range(10, 15), f.t_int_range(20, 25)];
        let result = combine_default(f, inputs);
        assert_eq!(result.len(), 3);
    });
}

#[test]
fn many_disjoint_literals_kept_apart() {
    fixture(|f| {
        let inputs = vec![f.t_lit_int(1), f.t_lit_int(10), f.t_lit_int(100), f.t_lit_int(1000)];
        let result = combine_default(f, inputs);
        assert_eq!(result.len(), 4);
    });
}

#[test]
fn consecutive_literals_merge_to_range() {
    fixture(|f| {
        let inputs: Vec<_> = (1..=5i64).map(|i| f.t_lit_int(i)).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_int_range(1, 5)]);
    });
}

#[test]
fn literal_far_from_from_kept_apart() {
    fixture(|f| {
        let from = f.t_int_from(5);
        let result = combine_default(f, vec![from, f.t_lit_int(0)]);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn literal_4_with_from_5_merges_to_from_4() {
    fixture(|f| {
        let from = f.t_int_from(5);
        let result = combine_default(f, vec![from, f.t_lit_int(4)]);
        assert_eq!(result, vec![f.t_int_from(4)]);
    });
}

#[test]
fn literal_n_minus_1_with_to_n_merges_to_to_n() {
    fixture(|f| {
        let to = f.t_int_to(5);
        let result = combine_default(f, vec![to, f.t_lit_int(6)]);
        assert_eq!(result, vec![f.t_int_to(6)]);
    });
}

#[test]
fn range_at_minmax_boundaries() {
    fixture(|f| {
        let almost_min = i64::MIN + 1;
        let almost_max = i64::MAX - 1;

        let range = f.t_int_range(almost_min, almost_max);
        let result = combine_default(f, vec![range]);
        assert_eq!(result.len(), 1);
    });
}

#[test]
fn literal_min_max() {
    fixture(|f| {
        let min = i64::MIN;
        let max = i64::MAX;
        assert_self_idempotent(f, f.t_lit_int(min), 3);
        assert_self_idempotent(f, f.t_lit_int(max), 3);
        let result = combine_default(f, vec![f.t_lit_int(min), f.t_lit_int(max)]);
        assert_eq!(result.len(), 2);
    });
}

#[test]
fn many_distinct_literals_exceed_threshold_generalise() {
    fixture(|f| {
        let n = 200usize;
        let inputs: Vec<_> = (0..n).map(|i| f.t_lit_int(i as i64)).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_int()]);
    });
}

#[test]
fn non_adjacent_literals_kept_under_threshold() {
    fixture(|f| {
        let n = 100usize;
        let inputs: Vec<_> = (0..n).map(|i| f.t_lit_int((i as i64) * 10)).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result.len(), n);
    });
}

#[test]
fn custom_low_threshold_generalises_quickly() {
    fixture(|f| {
        let n = 10usize;
        let inputs: Vec<_> = (0..n).map(|i| f.t_lit_int(i as i64)).collect();
        let result = combine_with_int_threshold(f, inputs, 5);
        assert_eq!(result, vec![f.t_int()]);
    });
}

#[test]
fn repeated_same_literal_collapses_to_single_literal() {
    fixture(|f| {
        let inputs = vec![f.t_lit_int(42); 200];
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_lit_int(42)]);
    });
}

#[test]
fn positive_int_or_range_extends() {
    fixture(|f| {
        let range = f.t_int_range(-5, 0);
        let result = combine_default(f, vec![f.t_positive_int(), range]);
        assert_eq!(result, vec![f.t_int_from(-5)]);
    });
}

#[test]
fn negative_int_or_range_extends() {
    fixture(|f| {
        let range = f.t_int_range(0, 5);
        let result = combine_default(f, vec![f.t_negative_int(), range]);
        assert_eq!(result, vec![f.t_int_to(5)]);
    });
}

#[test]
fn small_subranges_merge_into_named_range() {
    fixture(|f| {
        let inputs: Vec<_> = (0..5i64).map(|i| f.t_int_range(2 * i + 1, 2 * i + 2)).collect();
        let result = combine_default(f, inputs);
        assert_eq!(result, vec![f.t_int_range(1, 10)]);
    });
}

#[test]
fn integer_combine_order_independent() {
    fixture(|f| {
        let cases = [
            vec![f.t_lit_int(1), f.t_lit_int(2), f.t_lit_int(3)],
            vec![f.t_lit_int(0), f.t_int_range(5, 10), f.t_int_from(20)],
            vec![f.t_int(), f.t_lit_int(0), f.t_int_range(5, 10)],
            vec![f.t_positive_int(), f.t_negative_int(), f.t_lit_int(0)],
            vec![f.t_int_range(0, 5), f.t_int_range(10, 15), f.t_lit_int(20)],
            vec![f.t_int_range(0, 5), f.t_int_range(6, 10), f.t_int_range(11, 15)],
            vec![f.t_int_unspec_lit(), f.t_lit_int(5)],
        ];
        for case in cases {
            let r1 = combine_default(f, case.clone());
            let mut reversed = case;
            reversed.reverse();
            let r2 = combine_default(f, reversed);
            assert_multiset_eq(&r1, &r2);
        }
    });
}
