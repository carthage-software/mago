use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::float::TFloat;
use mago_codex::ttype::comparator::union_comparator::can_expression_types_be_identical;
use mago_codex::ttype::union::TUnion;
use mago_php_version::PHPVersion;

use crate::utils::php_emulation::numeric_string_equals_int;

#[inline]
pub fn is_always_less_than_or_equal(lhs: &TUnion, rhs: &TUnion) -> bool {
    if let (Some(max_lhs), Some(min_rhs)) = (lhs.get_maximum_int_value(), rhs.get_minimum_int_value()) {
        return max_lhs <= min_rhs;
    }

    is_always_less_than(lhs, rhs) || is_always_identical_to(lhs, rhs)
}

#[inline]
pub fn is_always_greater_than_or_equal(lhs: &TUnion, rhs: &TUnion) -> bool {
    if let (Some(min_lhs), Some(max_rhs)) = (lhs.get_minimum_int_value(), rhs.get_maximum_int_value()) {
        return min_lhs >= max_rhs;
    }

    is_always_greater_than(lhs, rhs) || is_always_identical_to(lhs, rhs)
}

/// Checks if the left-hand side type is always strictly less than the right-hand side type.
/// Returns `false` if uncertain.
pub fn is_always_less_than(lhs: &TUnion, rhs: &TUnion) -> bool {
    if lhs.is_null() && !rhs.is_null() {
        return true;
    }

    if lhs.is_false() && rhs.is_true() {
        return true;
    }

    if lhs.is_false() && !rhs.is_null() && !rhs.is_false() {
        return true;
    }

    if let (Some(max_lhs), Some(min_rhs)) = (lhs.get_maximum_int_value(), rhs.get_minimum_int_value()) {
        return max_lhs < min_rhs;
    }

    if !lhs.is_single() || !rhs.is_single() {
        return false;
    }

    let lhs_atomic = lhs.get_single();
    let rhs_atomic = rhs.get_single();

    match (lhs_atomic, rhs_atomic) {
        (TAtomic::Scalar(TScalar::Float(l)), TAtomic::Scalar(TScalar::Float(r))) => match (l, r) {
            (TFloat::Literal(l_val), TFloat::Literal(r_val)) => return l_val < r_val,
            _ => return false,
        },
        _ => {}
    }

    false
}

/// Checks if the left-hand side type is always strictly greater than the right-hand side type.
/// Returns `false` if uncertain.
pub fn is_always_greater_than(lhs: &TUnion, rhs: &TUnion) -> bool {
    if !lhs.is_null() && rhs.is_null() {
        return true;
    }

    if lhs.is_true() && rhs.is_false() {
        return true;
    }

    if lhs.is_true() && !rhs.is_null() && !rhs.is_true() {
        return true;
    }

    if let (Some(min_lhs), Some(max_rhs)) = (lhs.get_minimum_int_value(), rhs.get_maximum_int_value()) {
        return min_lhs > max_rhs;
    }

    if !lhs.is_single() || !rhs.is_single() {
        return false;
    }

    let lhs_atomic = lhs.get_single();
    let rhs_atomic = rhs.get_single();

    match (lhs_atomic, rhs_atomic) {
        (TAtomic::Scalar(TScalar::Float(l)), TAtomic::Scalar(TScalar::Float(r))) => match (l, r) {
            (TFloat::Literal(l_val), TFloat::Literal(r_val)) => return l_val > r_val,
            _ => return false,
        },
        _ => {}
    }

    false
}

pub fn is_always_identical_to(lhs: &TUnion, rhs: &TUnion) -> bool {
    if lhs.is_null() && rhs.is_null() {
        return true;
    }

    if lhs.is_false() && rhs.is_false() {
        return true;
    }

    if lhs.is_true() && rhs.is_true() {
        return true;
    }

    if lhs.is_enum() && rhs.is_enum() {
        let left_cases = lhs.get_enum_cases();
        let right_cases = rhs.get_enum_cases();

        if left_cases.len() > 1 || right_cases.len() > 1 {
            return false;
        }

        let (left_enum, left_case) = left_cases[0];
        let (right_enum, right_case) = right_cases[0];

        return right_case.is_some() && left_case.is_some() && left_enum == right_enum && left_case == right_case;
    }

    if let (Some(l), Some(r)) = (lhs.get_single_literal_int_value(), rhs.get_single_literal_int_value()) {
        return l == r;
    }

    if let (Some(l), Some(r)) = (lhs.get_single_literal_float_value(), rhs.get_single_literal_float_value()) {
        return l == r;
    }

    if let (Some(l), Some(r)) = (lhs.get_single_literal_string_value(), rhs.get_single_literal_string_value()) {
        return l == r;
    }

    if !lhs.is_single() || !rhs.is_single() {
        return false;
    }

    match (lhs.get_single(), rhs.get_single()) {
        (TAtomic::Array(lhs), TAtomic::Array(rhs)) => are_sealed_arrays_always_identical(lhs, rhs),
        _ => false,
    }
}

