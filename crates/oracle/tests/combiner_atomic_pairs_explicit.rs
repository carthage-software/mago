mod common;

use common::*;

use mago_oracle::ty::Atom;
use mago_oracle::ty::well_known;

#[track_caller]
fn check<'arena>(f: &mut Fixture<'_, 'arena>, input: Vec<Atom<'arena>>, expected: &[Atom<'arena>]) {
    let result = combine_default(f, input);
    let mut actual = result.clone();
    actual.sort_unstable();
    let mut expected_sorted = expected.to_vec();
    expected_sorted.sort_unstable();
    assert_eq!(actual, expected_sorted, "got {result:?}, expected {expected:?}");
}

#[test]
fn p_true_true() {
    fixture(|f| {
        check(f, vec![f.t_true(), f.t_true()], &[f.t_true()]);
    });
}
#[test]
fn p_true_false() {
    fixture(|f| {
        check(f, vec![f.t_true(), f.t_false()], &[f.t_bool()]);
    });
}
#[test]
fn p_true_bool() {
    fixture(|f| {
        check(f, vec![f.t_true(), f.t_bool()], &[f.t_bool()]);
    });
}
#[test]
fn p_false_true() {
    fixture(|f| {
        check(f, vec![f.t_false(), f.t_true()], &[f.t_bool()]);
    });
}
#[test]
fn p_false_false() {
    fixture(|f| {
        check(f, vec![f.t_false(), f.t_false()], &[f.t_false()]);
    });
}
#[test]
fn p_false_bool() {
    fixture(|f| {
        check(f, vec![f.t_false(), f.t_bool()], &[f.t_bool()]);
    });
}
#[test]
fn p_bool_true() {
    fixture(|f| {
        check(f, vec![f.t_bool(), f.t_true()], &[f.t_bool()]);
    });
}
#[test]
fn p_bool_false() {
    fixture(|f| {
        check(f, vec![f.t_bool(), f.t_false()], &[f.t_bool()]);
    });
}
#[test]
fn p_bool_bool() {
    fixture(|f| {
        check(f, vec![f.t_bool(), f.t_bool()], &[f.t_bool()]);
    });
}

#[test]
fn p_int_int() {
    fixture(|f| {
        check(f, vec![f.t_int(), f.t_int()], &[f.t_int()]);
    });
}
#[test]
fn p_int_lit_0() {
    fixture(|f| {
        check(f, vec![f.t_int(), f.t_lit_int(0)], &[f.t_int()]);
    });
}
#[test]
fn p_int_lit_1() {
    fixture(|f| {
        check(f, vec![f.t_int(), f.t_lit_int(1)], &[f.t_int()]);
    });
}
#[test]
fn p_int_lit_neg() {
    fixture(|f| {
        check(f, vec![f.t_int(), f.t_lit_int(-1)], &[f.t_int()]);
    });
}
#[test]
fn p_lit_0_int() {
    fixture(|f| {
        check(f, vec![f.t_lit_int(0), f.t_int()], &[f.t_int()]);
    });
}
#[test]
fn p_lit_0_lit_0() {
    fixture(|f| {
        check(f, vec![f.t_lit_int(0), f.t_lit_int(0)], &[f.t_lit_int(0)]);
    });
}
#[test]
fn p_lit_0_lit_1() {
    fixture(|f| {
        let zero_to_one = f.t_int_range(0, 1);
        check(f, vec![f.t_lit_int(0), f.t_lit_int(1)], &[zero_to_one]);
    });
}
#[test]
fn p_lit_neg_lit_pos() {
    fixture(|f| {
        check(f, vec![f.t_lit_int(-1), f.t_lit_int(1)], &[f.t_lit_int(-1), f.t_lit_int(1)]);
    });
}

#[test]
fn p_int_positive() {
    fixture(|f| {
        check(f, vec![f.t_int(), f.t_positive_int()], &[f.t_int()]);
    });
}
#[test]
fn p_int_negative() {
    fixture(|f| {
        check(f, vec![f.t_int(), f.t_negative_int()], &[f.t_int()]);
    });
}
#[test]
fn p_int_non_neg() {
    fixture(|f| {
        check(f, vec![f.t_int(), f.t_non_negative_int()], &[f.t_int()]);
    });
}
#[test]
fn p_int_non_pos() {
    fixture(|f| {
        check(f, vec![f.t_int(), f.t_non_positive_int()], &[f.t_int()]);
    });
}
#[test]
fn p_int_range() {
    fixture(|f| {
        let zero_to_ten = f.t_int_range(0, 10);
        check(f, vec![f.t_int(), zero_to_ten], &[f.t_int()]);
    });
}

