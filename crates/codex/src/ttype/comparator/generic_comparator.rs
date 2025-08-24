use crate::get_class_like;
use crate::is_instance_of;
use crate::metadata::CodebaseMetadata;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::union_comparator;
use crate::ttype::get_specialized_template_type;
use crate::ttype::template::variance::Variance;

pub(crate) fn is_contained_by(
    codebase: &CodebaseMetadata,
    input_type_part: &TAtomic,
    container_type_part: &TAtomic,
    inside_assertion: bool,
    atomic_comparison_result: &mut ComparisonResult,
) -> bool {
    let TAtomic::Object(TObject::Named(container_object)) = container_type_part else {
        return false;
    };

    let TAtomic::Object(TObject::Named(input_object)) = input_type_part else {
        return false;
    };

    let Some(container_metadata) = get_class_like(codebase, &container_object.name) else {
        return false;
    };

    let Some(input_metadata) = get_class_like(codebase, &input_object.name) else {
        return false;
    };

    if !is_instance_of(codebase, &input_object.name, &container_object.name) {
        return false;
    }

    let container_type_parameters = container_object.get_type_parameters().unwrap_or_default();
    let input_type_parameters = input_object.get_type_parameters().unwrap_or_default();

    let mut all_parameters_match = true;
    for (parameter_offset, container_type_parameter) in container_type_parameters.iter().enumerate() {
        let Some((template_name, _)) = container_metadata.template_types.get(parameter_offset) else {
            continue;
        };

        let Some(specialized_template_type) = get_specialized_template_type(
            codebase,
            template_name,
            &container_metadata.name,
            input_metadata,
            Some(input_type_parameters),
        ) else {
            return false;
        };

        let mut parameter_comparison_result = ComparisonResult::new();

        if !union_comparator::is_contained_by(
            codebase,
            &specialized_template_type,
            container_type_parameter,
            false,
            specialized_template_type.ignore_falsable_issues,
            false,
            &mut parameter_comparison_result,
        ) {
            if let Some(Variance::Contravariant) = container_metadata.template_variance.get(&parameter_offset)
                && union_comparator::is_contained_by(
                    codebase,
                    container_type_parameter,
                    &specialized_template_type,
                    false,
                    container_type_parameter.ignore_falsable_issues,
                    inside_assertion,
                    &mut parameter_comparison_result,
                )
            {
                continue;
            }

            update_failed_result_from_nested(atomic_comparison_result, parameter_comparison_result);

            all_parameters_match = false;
        }
    }

    all_parameters_match
}

pub(crate) fn update_failed_result_from_nested(
    atomic_comparison_result: &mut ComparisonResult,
    param_comparison_result: ComparisonResult,
) {
    atomic_comparison_result.type_coerced = Some(if let Some(val) = atomic_comparison_result.type_coerced {
        val
    } else {
        param_comparison_result.type_coerced.unwrap_or(false)
    });

    atomic_comparison_result.type_coerced_from_nested_mixed =
        Some(if let Some(val) = atomic_comparison_result.type_coerced_from_nested_mixed {
            val
        } else {
            param_comparison_result.type_coerced_from_nested_mixed.unwrap_or(false)
        });

    atomic_comparison_result.type_coerced_from_as_mixed =
        Some(if let Some(val) = atomic_comparison_result.type_coerced_from_as_mixed {
            val
        } else {
            param_comparison_result.type_coerced_from_as_mixed.unwrap_or(false)
        });

    atomic_comparison_result.type_coerced_to_literal =
        Some(if let Some(val) = atomic_comparison_result.type_coerced_to_literal {
            val
        } else {
            param_comparison_result.type_coerced_to_literal.unwrap_or(false)
        });
}
