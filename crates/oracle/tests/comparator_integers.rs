mod common;

use common::*;

#[test]
fn int_reflexive() {
    fixture(|f| {
        let int = f.t_int();
        assert_atomic_subtype(f, int, int);
    });
}

#[test]
fn lit_int_reflexive_for_many_values() {
    fixture(|f| {
        for v in -50..=50i64 {
            let literal = f.t_lit_int(v);
            assert_atomic_subtype(f, literal, literal);
        }
    });
}

#[test]
fn int_contains_every_literal() {
    fixture(|f| {
        let int = f.t_int();
        for v in -200..=200i64 {
            let literal = f.t_lit_int(v);
            assert_atomic_subtype(f, literal, int);
        }
    });
}

#[test]
fn int_does_not_contain_specific_literal() {
    fixture(|f| {
        let int = f.t_int();
        for v in [-100i64, 0, 100] {
            let literal = f.t_lit_int(v);
            assert_atomic_not_subtype(f, int, literal);
        }
    });
}

#[test]
fn distinct_literals_are_disjoint() {
    fixture(|f| {
        for a in -10..=10i64 {
            for b in -10..=10i64 {
                if a == b {
                    continue;
                }
                let left = f.t_lit_int(a);
                let right = f.t_lit_int(b);
                assert_atomic_not_subtype(f, left, right);
            }
        }
    });
}

#[test]
fn positive_int_contains_strictly_positive_literals() {
    fixture(|f| {
        let positive = f.t_positive_int();
        for v in 1..=100i64 {
            let literal = f.t_lit_int(v);
            assert_atomic_subtype(f, literal, positive);
        }
    });
}

#[test]
fn positive_int_does_not_contain_zero() {
    fixture(|f| {
        let zero = f.t_lit_int(0);
        let positive = f.t_positive_int();
        assert_atomic_not_subtype(f, zero, positive);
    });
}

#[test]
fn positive_int_does_not_contain_negatives() {
    fixture(|f| {
        let positive = f.t_positive_int();
        for v in [-1i64, -10, -100] {
            let literal = f.t_lit_int(v);
            assert_atomic_not_subtype(f, literal, positive);
        }
    });
}

#[test]
fn non_negative_int_contains_zero_and_positives() {
    fixture(|f| {
        let non_negative = f.t_non_negative_int();
        for v in 0..=100i64 {
            let literal = f.t_lit_int(v);
            assert_atomic_subtype(f, literal, non_negative);
        }
    });
}

#[test]
fn non_negative_int_does_not_contain_negatives() {
    fixture(|f| {
        let non_negative = f.t_non_negative_int();
        for v in [-1i64, -10, -100] {
            let literal = f.t_lit_int(v);
            assert_atomic_not_subtype(f, literal, non_negative);
        }
    });
}

#[test]
fn negative_int_contains_strictly_negative_literals() {
    fixture(|f| {
        let negative = f.t_negative_int();
        for v in -100..=-1i64 {
            let literal = f.t_lit_int(v);
            assert_atomic_subtype(f, literal, negative);
        }
    });
}

#[test]
fn negative_int_does_not_contain_zero() {
    fixture(|f| {
        let zero = f.t_lit_int(0);
        let negative = f.t_negative_int();
        assert_atomic_not_subtype(f, zero, negative);
    });
}

#[test]
fn negative_int_does_not_contain_positives() {
    fixture(|f| {
        let negative = f.t_negative_int();
        for v in [1i64, 10, 100] {
            let literal = f.t_lit_int(v);
            assert_atomic_not_subtype(f, literal, negative);
        }
    });
}

#[test]
fn non_positive_int_contains_zero_and_negatives() {
    fixture(|f| {
        let non_positive = f.t_non_positive_int();
        for v in -100..=0i64 {
            let literal = f.t_lit_int(v);
            assert_atomic_subtype(f, literal, non_positive);
        }
    });
}

#[test]
fn non_positive_int_does_not_contain_positives() {
    fixture(|f| {
        let non_positive = f.t_non_positive_int();
        for v in [1i64, 10, 100] {
            let literal = f.t_lit_int(v);
            assert_atomic_not_subtype(f, literal, non_positive);
        }
    });
}

#[test]
fn positive_in_int() {
    fixture(|f| {
        let positive = f.t_positive_int();
        let int = f.t_int();
        assert_atomic_subtype(f, positive, int);
    });
}

#[test]
fn negative_in_int() {
    fixture(|f| {
        let negative = f.t_negative_int();
        let int = f.t_int();
        assert_atomic_subtype(f, negative, int);
    });
}

#[test]
fn non_negative_in_int() {
    fixture(|f| {
        let non_negative = f.t_non_negative_int();
        let int = f.t_int();
        assert_atomic_subtype(f, non_negative, int);
    });
}

#[test]
fn non_positive_in_int() {
    fixture(|f| {
        let non_positive = f.t_non_positive_int();
        let int = f.t_int();
        assert_atomic_subtype(f, non_positive, int);
    });
}

#[test]
fn int_not_in_positive() {
    fixture(|f| {
        let int = f.t_int();
        let positive = f.t_positive_int();
        assert_atomic_not_subtype(f, int, positive);
    });
}