#[test]
fn p_positive_negative() {
    fixture(|f| {
        check(f, vec![f.t_positive_int(), f.t_negative_int()], &[f.t_negative_int(), f.t_positive_int()]);
    });
}

#[test]
fn p_non_neg_non_pos() {
    fixture(|f| {
        check(f, vec![f.t_non_negative_int(), f.t_non_positive_int()], &[f.t_int()]);
    });
}
#[test]
fn p_pos_lit_0() {
    fixture(|f| {
        check(f, vec![f.t_positive_int(), f.t_lit_int(0)], &[f.t_non_negative_int()]);
    });
}
#[test]
fn p_neg_lit_0() {
    fixture(|f| {
        check(f, vec![f.t_negative_int(), f.t_lit_int(0)], &[f.t_non_positive_int()]);
    });
}
#[test]
fn p_pos_lit_neg_1() {
    fixture(|f| {
        check(f, vec![f.t_positive_int(), f.t_lit_int(-1)], &[f.t_lit_int(-1), f.t_positive_int()]);
    });
}
#[test]
fn p_range_overlap() {
    fixture(|f| {
        let zero_to_five = f.t_int_range(0, 5);
        let three_to_ten = f.t_int_range(3, 10);
        let zero_to_ten = f.t_int_range(0, 10);
        check(f, vec![zero_to_five, three_to_ten], &[zero_to_ten]);
    });
}
#[test]
fn p_range_adjacent() {
    fixture(|f| {
        let zero_to_five = f.t_int_range(0, 5);
        let six_to_ten = f.t_int_range(6, 10);
        let zero_to_ten = f.t_int_range(0, 10);
        check(f, vec![zero_to_five, six_to_ten], &[zero_to_ten]);
    });
}

#[test]
fn p_range_disjoint() {
    fixture(|f| {
        let zero_to_five = f.t_int_range(0, 5);
        let ten_to_fifteen = f.t_int_range(10, 15);
        check(f, vec![zero_to_five, ten_to_fifteen], &[zero_to_five, ten_to_fifteen]);
    });
}

#[test]
fn p_from_to_overlap() {
    fixture(|f| {
        let from_zero = f.t_int_from(0);
        let to_five = f.t_int_to(5);
        check(f, vec![from_zero, to_five], &[f.t_int()]);
    });
}
#[test]
fn p_from_to_adjacent() {
    fixture(|f| {
        let from_five = f.t_int_from(5);
        let to_four = f.t_int_to(4);
        check(f, vec![from_five, to_four], &[f.t_int()]);
    });
}

#[test]
fn p_from_to_disjoint() {
    fixture(|f| {
        let from_ten = f.t_int_from(10);
        let to_zero = f.t_int_to(0);
        check(f, vec![from_ten, to_zero], &[from_ten, to_zero]);
    });
}

#[test]
fn p_from_lit_extends() {
    fixture(|f| {
        let from_five = f.t_int_from(5);
        let from_four = f.t_int_from(4);
        check(f, vec![from_five, f.t_lit_int(4)], &[from_four]);
    });
}
#[test]
fn p_to_lit_extends() {
    fixture(|f| {
        let to_five = f.t_int_to(5);
        let to_six = f.t_int_to(6);
        check(f, vec![to_five, f.t_lit_int(6)], &[to_six]);
    });
}

