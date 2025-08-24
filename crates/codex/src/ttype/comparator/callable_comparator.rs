use std::borrow::Cow;

use crate::metadata::CodebaseMetadata;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::cast::cast_atomic_to_callable;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::union_comparator;
use crate::ttype::expander::get_signature_of_function_like_identifier;

pub(crate) fn is_contained_by(
    codebase: &CodebaseMetadata,
    input_type_part: &TAtomic,
    container_type_part: &TAtomic,
    atomic_comparison_result: &mut ComparisonResult,
) -> bool {
    let TAtomic::Callable(TCallable::Signature(container_signature)) = container_type_part else {
        return false;
    };

    let Some(input_callable) = cast_atomic_to_callable(input_type_part, codebase, None) else {
        return false;
    };

    let input_signature = match input_callable.as_ref() {
        TCallable::Signature(signature) => Cow::Borrowed(signature),
        TCallable::Alias(function_like_identifier) => {
            let Some(signature) = get_signature_of_function_like_identifier(function_like_identifier, codebase) else {
                return false;
            };

            Cow::Owned(signature)
        }
    };

    if container_signature.is_closure() && !input_signature.is_closure() {
        return false;
    }

    if container_signature.is_pure() && !input_signature.is_pure() {
        return false;
    }

    for (i, input_parameters) in input_signature.get_parameters().iter().enumerate() {
        let container_parameter;
        if let Some(inner) = container_signature.get_parameters().get(i) {
            container_parameter = inner;
        } else if let Some(last_parameter) = container_signature.get_parameters().last().filter(|p| p.is_variadic()) {
            container_parameter = last_parameter;
        } else {
            if input_parameters.has_default() {
                break;
            }

            return false;
        }

        let Some(container_parameter_type) = container_parameter.get_type_signature() else {
            continue;
        };

        if container_parameter_type.is_mixed() {
            continue;
        }

        let Some(input_parameter_type) = input_parameters.get_type_signature() else {
            continue;
        };

        let mut parameter_comparison_result = ComparisonResult::new();

        if !union_comparator::is_contained_by(
            codebase,
            container_parameter_type,
            input_parameter_type,
            false,
            false,
            false,
            &mut parameter_comparison_result,
        ) {
            return false;
        }

        atomic_comparison_result
            .type_variable_lower_bounds
            .extend(parameter_comparison_result.type_variable_upper_bounds);

        atomic_comparison_result
            .type_variable_upper_bounds
            .extend(parameter_comparison_result.type_variable_lower_bounds);
    }

    let Some(container_return_type) = container_signature.get_return_type() else {
        return true;
    };

    if container_return_type.is_void() {
        return true;
    }

    let Some(input_return_type) = input_signature.get_return_type() else {
        atomic_comparison_result.type_coerced = Some(true);
        atomic_comparison_result.type_coerced_from_nested_mixed = Some(true);

        return false;
    };

    if input_return_type.is_void() && container_return_type.is_nullable() {
        return true;
    }

    if input_return_type.is_never() {
        return true;
    }

    union_comparator::is_contained_by(
        codebase,
        input_return_type,
        container_return_type,
        false,
        input_return_type.ignore_falsable_issues,
        false,
        atomic_comparison_result,
    )
}
