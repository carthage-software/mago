use mago_atom::Atom;

use crate::metadata::CodebaseMetadata;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::object::TObject;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::union_comparator;
use crate::ttype::get_specialized_template_type;
use crate::ttype::template::variance::Variance;
use crate::ttype::union::TUnion;

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

    let (input_name, input_type_parameters): (Atom, Option<&[TUnion]>) = match input_type_part {
        TAtomic::Object(TObject::Named(obj)) => (obj.name, obj.get_type_parameters()),
        TAtomic::Object(TObject::Enum(e)) => (e.name, None),
        _ => return false,
    };

    let Some(container_metadata) = codebase.get_class_like(&container_object.name) else {
        return false;
    };

    let Some(input_metadata) = codebase.get_class_like(&input_name) else {
        return false;
    };

    if !codebase.is_instance_of(&input_name, &container_object.name) {
        return false;
    }

    let Some(container_type_parameters) = container_object.get_type_parameters() else {
        return true;
    };

    let mut all_parameters_match = true;
    for (parameter_offset, container_type_parameter) in container_type_parameters.iter().enumerate() {
        let Some((template_name, _)) = container_metadata.template_types.get_index(parameter_offset) else {
            continue;
        };

        let Some(mut specialized_template_type) = get_specialized_template_type(
            codebase,
            template_name,
            &container_metadata.name,
            input_metadata,
            input_type_parameters,
        ) else {
            return false;
        };

        // When the input has no explicit type parameters, the specialized type
        // comes from template defaults, not explicit annotations.
        if input_type_parameters.is_none() {
            specialized_template_type.set_from_template_default(true);
        }

        let mut parameter_comparison_result = ComparisonResult::new();

        if !union_comparator::is_contained_by(
            codebase,
            &specialized_template_type,
            container_type_parameter,
            false,
            specialized_template_type.ignore_falsable_issues(),
            false,
            &mut parameter_comparison_result,
        ) {
            if let Some(Variance::Contravariant) = container_metadata.template_variance.get(&parameter_offset)
                && union_comparator::is_contained_by(
                    codebase,
                    container_type_parameter,
                    &specialized_template_type,
                    false,
                    container_type_parameter.ignore_falsable_issues(),
                    inside_assertion,
                    &mut parameter_comparison_result,
                )
            {
                continue;
            }

            update_failed_result_from_nested(atomic_comparison_result, &parameter_comparison_result);

            if !parameter_comparison_result.type_coerced_from_as_mixed.unwrap_or(false) {
                all_parameters_match = false;
            }
        }
    }

    all_parameters_match
}

pub(crate) fn update_failed_result_from_nested(
    atomic_comparison_result: &mut ComparisonResult,
    param_comparison_result: &ComparisonResult,
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