#[test]
fn p_string_string() {
    fixture(|f| {
        check(f, vec![f.t_string(), f.t_string()], &[f.t_string()]);
    });
}
#[test]
fn p_string_lit_empty() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        check(f, vec![f.t_string(), empty], &[f.t_string()]);
    });
}
#[test]
fn p_string_lit_hi() {
    fixture(|f| {
        let hi = f.t_lit_string("hi");
        check(f, vec![f.t_string(), hi], &[f.t_string()]);
    });
}
#[test]
fn p_string_lit_0() {
    fixture(|f| {
        let zero = f.t_lit_string("0");
        check(f, vec![f.t_string(), zero], &[f.t_string()]);
    });
}
#[test]
fn p_lit_string_string() {
    fixture(|f| {
        let hi = f.t_lit_string("hi");
        check(f, vec![hi, f.t_string()], &[f.t_string()]);
    });
}
#[test]
fn p_lit_a_lit_b() {
    fixture(|f| {
        let letter_a = f.t_lit_string("a");
        let letter_b = f.t_lit_string("b");
        check(f, vec![letter_a, letter_b], &[letter_a, letter_b]);
    });
}
#[test]
fn p_lit_a_lit_a() {
    fixture(|f| {
        let letter_a = f.t_lit_string("a");
        check(f, vec![letter_a, letter_a], &[letter_a]);
    });
}
#[test]
fn p_lit_uppercase_kept() {
    fixture(|f| {
        let capitalized = f.t_lit_string("Hello");
        let lowercased = f.t_lit_string("hello");
        check(f, vec![capitalized, lowercased], &[capitalized, lowercased]);
    });
}

#[test]
fn p_string_non_empty() {
    fixture(|f| {
        check(f, vec![f.t_string(), f.t_non_empty_string()], &[f.t_string()]);
    });
}
#[test]
fn p_non_empty_string() {
    fixture(|f| {
        check(f, vec![f.t_non_empty_string(), f.t_string()], &[f.t_string()]);
    });
}
#[test]
fn p_non_empty_lit_hi() {
    fixture(|f| {
        let hi = f.t_lit_string("hi");
        check(f, vec![f.t_non_empty_string(), hi], &[f.t_non_empty_string()]);
    });
}
#[test]
fn p_non_empty_lit_0() {
    fixture(|f| {
        let zero = f.t_lit_string("0");
        check(f, vec![f.t_non_empty_string(), zero], &[f.t_non_empty_string()]);
    });
}
#[test]
fn p_non_empty_lit_empty() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        check(f, vec![f.t_non_empty_string(), empty], &[f.t_non_empty_string(), empty]);
    });
}

#[test]
fn p_lit_empty_non_empty() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        check(f, vec![empty, f.t_non_empty_string()], &[f.t_string()]);
    });
}
#[test]
fn p_numeric_string() {
    fixture(|f| {
        check(f, vec![f.t_numeric_string(), f.t_string()], &[f.t_string()]);
    });
}
#[test]
fn p_numeric_lit_123() {
    fixture(|f| {
        let digits = f.t_lit_string("123");
        check(f, vec![f.t_numeric_string(), digits], &[f.t_numeric_string()]);
    });
}
#[test]
fn p_numeric_lit_abc() {
    fixture(|f| {
        let letters = f.t_lit_string("abc");
        check(f, vec![f.t_numeric_string(), letters], &[f.t_numeric_string(), letters]);
    });
}
#[test]
fn p_lower_lit_hi_lower() {
    fixture(|f| {
        let hi = f.t_lit_string("hi");
        check(f, vec![f.t_lower_string(), hi], &[f.t_lower_string()]);
    });
}
#[test]
fn p_lower_lit_hi_upper() {
    fixture(|f| {
        let hi_upper = f.t_lit_string("HI");
        check(f, vec![f.t_lower_string(), hi_upper], &[f.t_lower_string(), hi_upper]);
    });
}
#[test]
fn p_upper_lit_hi_upper() {
    fixture(|f| {
        let hi_upper = f.t_lit_string("HI");
        check(f, vec![f.t_upper_string(), hi_upper], &[f.t_upper_string()]);
    });
}
#[test]
fn p_upper_lit_hi_lower() {
    fixture(|f| {
        let hi = f.t_lit_string("hi");
        check(f, vec![f.t_upper_string(), hi], &[hi, f.t_upper_string()]);
    });
}
#[test]
fn p_truthy_lit_hi_lower() {
    fixture(|f| {
        let hi = f.t_lit_string("hi");
        check(f, vec![f.t_truthy_string(), hi], &[f.t_truthy_string()]);
    });
}
#[test]
fn p_truthy_lit_0() {
    fixture(|f| {
        let zero = f.t_lit_string("0");
        check(f, vec![f.t_truthy_string(), zero], &[zero, f.t_truthy_string()]);
    });
}
#[test]
fn p_truthy_lit_empty() {
    fixture(|f| {
        let empty = f.t_lit_string("");
        check(f, vec![f.t_truthy_string(), empty], &[empty, f.t_truthy_string()]);
    });
}
#[test]
fn p_lower_upper() {
    fixture(|f| {
        check(f, vec![f.t_lower_string(), f.t_upper_string()], &[f.t_string()]);
    });
}
#[test]
fn p_non_empty_truthy() {
    fixture(|f| {
        check(f, vec![f.t_non_empty_string(), f.t_truthy_string()], &[f.t_non_empty_string()]);
    });
}
#[test]
fn p_truthy_non_empty() {
    fixture(|f| {
        check(f, vec![f.t_truthy_string(), f.t_non_empty_string()], &[f.t_non_empty_string()]);
    });
}
#[test]
fn p_non_empty_lower() {
    fixture(|f| {
        check(f, vec![f.t_non_empty_string(), f.t_lower_string()], &[f.t_string()]);
    });
}

