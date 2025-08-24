use crate::metadata::CodebaseMetadata;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::float::TFloat;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::class_string_comparator;

pub fn is_contained_by(
    codebase: &CodebaseMetadata,
    input_type_part: &TAtomic,
    container_type_part: &TAtomic,
    inside_assertion: bool,
    atomic_comparison_result: &mut ComparisonResult,
) -> bool {
    match (container_type_part, input_type_part) {
        (TAtomic::Null, TAtomic::Null) => return true,
        (
            TAtomic::Scalar(TScalar::Numeric),
            TAtomic::Scalar(TScalar::Integer(_) | TScalar::Float(_) | TScalar::Numeric),
        ) => return true,
        (TAtomic::Scalar(TScalar::Numeric), TAtomic::Scalar(TScalar::String(string))) if string.is_numeric => {
            return true;
        }
        (TAtomic::Scalar(TScalar::Generic), TAtomic::Scalar(_)) => return true,
        (TAtomic::Scalar(TScalar::ArrayKey), TAtomic::Scalar(i)) if i.is_int() || i.is_any_string() => {
            return true;
        }
        (TAtomic::Scalar(TScalar::Integer(ci)), TAtomic::Scalar(TScalar::Integer(ii))) if ci.contains(*ii) => {
            return true;
        }
        (
            TAtomic::Scalar(TScalar::Float(TFloat { value: None })),
            // Special case: In PHP, an integer can be passed as a float, but not the other way around.
            TAtomic::Scalar(TScalar::Float(_)) | TAtomic::Scalar(TScalar::Integer(_)),
        ) => {
            return true;
        }
        (
            TAtomic::Scalar(TScalar::Float(TFloat { value: Some(c) })),
            TAtomic::Scalar(TScalar::Float(TFloat { value: Some(i) })),
        ) if c == i => {
            return true;
        }
        (TAtomic::Scalar(TScalar::String(c)), TAtomic::Scalar(TScalar::String(_) | TScalar::ClassLikeString(_)))
            if c.is_boring() =>
        {
            return true;
        }
        (TAtomic::Scalar(TScalar::Bool(c)), TAtomic::Scalar(TScalar::Bool(_))) if c.is_general() => {
            return true;
        }
        (TAtomic::Scalar(TScalar::Bool(c)), TAtomic::Scalar(TScalar::Bool(i))) if c.is_true() && i.is_true() => {
            return true;
        }
        (TAtomic::Scalar(TScalar::Bool(c)), TAtomic::Scalar(TScalar::Bool(i))) if c.is_false() && i.is_false() => {
            return true;
        }
        (TAtomic::Scalar(TScalar::String(c)), TAtomic::Scalar(TScalar::String(i))) => {
            if c.is_truthy() && !i.is_truthy() {
                return false;
            }

            if c.is_known_numeric() && !i.is_known_numeric() {
                return false;
            }

            if c.is_non_empty() && !i.is_non_empty() {
                return false;
            }

            if c.is_lowercase() && !i.is_lowercase() {
                return false;
            }

            if c.is_unspecified_literal() && !i.is_literal_origin() {
                return false;
            } else if let Some(v) = c.get_known_literal_value() {
                if let Some(i) = i.get_known_literal_value() {
                    if v != i {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            return true;
        }
        (TAtomic::Scalar(TScalar::String(c)), TAtomic::Scalar(TScalar::ClassLikeString(_)))
            if !c.is_unspecified_literal() && !c.is_numeric =>
        {
            return true;
        }
        (TAtomic::Scalar(TScalar::ClassLikeString(container_class_string)), TAtomic::Scalar(input_scalar)) => {
            return class_string_comparator::is_contained_by(
                codebase,
                input_scalar,
                container_class_string,
                inside_assertion,
                atomic_comparison_result,
            );
        }
        _ => {}
    }

    if matches!(input_type_part, TAtomic::Scalar(TScalar::String(s)) if s.is_boring())
        && container_type_part.is_string_subtype()
    {
        atomic_comparison_result.type_coerced = Some(true);
        if container_type_part.is_literal() {
            atomic_comparison_result.type_coerced_to_literal = Some(true);
        }

        return false;
    }

    if (matches!(input_type_part, TAtomic::Scalar(TScalar::Bool(input)) if input.is_general()))
        && matches!(container_type_part, TAtomic::Scalar(TScalar::Bool(container)) if !container.is_general())
    {
        atomic_comparison_result.type_coerced = Some(true);
        return false;
    }

    if let TAtomic::Scalar(TScalar::ArrayKey) = input_type_part
        && (container_type_part.is_int() || container_type_part.is_any_string())
    {
        atomic_comparison_result.type_coerced = Some(true);
        return false;
    }

    if matches!(input_type_part, TAtomic::Scalar(TScalar::String(s)) if s.is_non_empty && !s.is_truthy)
        && matches!(container_type_part, TAtomic::Scalar(TScalar::ClassLikeString(_)))
    {
        atomic_comparison_result.type_coerced = Some(true);
        return false;
    }

    false
}
