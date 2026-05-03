mod comparator_common;

use comparator_common::*;

#[test]
fn int_reflexive() {
    assert_atomic_subtype(&t_int(), &t_int());
}

#[test]
fn lit_int_reflexive_for_many_values() {
    for v in -50..=50i64 {
        assert_atomic_subtype(&t_lit_int(v), &t_lit_int(v));
    }
}

#[test]
fn int_contains_every_literal() {
    for v in -200..=200i64 {
        assert_atomic_subtype(&t_lit_int(v), &t_int());
    }
}

#[test]
fn int_does_not_contain_specific_literal() {
    for v in [-100i64, 0, 100] {
        assert_atomic_not_subtype(&t_int(), &t_lit_int(v));
    }
}

#[test]
fn distinct_literals_are_disjoint() {
    for a in -10..=10i64 {
        for b in -10..=10i64 {
            if a == b {
                continue;
            }
            assert_atomic_not_subtype(&t_lit_int(a), &t_lit_int(b));
        }
    }
}

#[test]
fn positive_int_contains_strictly_positive_literals() {
    for v in 1..=100i64 {
        assert_atomic_subtype(&t_lit_int(v), &t_positive_int());
    }
}

#[test]
fn positive_int_does_not_contain_zero() {
    assert_atomic_not_subtype(&t_lit_int(0), &t_positive_int());
}

#[test]
fn positive_int_does_not_contain_negatives() {
    for v in [-1i64, -10, -100] {
        assert_atomic_not_subtype(&t_lit_int(v), &t_positive_int());
    }
}

#[test]
fn non_negative_int_contains_zero_and_positives() {
    for v in 0..=100i64 {
        assert_atomic_subtype(&t_lit_int(v), &t_non_negative_int());
    }
}

#[test]
fn non_negative_int_does_not_contain_negatives() {
    for v in [-1i64, -10, -100] {
        assert_atomic_not_subtype(&t_lit_int(v), &t_non_negative_int());
    }
}

#[test]
fn negative_int_contains_strictly_negative_literals() {
    for v in -100..=-1i64 {
        assert_atomic_subtype(&t_lit_int(v), &t_negative_int());
    }
}

#[test]
fn negative_int_does_not_contain_zero() {
    assert_atomic_not_subtype(&t_lit_int(0), &t_negative_int());
}

#[test]
fn negative_int_does_not_contain_positives() {
    for v in [1i64, 10, 100] {
        assert_atomic_not_subtype(&t_lit_int(v), &t_negative_int());
    }
}

#[test]
fn non_positive_int_contains_zero_and_negatives() {
    for v in -100..=0i64 {
        assert_atomic_subtype(&t_lit_int(v), &t_non_positive_int());
    }
}

#[test]
fn non_positive_int_does_not_contain_positives() {
    for v in [1i64, 10, 100] {
        assert_atomic_not_subtype(&t_lit_int(v), &t_non_positive_int());
    }
}

#[test]
fn positive_in_int() {
    assert_atomic_subtype(&t_positive_int(), &t_int());
}

#[test]
fn negative_in_int() {
    assert_atomic_subtype(&t_negative_int(), &t_int());
}

#[test]
fn non_negative_in_int() {
    assert_atomic_subtype(&t_non_negative_int(), &t_int());
}

#[test]
fn non_positive_in_int() {
    assert_atomic_subtype(&t_non_positive_int(), &t_int());
}

#[test]
fn int_not_in_positive() {
    assert_atomic_not_subtype(&t_int(), &t_positive_int());
}

#[test]
fn int_not_in_negative() {
    assert_atomic_not_subtype(&t_int(), &t_negative_int());
}

#[test]
fn positive_in_non_negative() {
    assert_atomic_subtype(&t_positive_int(), &t_non_negative_int());
}

#[test]
fn non_negative_not_in_positive() {
    assert_atomic_not_subtype(&t_non_negative_int(), &t_positive_int());
}

#[test]
fn negative_in_non_positive() {
    assert_atomic_subtype(&t_negative_int(), &t_non_positive_int());
}

#[test]
fn non_positive_not_in_negative() {
    assert_atomic_not_subtype(&t_non_positive_int(), &t_negative_int());
}

#[test]
fn from_5_in_positive() {
    assert_atomic_subtype(&t_int_from(5), &t_positive_int());
}

#[test]
fn from_1_in_positive() {
    assert_atomic_subtype(&t_int_from(1), &t_positive_int());
}

#[test]
fn from_0_not_in_positive() {
    assert_atomic_not_subtype(&t_int_from(0), &t_positive_int());
}

#[test]
fn from_0_in_non_negative() {
    assert_atomic_subtype(&t_int_from(0), &t_non_negative_int());
}

#[test]
fn to_minus_1_in_negative() {
    assert_atomic_subtype(&t_int_to(-1), &t_negative_int());
}

#[test]
fn to_0_in_non_positive() {
    assert_atomic_subtype(&t_int_to(0), &t_non_positive_int());
}

#[test]
fn to_minus_1_in_non_positive() {
    assert_atomic_subtype(&t_int_to(-1), &t_non_positive_int());
}