#[test]
fn p_float_float() {
    fixture(|f| {
        check(f, vec![f.t_float(), f.t_float()], &[f.t_float()]);
    });
}
#[test]
fn p_float_lit() {
    fixture(|f| {
        check(f, vec![f.t_float(), f.t_lit_float(1.5)], &[f.t_float()]);
    });
}
#[test]
fn p_lit_float() {
    fixture(|f| {
        check(f, vec![f.t_lit_float(1.5), f.t_float()], &[f.t_float()]);
    });
}
#[test]
fn p_lit_lit_float() {
    fixture(|f| {
        check(f, vec![f.t_lit_float(1.5), f.t_lit_float(1.5)], &[f.t_lit_float(1.5)]);
    });
}
#[test]
fn p_lit_lit_float_distinct() {
    fixture(|f| {
        check(f, vec![f.t_lit_float(1.0), f.t_lit_float(2.0)], &[f.t_lit_float(1.0), f.t_lit_float(2.0)]);
    });
}

#[test]
fn p_int_string() {
    fixture(|f| {
        check(f, vec![f.t_int(), f.t_string()], &[f.t_int(), f.t_string()]);
    });
}
#[test]
fn p_int_float() {
    fixture(|f| {
        check(f, vec![f.t_int(), f.t_float()], &[f.t_float(), f.t_int()]);
    });
}
#[test]
fn p_int_bool() {
    fixture(|f| {
        check(f, vec![f.t_int(), f.t_bool()], &[f.t_bool(), f.t_int()]);
    });
}
#[test]
fn p_string_float() {
    fixture(|f| {
        check(f, vec![f.t_string(), f.t_float()], &[f.t_float(), f.t_string()]);
    });
}
#[test]
fn p_string_bool() {
    fixture(|f| {
        check(f, vec![f.t_string(), f.t_bool()], &[f.t_bool(), f.t_string()]);
    });
}
#[test]
fn p_float_bool() {
    fixture(|f| {
        check(f, vec![f.t_float(), f.t_bool()], &[f.t_bool(), f.t_float()]);
    });
}

#[test]
fn p_numeric_int() {
    fixture(|f| {
        check(f, vec![f.t_numeric(), f.t_int()], &[f.t_numeric()]);
    });
}
#[test]
fn p_int_numeric() {
    fixture(|f| {
        check(f, vec![f.t_int(), f.t_numeric()], &[f.t_numeric()]);
    });
}
#[test]
fn p_numeric_float() {
    fixture(|f| {
        check(f, vec![f.t_numeric(), f.t_float()], &[f.t_numeric()]);
    });
}
#[test]
fn p_float_numeric() {
    fixture(|f| {
        check(f, vec![f.t_float(), f.t_numeric()], &[f.t_numeric()]);
    });
}
#[test]
fn p_numeric_lit_int() {
    fixture(|f| {
        check(f, vec![f.t_numeric(), f.t_lit_int(5)], &[f.t_numeric()]);
    });
}
#[test]
fn p_lit_int_numeric() {
    fixture(|f| {
        check(f, vec![f.t_lit_int(5), f.t_numeric()], &[f.t_numeric()]);
    });
}