#[test]
fn int_not_in_negative() {
    fixture(|f| {
        let int = f.t_int();
        let negative = f.t_negative_int();
        assert_atomic_not_subtype(f, int, negative);
    });
}

#[test]
fn positive_in_non_negative() {
    fixture(|f| {
        let positive = f.t_positive_int();
        let non_negative = f.t_non_negative_int();
        assert_atomic_subtype(f, positive, non_negative);
    });
}

#[test]
fn non_negative_not_in_positive() {
    fixture(|f| {
        let non_negative = f.t_non_negative_int();
        let positive = f.t_positive_int();
        assert_atomic_not_subtype(f, non_negative, positive);
    });
}

#[test]
fn negative_in_non_positive() {
    fixture(|f| {
        let negative = f.t_negative_int();
        let non_positive = f.t_non_positive_int();
        assert_atomic_subtype(f, negative, non_positive);
    });
}

#[test]
fn non_positive_not_in_negative() {
    fixture(|f| {
        let non_positive = f.t_non_positive_int();
        let negative = f.t_negative_int();
        assert_atomic_not_subtype(f, non_positive, negative);
    });
}

#[test]
fn from_5_in_positive() {
    fixture(|f| {
        let from = f.t_int_from(5);
        let positive = f.t_positive_int();
        assert_atomic_subtype(f, from, positive);
    });
}

#[test]
fn from_1_in_positive() {
    fixture(|f| {
        let from = f.t_int_from(1);
        let positive = f.t_positive_int();
        assert_atomic_subtype(f, from, positive);
    });
}

#[test]
fn from_0_not_in_positive() {
    fixture(|f| {
        let from = f.t_int_from(0);
        let positive = f.t_positive_int();
        assert_atomic_not_subtype(f, from, positive);
    });
}

#[test]
fn from_0_in_non_negative() {
    fixture(|f| {
        let from = f.t_int_from(0);
        let non_negative = f.t_non_negative_int();
        assert_atomic_subtype(f, from, non_negative);
    });
}

#[test]
fn to_minus_1_in_negative() {
    fixture(|f| {
        let to = f.t_int_to(-1);
        let negative = f.t_negative_int();
        assert_atomic_subtype(f, to, negative);
    });
}

#[test]
fn to_0_in_non_positive() {
    fixture(|f| {
        let to = f.t_int_to(0);
        let non_positive = f.t_non_positive_int();
        assert_atomic_subtype(f, to, non_positive);
    });
}

#[test]
fn to_minus_1_in_non_positive() {
    fixture(|f| {
        let to = f.t_int_to(-1);
        let non_positive = f.t_non_positive_int();
        assert_atomic_subtype(f, to, non_positive);
    });
}

#[test]
fn range_inside_range() {
    fixture(|f| {
        for ((a_lo, a_hi), (b_lo, b_hi)) in [((1i64, 5), (0i64, 10)), ((0, 10), (-100, 100)), ((50, 60), (0, 100))] {
            let inner = f.t_int_range(a_lo, a_hi);
            let outer = f.t_int_range(b_lo, b_hi);
            assert_atomic_subtype(f, inner, outer);
        }
    });
}

#[test]
fn range_not_inside_smaller_range() {
    fixture(|f| {
        for ((a_lo, a_hi), (b_lo, b_hi)) in
            [((0i64, 15), (0i64, 10)), ((-1, 5), (0, 5)), ((0, 11), (0, 10)), ((-100, 100), (0, 50))]
        {
            let inner = f.t_int_range(a_lo, a_hi);
            let outer = f.t_int_range(b_lo, b_hi);
            assert_atomic_not_subtype(f, inner, outer);
        }
    });
}

#[test]
fn equal_range_is_subtype() {
    fixture(|f| {
        for (lo, hi) in [(0i64, 10), (-50, 50), (-100, 100)] {
            let range = f.t_int_range(lo, hi);
            assert_atomic_subtype(f, range, range);
        }
    });
}

#[test]
fn lit_inside_range() {
    fixture(|f| {
        let range = f.t_int_range(0, 10);
        for v in 0..=10i64 {
            let literal = f.t_lit_int(v);
            assert_atomic_subtype(f, literal, range);
        }
    });
}

#[test]
fn lit_not_inside_range_outside_bounds() {
    fixture(|f| {
        let range = f.t_int_range(0, 10);
        for v in [-1i64, 11, 100, -100] {
            let literal = f.t_lit_int(v);
            assert_atomic_not_subtype(f, literal, range);
        }
    });
}

#[test]
fn range_in_int() {
    fixture(|f| {
        let int = f.t_int();
        for (lo, hi) in [(0i64, 10), (-50, 50), (i64::MIN + 1, i64::MAX - 1)] {
            let range = f.t_int_range(lo, hi);
            assert_atomic_subtype(f, range, int);
        }
    });
}

#[test]
fn range_in_positive_when_lo_geq_1() {
    fixture(|f| {
        let positive = f.t_positive_int();
        let one_to_ten = f.t_int_range(1, 10);
        let five_to_hundred = f.t_int_range(5, 100);
        assert_atomic_subtype(f, one_to_ten, positive);
        assert_atomic_subtype(f, five_to_hundred, positive);
    });
}