#[test]
fn range_inside_range() {
    for ((a_lo, a_hi), (b_lo, b_hi)) in [((1i64, 5), (0i64, 10)), ((0, 10), (-100, 100)), ((50, 60), (0, 100))] {
        assert_atomic_subtype(&t_int_range(a_lo, a_hi), &t_int_range(b_lo, b_hi));
    }
}

#[test]
fn range_not_inside_smaller_range() {
    for ((a_lo, a_hi), (b_lo, b_hi)) in
        [((0i64, 15), (0i64, 10)), ((-1, 5), (0, 5)), ((0, 11), (0, 10)), ((-100, 100), (0, 50))]
    {
        assert_atomic_not_subtype(&t_int_range(a_lo, a_hi), &t_int_range(b_lo, b_hi));
    }
}

#[test]
fn equal_range_is_subtype() {
    for (lo, hi) in [(0i64, 10), (-50, 50), (-100, 100)] {
        assert_atomic_subtype(&t_int_range(lo, hi), &t_int_range(lo, hi));
    }
}

#[test]
fn lit_inside_range() {
    for v in 0..=10i64 {
        assert_atomic_subtype(&t_lit_int(v), &t_int_range(0, 10));
    }
}

#[test]
fn lit_not_inside_range_outside_bounds() {
    for v in [-1i64, 11, 100, -100] {
        assert_atomic_not_subtype(&t_lit_int(v), &t_int_range(0, 10));
    }
}

#[test]
fn range_in_int() {
    for (lo, hi) in [(0i64, 10), (-50, 50), (i64::MIN + 1, i64::MAX - 1)] {
        assert_atomic_subtype(&t_int_range(lo, hi), &t_int());
    }
}

#[test]
fn range_in_positive_when_lo_geq_1() {
    assert_atomic_subtype(&t_int_range(1, 10), &t_positive_int());
    assert_atomic_subtype(&t_int_range(5, 100), &t_positive_int());
}

#[test]
fn range_not_in_positive_when_lo_lt_1() {
    assert_atomic_not_subtype(&t_int_range(0, 10), &t_positive_int());
    assert_atomic_not_subtype(&t_int_range(-5, 5), &t_positive_int());
}

#[test]
fn range_in_non_negative_when_lo_geq_0() {
    assert_atomic_subtype(&t_int_range(0, 10), &t_non_negative_int());
    assert_atomic_subtype(&t_int_range(5, 100), &t_non_negative_int());
}

#[test]
fn range_not_in_non_negative_when_lo_lt_0() {
    assert_atomic_not_subtype(&t_int_range(-1, 10), &t_non_negative_int());
}

#[test]
fn range_in_negative_when_hi_leq_minus_1() {
    assert_atomic_subtype(&t_int_range(-100, -1), &t_negative_int());
    assert_atomic_subtype(&t_int_range(-50, -10), &t_negative_int());
}

#[test]
fn range_not_in_negative_when_hi_geq_0() {
    assert_atomic_not_subtype(&t_int_range(-10, 0), &t_negative_int());
}

#[test]
fn lit_in_unspec_lit() {
    for v in [-100i64, 0, 1, 100] {
        assert_atomic_subtype(&t_lit_int(v), &t_int_unspec_lit());
    }
}

#[test]
fn unspec_lit_in_int() {
    assert_atomic_subtype(&t_int_unspec_lit(), &t_int());
}

#[test]
fn int_not_in_unspec_lit() {
    assert_atomic_not_subtype(&t_int(), &t_int_unspec_lit());
}

#[test]
fn unspec_lit_not_in_specific_lit() {
    assert_atomic_not_subtype(&t_int_unspec_lit(), &t_lit_int(5));
}

#[test]
fn lit_zero_in_non_negative_and_non_positive() {
    assert_atomic_subtype(&t_lit_int(0), &t_non_negative_int());
    assert_atomic_subtype(&t_lit_int(0), &t_non_positive_int());
}

#[test]
fn lit_zero_not_in_positive_or_negative() {
    assert_atomic_not_subtype(&t_lit_int(0), &t_positive_int());
    assert_atomic_not_subtype(&t_lit_int(0), &t_negative_int());
}

#[test]
fn from_min_to_max_is_int() {
    let r = t_int_range(i64::MIN + 1, i64::MAX - 1);
    assert_atomic_subtype(&r, &t_int());
}

#[test]
fn lit_in_from() {
    for n in [5i64, 10, 100] {
        for v in n..=(n + 50) {
            assert_atomic_subtype(&t_lit_int(v), &t_int_from(n));
        }
    }
}

#[test]
fn lit_not_in_from_below() {
    for n in [5i64, 10] {
        for v in (n - 5)..n {
            assert_atomic_not_subtype(&t_lit_int(v), &t_int_from(n));
        }
    }
}

#[test]
fn lit_in_to() {
    for n in [-5i64, 0, 5] {
        for v in (n - 50)..=n {
            assert_atomic_subtype(&t_lit_int(v), &t_int_to(n));
        }
    }
}

#[test]
fn lit_not_in_to_above() {
    for n in [-5i64, 0, 5] {
        for v in (n + 1)..(n + 50) {
            assert_atomic_not_subtype(&t_lit_int(v), &t_int_to(n));
        }
    }
}