#[test]
fn p_ak_int() {
    fixture(|f| {
        check(f, vec![f.t_array_key(), f.t_int()], &[f.t_array_key()]);
    });
}
#[test]
fn p_int_ak() {
    fixture(|f| {
        check(f, vec![f.t_int(), f.t_array_key()], &[f.t_array_key()]);
    });
}
#[test]
fn p_ak_string() {
    fixture(|f| {
        check(f, vec![f.t_array_key(), f.t_string()], &[f.t_array_key()]);
    });
}
#[test]
fn p_string_ak() {
    fixture(|f| {
        check(f, vec![f.t_string(), f.t_array_key()], &[f.t_array_key()]);
    });
}
#[test]
fn p_ak_float() {
    fixture(|f| {
        check(f, vec![f.t_array_key(), f.t_float()], &[f.t_array_key(), f.t_float()]);
    });
}
#[test]
fn p_ak_bool() {
    fixture(|f| {
        check(f, vec![f.t_array_key(), f.t_bool()], &[f.t_array_key(), f.t_bool()]);
    });
}
#[test]
fn p_ak_null() {
    fixture(|f| {
        check(f, vec![f.t_array_key(), f.null()], &[f.t_array_key(), f.null()]);
    });
}

#[test]
fn p_scalar_int() {
    fixture(|f| {
        check(f, vec![f.t_scalar(), f.t_int()], &[f.t_scalar()]);
    });
}
#[test]
fn p_int_scalar() {
    fixture(|f| {
        check(f, vec![f.t_int(), f.t_scalar()], &[f.t_scalar()]);
    });
}
#[test]
fn p_scalar_string() {
    fixture(|f| {
        check(f, vec![f.t_scalar(), f.t_string()], &[f.t_scalar()]);
    });
}
#[test]
fn p_string_scalar() {
    fixture(|f| {
        check(f, vec![f.t_string(), f.t_scalar()], &[f.t_scalar()]);
    });
}
#[test]
fn p_scalar_float() {
    fixture(|f| {
        check(f, vec![f.t_scalar(), f.t_float()], &[f.t_scalar()]);
    });
}
#[test]
fn p_float_scalar() {
    fixture(|f| {
        check(f, vec![f.t_float(), f.t_scalar()], &[f.t_scalar()]);
    });
}
#[test]
fn p_scalar_numeric() {
    fixture(|f| {
        check(f, vec![f.t_scalar(), f.t_numeric()], &[f.t_scalar()]);
    });
}
#[test]
fn p_numeric_scalar() {
    fixture(|f| {
        check(f, vec![f.t_numeric(), f.t_scalar()], &[f.t_scalar()]);
    });
}
#[test]
fn p_scalar_ak() {
    fixture(|f| {
        check(f, vec![f.t_scalar(), f.t_array_key()], &[f.t_scalar()]);
    });
}
#[test]
fn p_ak_scalar() {
    fixture(|f| {
        check(f, vec![f.t_array_key(), f.t_scalar()], &[f.t_scalar()]);
    });
}
#[test]
fn p_scalar_bool() {
    fixture(|f| {
        check(f, vec![f.t_scalar(), f.t_bool()], &[f.t_scalar()]);
    });
}
#[test]
fn p_bool_scalar() {
    fixture(|f| {
        check(f, vec![f.t_bool(), f.t_scalar()], &[f.t_scalar()]);
    });
}
#[test]
fn p_scalar_true() {
    fixture(|f| {
        check(f, vec![f.t_scalar(), f.t_true()], &[f.t_scalar()]);
    });
}
#[test]
fn p_true_scalar() {
    fixture(|f| {
        check(f, vec![f.t_true(), f.t_scalar()], &[f.t_scalar()]);
    });
}
#[test]
fn p_scalar_false() {
    fixture(|f| {
        check(f, vec![f.t_scalar(), f.t_false()], &[f.t_scalar()]);
    });
}
#[test]
fn p_false_scalar() {
    fixture(|f| {
        check(f, vec![f.t_false(), f.t_scalar()], &[f.t_scalar()]);
    });
}