fn are_sealed_arrays_always_identical(lhs: &TArray, rhs: &TArray) -> bool {
    match (lhs, rhs) {
        (TArray::List(lhs), TArray::List(rhs)) => {
            if !lhs.element_type.is_never()
                || !rhs.element_type.is_never()
                || lhs.known_count != rhs.known_count
                || lhs.known_count.is_none()
            {
                return false;
            }

            let (Some(lhs_elements), Some(rhs_elements)) = (&lhs.known_elements, &rhs.known_elements) else {
                return false;
            };

            lhs_elements.len() == rhs_elements.len()
                && lhs_elements.iter().zip(rhs_elements).all(
                    |((lhs_offset, (lhs_optional, lhs_type)), (rhs_offset, (rhs_optional, rhs_type)))| {
                        lhs_offset == rhs_offset
                            && !lhs_optional
                            && !rhs_optional
                            && is_always_identical_to(lhs_type, rhs_type)
                    },
                )
        }
        (TArray::Keyed(lhs), TArray::Keyed(rhs)) => {
            if lhs.parameters.is_some() || rhs.parameters.is_some() {
                return false;
            }

            let lhs_items = lhs.known_items.as_ref().filter(|items| !items.is_empty());
            let rhs_items = rhs.known_items.as_ref().filter(|items| !items.is_empty());

            match (lhs_items, rhs_items) {
                (None, None) => !lhs.non_empty && !rhs.non_empty,
                (Some(lhs_items), Some(rhs_items)) if lhs_items.len() <= 1 && rhs_items.len() <= 1 => {
                    lhs_items.len() == rhs_items.len()
                        && lhs_items.iter().zip(rhs_items).all(
                            |((lhs_key, (lhs_optional, lhs_type)), (rhs_key, (rhs_optional, rhs_type)))| {
                                lhs_key == rhs_key
                                    && !lhs_optional
                                    && !rhs_optional
                                    && is_always_identical_to(lhs_type, rhs_type)
                            },
                        )
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// Checks if two types are guaranteed to be non-equal under PHP's loose equality (`==`).
///
/// Loose equality performs type juggling that can make values of different types compare
/// equal (e.g. `0 == "0"`, `0 == false`, `5 == 5.0`, `"10" == "1e1"`). This function is
/// a safe approximation that handles primitive categories and literal integer-string pairs.
pub fn are_definitely_not_loosely_equal(
    codebase: &CodebaseMetadata,
    php_version: PHPVersion,
    lhs: &TUnion,
    rhs: &TUnion,
) -> bool {
    if let Some(loosely_equal) = compare_literal_int_and_string(php_version, lhs, rhs) {
        return !loosely_equal;
    }

    if (lhs.is_int() && rhs.is_int())
        || (lhs.is_bool() && rhs.is_bool())
        || (lhs.is_float() && rhs.is_float())
        || (lhs.is_null() && rhs.is_null())
    {
        are_definitely_not_identical(codebase, lhs, rhs, false)
    } else {
        false
    }
}

pub fn are_definitely_loosely_equal(php_version: PHPVersion, lhs: &TUnion, rhs: &TUnion) -> bool {
    compare_literal_int_and_string(php_version, lhs, rhs) == Some(true)
}

fn compare_literal_int_and_string(php_version: PHPVersion, lhs: &TUnion, rhs: &TUnion) -> Option<bool> {
    if php_version < PHPVersion::PHP80 {
        return None;
    }

    if let (Some(integer), Some(string)) = (lhs.get_single_literal_int_value(), rhs.get_single_literal_string_value()) {
        return Some(numeric_string_equals_int(string, integer));
    }

    if let (Some(string), Some(integer)) = (lhs.get_single_literal_string_value(), rhs.get_single_literal_int_value()) {
        return Some(numeric_string_equals_int(string, integer));
    }

    None
}

pub fn are_definitely_not_identical(
    codebase: &CodebaseMetadata,
    lhs: &TUnion,
    rhs: &TUnion,
    allow_type_coercion: bool,
) -> bool {
    // If either type is mixed, we cannot determine non-identity.
    if lhs.has_mixed() || lhs.has_mixed_template() || rhs.has_mixed() || rhs.has_mixed_template() {
        return false;
    }

    if !can_expression_types_be_identical(codebase, lhs, rhs, true, allow_type_coercion) {
        return true;
    }

    if (lhs.is_never() && !rhs.is_never()) || (!lhs.is_never() && rhs.is_never()) {
        return true;
    }

    if lhs.is_enum() && rhs.is_enum() {
        let left_cases = lhs.get_enum_cases();
        let right_cases = rhs.get_enum_cases();

        if left_cases.len() == 1
            && right_cases.len() == 1
            && let (left_enum, Some(left_case)) = left_cases[0]
            && let (right_enum, Some(right_case)) = right_cases[0]
        {
            return !left_enum.as_bytes().eq_ignore_ascii_case(right_enum.as_bytes()) || left_case != right_case;
        }
    }

    if (lhs.is_null() && (!rhs.is_null() && !rhs.can_be_null()))
        || (rhs.is_null() && (!lhs.is_null() && !lhs.can_be_null()))
    {
        return true;
    }

    if lhs.is_bool() {
        if !rhs.has_bool() {
            return true;
        }

        if rhs.is_true() && lhs.is_false() {
            return true;
        }

        if rhs.is_false() && lhs.is_true() {
            return true;
        }

        return !rhs.has_bool();
    } else if rhs.is_bool() && !lhs.has_bool() {
        return true;
    }
    // neither side is a fixed bool; fall through to literal-value comparisons

    if let Some(l) = lhs.get_single_literal_int_value()
        && let Some(r) = rhs.get_single_literal_int_value()
    {
        l != r
    } else if let Some(l) = lhs.get_single_literal_float_value()
        && let Some(r) = rhs.get_single_literal_float_value()
    {
        l != r
    } else if let Some(l) = lhs.get_single_literal_string_value() {
        if let Some(r) = rhs.get_single_literal_string_value() {
            l != r
        } else if let Some(r) = rhs.get_single_class_string_value() {
            !l.eq_ignore_ascii_case(r.as_bytes())
        } else {
            false
        }
    } else if let Some(r) = rhs.get_single_literal_string_value()
        && let Some(l) = lhs.get_single_class_string_value()
    {
        !r.eq_ignore_ascii_case(l.as_bytes())
    } else {
        false
    }
}
