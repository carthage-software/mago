use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::comparator::union_comparator::can_expression_types_be_identical;
use mago_codex::ttype::union::TUnion;

#[inline]
pub fn is_always_less_than_or_equal(lhs: &TUnion, rhs: &TUnion) -> bool {
    if let (Some(max_lhs), Some(min_rhs)) = (lhs.get_single_maximum_int_value(), rhs.get_single_minimum_int_value()) {
        return max_lhs <= min_rhs;
    }

    is_always_less_than(lhs, rhs) || is_always_identical_to(lhs, rhs)
}

#[inline]
pub fn is_always_greater_than_or_equal(lhs: &TUnion, rhs: &TUnion) -> bool {
    if let (Some(min_lhs), Some(max_rhs)) = (lhs.get_single_minimum_int_value(), rhs.get_single_maximum_int_value()) {
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

    if !lhs.is_single() || !rhs.is_single() {
        return false;
    }

    let lhs_atomic = lhs.get_single();
    let rhs_atomic = rhs.get_single();

    match (lhs_atomic, rhs_atomic) {
        (TAtomic::Scalar(TScalar::Integer(l)), TAtomic::Scalar(TScalar::Integer(r))) => match (l, r) {
            (TInteger::Literal(l_val), TInteger::Literal(r_val)) => return l_val < r_val,
            _ => return false,
        },
        (TAtomic::Scalar(TScalar::Float(l)), TAtomic::Scalar(TScalar::Float(r))) => match (l.value, r.value) {
            (Some(l_val), Some(r_val)) => return l_val < r_val,
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

    if !lhs.is_single() || !rhs.is_single() {
        return false;
    }

    let lhs_atomic = lhs.get_single();
    let rhs_atomic = rhs.get_single();

    match (lhs_atomic, rhs_atomic) {
        (TAtomic::Scalar(TScalar::Integer(l)), TAtomic::Scalar(TScalar::Integer(r))) => match (l, r) {
            (TInteger::Literal(l_val), TInteger::Literal(r_val)) => return l_val > r_val,
            _ => return false,
        },
        (TAtomic::Scalar(TScalar::Float(l)), TAtomic::Scalar(TScalar::Float(r))) => match (l.value, r.value) {
            (Some(l_val), Some(r_val)) => return l_val > r_val,
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

    false
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

    if (lhs.is_null() && (!rhs.is_null() && !rhs.is_nullable()))
        || (rhs.is_null() && (!lhs.is_null() && !lhs.is_nullable()))
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
            !l.eq_ignore_ascii_case(&r)
        } else {
            false
        }
    } else if let Some(r) = rhs.get_single_literal_string_value()
        && let Some(l) = lhs.get_single_class_string_value()
    {
        !r.eq_ignore_ascii_case(&l)
    } else {
        false
    }
}