#[test]
fn p_null_null() {
    fixture(|f| {
        check(f, vec![f.null(), f.null()], &[f.null()]);
    });
}
#[test]
fn p_void_void() {
    fixture(|f| {
        check(f, vec![f.void(), f.void()], &[f.void()]);
    });
}
#[test]
fn p_never_never() {
    fixture(|f| {
        check(f, vec![f.never(), f.never()], &[f.never()]);
    });
}
#[test]
fn p_null_void() {
    fixture(|f| {
        check(f, vec![f.null(), f.void()], &[f.null()]);
    });
}
#[test]
fn p_void_null() {
    fixture(|f| {
        check(f, vec![f.void(), f.null()], &[f.null()]);
    });
}
#[test]
fn p_null_never() {
    fixture(|f| {
        check(f, vec![f.null(), f.never()], &[f.null()]);
    });
}
#[test]
fn p_never_null() {
    fixture(|f| {
        check(f, vec![f.never(), f.null()], &[f.null()]);
    });
}
#[test]
fn p_void_never() {
    fixture(|f| {
        check(f, vec![f.void(), f.never()], &[f.void()]);
    });
}
#[test]
fn p_never_void() {
    fixture(|f| {
        check(f, vec![f.never(), f.void()], &[f.void()]);
    });
}
#[test]
fn p_null_int() {
    fixture(|f| {
        check(f, vec![f.null(), f.t_int()], &[f.t_int(), f.null()]);
    });
}
#[test]
fn p_void_int() {
    fixture(|f| {
        check(f, vec![f.void(), f.t_int()], &[f.t_int(), f.null()]);
    });
}
#[test]
fn p_never_int() {
    fixture(|f| {
        check(f, vec![f.never(), f.t_int()], &[f.t_int()]);
    });
}
#[test]
fn p_null_object() {
    fixture(|f| {
        check(f, vec![f.null(), f.t_object_any()], &[f.null(), f.t_object_any()]);
    });
}
#[test]
fn p_void_object() {
    fixture(|f| {
        check(f, vec![f.void(), f.t_object_any()], &[f.t_object_any(), f.null()]);
    });
}
#[test]
fn p_never_object() {
    fixture(|f| {
        check(f, vec![f.never(), f.t_object_any()], &[f.t_object_any()]);
    });
}
#[test]
fn p_null_resource() {
    fixture(|f| {
        check(f, vec![f.null(), f.t_resource()], &[f.null(), f.t_resource()]);
    });
}
#[test]
fn p_void_resource() {
    fixture(|f| {
        check(f, vec![f.void(), f.t_resource()], &[f.t_resource(), f.null()]);
    });
}
#[test]
fn p_never_resource() {
    fixture(|f| {
        check(f, vec![f.never(), f.t_resource()], &[f.t_resource()]);
    });
}

#[test]
fn p_mixed_int() {
    fixture(|f| {
        check(f, vec![f.mixed(), f.t_int()], &[f.mixed()]);
    });
}
#[test]
fn p_int_mixed() {
    fixture(|f| {
        check(f, vec![f.t_int(), f.mixed()], &[f.mixed()]);
    });
}
#[test]
fn p_mixed_string() {
    fixture(|f| {
        check(f, vec![f.mixed(), f.t_string()], &[f.mixed()]);
    });
}
#[test]
fn p_mixed_object() {
    fixture(|f| {
        check(f, vec![f.mixed(), f.t_object_any()], &[f.mixed()]);
    });
}
#[test]
fn p_mixed_array() {
    fixture(|f| {
        check(f, vec![f.mixed(), f.t_empty_array()], &[f.mixed()]);
    });
}
#[test]
fn p_mixed_null() {
    fixture(|f| {
        check(f, vec![f.mixed(), f.null()], &[f.mixed()]);
    });
}
#[test]
fn p_mixed_void() {
    fixture(|f| {
        check(f, vec![f.mixed(), f.void()], &[f.mixed()]);
    });
}
#[test]
fn p_mixed_never() {
    fixture(|f| {
        check(f, vec![f.mixed(), f.never()], &[f.mixed()]);
    });
}
#[test]
fn p_mixed_resource() {
    fixture(|f| {
        check(f, vec![f.mixed(), f.t_resource()], &[f.mixed()]);
    });
}
#[test]
fn p_mixed_mixed() {
    fixture(|f| {
        check(f, vec![f.mixed(), f.mixed()], &[f.mixed()]);
    });
}