#[test]
fn range_not_in_positive_when_lo_lt_1() {
    fixture(|f| {
        let positive = f.t_positive_int();
        let zero_to_ten = f.t_int_range(0, 10);
        let minus_five_to_five = f.t_int_range(-5, 5);
        assert_atomic_not_subtype(f, zero_to_ten, positive);
        assert_atomic_not_subtype(f, minus_five_to_five, positive);
    });
}

#[test]
fn range_in_non_negative_when_lo_geq_0() {
    fixture(|f| {
        let non_negative = f.t_non_negative_int();
        let zero_to_ten = f.t_int_range(0, 10);
        let five_to_hundred = f.t_int_range(5, 100);
        assert_atomic_subtype(f, zero_to_ten, non_negative);
        assert_atomic_subtype(f, five_to_hundred, non_negative);
    });
}

#[test]
fn range_not_in_non_negative_when_lo_lt_0() {
    fixture(|f| {
        let non_negative = f.t_non_negative_int();
        let range = f.t_int_range(-1, 10);
        assert_atomic_not_subtype(f, range, non_negative);
    });
}

#[test]
fn range_in_negative_when_hi_leq_minus_1() {
    fixture(|f| {
        let negative = f.t_negative_int();
        let wide = f.t_int_range(-100, -1);
        let narrow = f.t_int_range(-50, -10);
        assert_atomic_subtype(f, wide, negative);
        assert_atomic_subtype(f, narrow, negative);
    });
}

#[test]
fn range_not_in_negative_when_hi_geq_0() {
    fixture(|f| {
        let negative = f.t_negative_int();
        let range = f.t_int_range(-10, 0);
        assert_atomic_not_subtype(f, range, negative);
    });
}

#[test]
fn lit_in_unspec_lit() {
    fixture(|f| {
        let unspecified = f.t_int_unspec_lit();
        for v in [-100i64, 0, 1, 100] {
            let literal = f.t_lit_int(v);
            assert_atomic_subtype(f, literal, unspecified);
        }
    });
}

#[test]
fn unspec_lit_in_int() {
    fixture(|f| {
        let unspecified = f.t_int_unspec_lit();
        let int = f.t_int();
        assert_atomic_subtype(f, unspecified, int);
    });
}

#[test]
fn int_not_in_unspec_lit() {
    fixture(|f| {
        let int = f.t_int();
        let unspecified = f.t_int_unspec_lit();
        assert_atomic_not_subtype(f, int, unspecified);
    });
}

#[test]
fn unspec_lit_not_in_specific_lit() {
    fixture(|f| {
        let unspecified = f.t_int_unspec_lit();
        let five = f.t_lit_int(5);
        assert_atomic_not_subtype(f, unspecified, five);
    });
}

#[test]
fn lit_zero_in_non_negative_and_non_positive() {
    fixture(|f| {
        let zero = f.t_lit_int(0);
        let non_negative = f.t_non_negative_int();
        let non_positive = f.t_non_positive_int();
        assert_atomic_subtype(f, zero, non_negative);
        assert_atomic_subtype(f, zero, non_positive);
    });
}

#[test]
fn lit_zero_not_in_positive_or_negative() {
    fixture(|f| {
        let zero = f.t_lit_int(0);
        let positive = f.t_positive_int();
        let negative = f.t_negative_int();
        assert_atomic_not_subtype(f, zero, positive);
        assert_atomic_not_subtype(f, zero, negative);
    });
}

#[test]
fn from_min_to_max_is_int() {
    fixture(|f| {
        let range = f.t_int_range(i64::MIN + 1, i64::MAX - 1);
        let int = f.t_int();
        assert_atomic_subtype(f, range, int);
    });
}

#[test]
fn lit_in_from() {
    fixture(|f| {
        for n in [5i64, 10, 100] {
            let from = f.t_int_from(n);
            for v in n..=(n + 50) {
                let literal = f.t_lit_int(v);
                assert_atomic_subtype(f, literal, from);
            }
        }
    });
}

#[test]
fn lit_not_in_from_below() {
    fixture(|f| {
        for n in [5i64, 10] {
            let from = f.t_int_from(n);
            for v in (n - 5)..n {
                let literal = f.t_lit_int(v);
                assert_atomic_not_subtype(f, literal, from);
            }
        }
    });
}

#[test]
fn lit_in_to() {
    fixture(|f| {
        for n in [-5i64, 0, 5] {
            let to = f.t_int_to(n);
            for v in (n - 50)..=n {
                let literal = f.t_lit_int(v);
                assert_atomic_subtype(f, literal, to);
            }
        }
    });
}

#[test]
fn lit_not_in_to_above() {
    fixture(|f| {
        for n in [-5i64, 0, 5] {
            let to = f.t_int_to(n);
            for v in (n + 1)..(n + 50) {
                let literal = f.t_lit_int(v);
                assert_atomic_not_subtype(f, literal, to);
            }
        }
    });
}