#[test]
fn p_cs_cs() {
    fixture(|f| {
        check(f, vec![f.t_class_string(), f.t_class_string()], &[f.t_class_string()]);
    });
}
#[test]
fn p_is_is() {
    fixture(|f| {
        check(f, vec![f.t_interface_string(), f.t_interface_string()], &[f.t_interface_string()]);
    });
}
#[test]
fn p_es_es() {
    fixture(|f| {
        check(f, vec![f.t_enum_string(), f.t_enum_string()], &[f.t_enum_string()]);
    });
}
#[test]
fn p_ts_ts() {
    fixture(|f| {
        check(f, vec![f.t_trait_string(), f.t_trait_string()], &[f.t_trait_string()]);
    });
}
#[test]
fn p_cs_is() {
    fixture(|f| {
        check(f, vec![f.t_class_string(), f.t_interface_string()], &[f.t_class_string(), f.t_interface_string()]);
    });
}
#[test]
fn p_cs_es() {
    fixture(|f| {
        check(f, vec![f.t_class_string(), f.t_enum_string()], &[f.t_class_string(), f.t_enum_string()]);
    });
}
#[test]
fn p_cs_string() {
    fixture(|f| {
        check(f, vec![f.t_class_string(), f.t_string()], &[f.t_string()]);
    });
}
#[test]
fn p_string_cs() {
    fixture(|f| {
        check(f, vec![f.t_string(), f.t_class_string()], &[f.t_string()]);
    });
}
#[test]
fn p_cs_ak() {
    fixture(|f| {
        check(f, vec![f.t_class_string(), f.t_array_key()], &[f.t_array_key()]);
    });
}
#[test]
fn p_cs_scalar() {
    fixture(|f| {
        check(f, vec![f.t_class_string(), f.t_scalar()], &[f.t_scalar()]);
    });
}
#[test]
fn p_cs_int() {
    fixture(|f| {
        check(f, vec![f.t_class_string(), f.t_int()], &[f.t_class_string(), f.t_int()]);
    });
}
#[test]
fn p_cs_null() {
    fixture(|f| {
        check(f, vec![f.t_class_string(), f.null()], &[f.t_class_string(), f.null()]);
    });
}
#[test]
fn p_cs_never() {
    fixture(|f| {
        check(f, vec![f.t_class_string(), f.never()], &[f.t_class_string()]);
    });
}

#[test]
fn p_res_res() {
    fixture(|f| {
        check(f, vec![f.t_resource(), f.t_resource()], &[f.t_resource()]);
    });
}
#[test]
fn p_open_open() {
    fixture(|f| {
        check(f, vec![f.t_open_resource(), f.t_open_resource()], &[f.t_open_resource()]);
    });
}
#[test]
fn p_closed_closed() {
    fixture(|f| {
        check(f, vec![f.t_closed_resource(), f.t_closed_resource()], &[f.t_closed_resource()]);
    });
}
#[test]
fn p_open_closed() {
    fixture(|f| {
        check(f, vec![f.t_open_resource(), f.t_closed_resource()], &[f.t_resource()]);
    });
}
#[test]
fn p_closed_open() {
    fixture(|f| {
        check(f, vec![f.t_closed_resource(), f.t_open_resource()], &[f.t_resource()]);
    });
}
#[test]
fn p_res_open() {
    fixture(|f| {
        check(f, vec![f.t_resource(), f.t_open_resource()], &[f.t_resource()]);
    });
}
#[test]
fn p_res_closed() {
    fixture(|f| {
        check(f, vec![f.t_resource(), f.t_closed_resource()], &[f.t_resource()]);
    });
}
#[test]
fn p_open_res() {
    fixture(|f| {
        check(f, vec![f.t_open_resource(), f.t_resource()], &[f.t_resource()]);
    });
}
#[test]
fn p_closed_res() {
    fixture(|f| {
        check(f, vec![f.t_closed_resource(), f.t_resource()], &[f.t_resource()]);
    });
}

#[test]
fn p_obj_obj() {
    fixture(|f| {
        check(f, vec![f.t_object_any(), f.t_object_any()], &[f.t_object_any()]);
    });
}
#[test]
fn p_obj_foo() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        check(f, vec![f.t_object_any(), foo], &[f.t_object_any()]);
    });
}
#[test]
fn p_foo_obj() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        check(f, vec![foo, f.t_object_any()], &[f.t_object_any()]);
    });
}
#[test]
fn p_foo_foo() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        check(f, vec![foo, foo], &[foo]);
    });
}
#[test]
fn p_foo_bar() {
    fixture(|f| {
        let foo = f.t_named("Foo");
        let bar = f.t_named("Bar");
        check(f, vec![foo, bar], &[bar, foo]);
    });
}
#[test]
fn p_e_e() {
    fixture(|f| {
        let enum_e = f.t_enum("E");
        check(f, vec![enum_e, enum_e], &[enum_e]);
    });
}
#[test]
fn p_e_f() {
    fixture(|f| {
        let enum_e = f.t_enum("E");
        let enum_f = f.t_enum("F");
        check(f, vec![enum_e, enum_f], &[enum_e, enum_f]);
    });
}
#[test]
fn p_e_ea() {
    fixture(|f| {
        let enum_e = f.t_enum("E");
        let case_a = f.t_enum_case("E", "A");
        check(f, vec![enum_e, case_a], &[enum_e]);
    });
}
#[test]
fn p_ea_ea() {
    fixture(|f| {
        let case_a = f.t_enum_case("E", "A");
        check(f, vec![case_a, case_a], &[case_a]);
    });
}
#[test]
fn p_ea_eb() {
    fixture(|f| {
        let case_a = f.t_enum_case("E", "A");
        let case_b = f.t_enum_case("E", "B");
        check(f, vec![case_a, case_b], &[case_a, case_b]);
    });
}

#[test]
fn p_arr_empty() {
    fixture(|f| {
        check(f, vec![f.t_empty_array()], &[f.t_empty_array()]);
    });
}
#[test]
fn p_arr_empty_empty() {
    fixture(|f| {
        check(f, vec![f.t_empty_array(), f.t_empty_array()], &[f.t_empty_array()]);
    });
}
#[test]
fn p_arr_list_int() {
    fixture(|f| {
        let list_of_int = f.t_list(well_known::TYPE_INT, false);
        check(f, vec![list_of_int], &[list_of_int]);
    });
}
#[test]
fn p_arr_list_int_x2() {
    fixture(|f| {
        let list_of_int = f.t_list(well_known::TYPE_INT, false);
        check(f, vec![list_of_int, list_of_int], &[list_of_int]);
    });
}
#[test]
fn p_arr_list_int_string() {
    fixture(|f| {
        let int_or_string = f.u_many(vec![well_known::INT, well_known::STRING]);
        let list_of_int = f.t_list(well_known::TYPE_INT, false);
        let list_of_string = f.t_list(well_known::TYPE_STRING, false);
        let list_of_int_or_string = f.t_list(int_or_string, false);
        check(f, vec![list_of_int, list_of_string], &[list_of_int_or_string]);
    });
}
#[test]
fn p_arr_ne_list_int() {
    fixture(|f| {
        let non_empty_list = f.t_list(well_known::TYPE_INT, true);
        check(f, vec![non_empty_list], &[non_empty_list]);
    });
}
#[test]
fn p_arr_ne_list_x2() {
    fixture(|f| {
        let non_empty_list = f.t_list(well_known::TYPE_INT, true);
        check(f, vec![non_empty_list, non_empty_list], &[non_empty_list]);
    });
}
#[test]
fn p_arr_ne_with_e() {
    fixture(|f| {
        let non_empty_list = f.t_list(well_known::TYPE_INT, true);
        let list_of_int = f.t_list(well_known::TYPE_INT, false);
        check(f, vec![non_empty_list, list_of_int], &[list_of_int]);
    });
}
#[test]
fn p_arr_empty_with_list() {
    fixture(|f| {
        let list_of_int = f.t_list(well_known::TYPE_INT, false);
        check(f, vec![f.t_empty_array(), list_of_int], &[list_of_int]);
    });
}
#[test]
fn p_arr_list_with_empty() {
    fixture(|f| {
        let list_of_int = f.t_list(well_known::TYPE_INT, false);
        check(f, vec![list_of_int, f.t_empty_array()], &[list_of_int]);
    });
}
