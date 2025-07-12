use itertools::Itertools;
use mago_interner::ThreadedInterner;

use crate::is_instance_of;
use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like_constant::ClassLikeConstantMetadata;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::keyed::TKeyedArray;
use crate::ttype::atomic::array::list::TList;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::iterable::TIterable;
use crate::ttype::atomic::mixed::TMixed;
use crate::ttype::atomic::object::TObject;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::array_comparator;
use crate::ttype::comparator::callable_comparator;
use crate::ttype::comparator::generic_comparator;
use crate::ttype::comparator::object_comparator;
use crate::ttype::comparator::resource_comparator;
use crate::ttype::comparator::scalar_comparator;
use crate::ttype::comparator::union_comparator;
use crate::ttype::get_iterable_parameters;
use crate::ttype::get_iterable_value_parameter;

use super::iterable_comparator;

pub fn is_contained_by(
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    input_type_part: &TAtomic,
    container_type_part: &TAtomic,
    inside_assertion: bool,
    atomic_comparison_result: &mut ComparisonResult,
) -> bool {
    if input_type_part == container_type_part {
        return true;
    }

    // `T <= A & B`
    if let Some(container_intersection_types) = container_type_part.get_intersection_types()
        && !container_intersection_types.is_empty()
    {
        for container_intersection_type in container_intersection_types {
            if !is_contained_by(
                codebase,
                interner,
                input_type_part,
                container_intersection_type,
                inside_assertion,
                atomic_comparison_result,
            ) {
                return false;
            }
        }

        // intersection <= intersection (e.g., A&B <= C&D)
        if input_type_part.has_intersection_types() {
            // We have proven the input is a subtype of all the container's parts.
            // This is sufficient.
            return true;
        }
    }

    // `A & B <= T`
    if let Some(input_intersection_types) = input_type_part.get_intersection_types()
        && !input_intersection_types.is_empty()
    {
        for input_intersection_type in input_intersection_types {
            if is_contained_by(
                codebase,
                interner,
                input_intersection_type,
                container_type_part,
                inside_assertion,
                atomic_comparison_result,
            ) {
                return true;
            }
        }
    }

    if let TAtomic::Object(TObject::Enum(enum_container)) = container_type_part {
        if let TAtomic::Object(TObject::Enum(enum_input)) = input_type_part {
            if !is_instance_of(codebase, interner, enum_input.get_name_ref(), enum_container.get_name_ref()) {
                return false;
            }

            if let Some(container_case) = enum_container.case.as_ref() {
                if let Some(input_case) = enum_input.case.as_ref() {
                    return container_case == input_case;
                } else {
                    return false;
                }
            }

            return true;
        }

        return false;
    }

    if container_type_part.is_mixed() || container_type_part.is_templated_as_mixed(&mut false) {
        if matches!(container_type_part, TAtomic::Mixed(mixed) if mixed.is_non_null())
            && (matches!(input_type_part, TAtomic::Null)
                || matches!(input_type_part, TAtomic::Mixed(mixed) if !mixed.is_non_null()))
        {
            return false;
        }

        return true;
    }

    if matches!(container_type_part, TAtomic::Placeholder) {
        return true;
    }

    if matches!(input_type_part, TAtomic::Never) {
        return true;
    }

    let mut input_type_has_any = false;
    if input_type_part.is_mixed_with_any(&mut input_type_has_any)
        || input_type_part.is_templated_as_mixed(&mut input_type_has_any)
    {
        atomic_comparison_result.type_coerced = Some(true);
        atomic_comparison_result.type_coerced_from_nested_mixed = Some(true);
        if input_type_has_any {
            atomic_comparison_result.type_coerced_from_nested_any = Some(true);
        }
        return false;
    }

    if let TAtomic::Null = input_type_part {
        if let TAtomic::GenericParameter(TGenericParameter { constraint, .. }) = container_type_part
            && (constraint.is_nullable() || constraint.is_mixed())
        {
            return true;
        }

        return false;
    }

    if input_type_part.is_some_scalar() {
        if container_type_part.is_generic_scalar() {
            return true;
        }

        if container_type_part.is_some_scalar() {
            return scalar_comparator::is_contained_by(
                codebase,
                interner,
                input_type_part,
                container_type_part,
                inside_assertion,
                atomic_comparison_result,
            );
        }
    }

    if let TAtomic::Callable(TCallable::Signature(_)) = container_type_part {
        if input_type_part.can_be_callable() {
            return callable_comparator::is_contained_by(
                codebase,
                interner,
                input_type_part,
                container_type_part,
                atomic_comparison_result,
            );
        }

        return false;
    }

    if let TAtomic::Object(TObject::Named(_)) = container_type_part {
        match input_type_part {
            TAtomic::Array(TArray::Keyed(_)) => {
                if let Some(parameters) = get_iterable_parameters(container_type_part, codebase, interner) {
                    return self::is_contained_by(
                        codebase,
                        interner,
                        input_type_part,
                        &TAtomic::Array(TArray::Keyed(TKeyedArray {
                            parameters: Some((Box::new(parameters.0), Box::new(parameters.1))),
                            known_items: None,
                            non_empty: false,
                        })),
                        inside_assertion,
                        atomic_comparison_result,
                    );
                }
            }
            TAtomic::Array(TArray::List(_)) => {
                if let Some(value_parameter) = get_iterable_value_parameter(container_type_part, codebase, interner) {
                    return self::is_contained_by(
                        codebase,
                        interner,
                        input_type_part,
                        &TAtomic::Array(TArray::List(TList {
                            element_type: Box::new(value_parameter),
                            known_elements: None,
                            non_empty: false,
                            known_count: None,
                        })),
                        inside_assertion,
                        atomic_comparison_result,
                    );
                }
            }
            _ => (),
        }
    }

    if let TAtomic::Resource(_) = container_type_part {
        return resource_comparator::is_contained_by(input_type_part, container_type_part);
    }

    if let TAtomic::Array(_) = container_type_part
        && let TAtomic::Array(_) = input_type_part
    {
        return array_comparator::is_contained_by(
            codebase,
            interner,
            input_type_part,
            container_type_part,
            inside_assertion,
            atomic_comparison_result,
        );
    }

    if let TAtomic::Iterable(_) = container_type_part {
        return iterable_comparator::is_contained_by(
            codebase,
            interner,
            input_type_part,
            container_type_part,
            inside_assertion,
            atomic_comparison_result,
        );
    }

    if let TAtomic::Object(TObject::Any) = container_type_part
        && let TAtomic::Object(_) = input_type_part
    {
        return true;
    }

    if let TAtomic::Object(TObject::Any) = input_type_part
        && let TAtomic::Object(TObject::Named(_) | TObject::Enum(_)) = container_type_part
    {
        atomic_comparison_result.type_coerced = Some(true);
        return false;
    }

    if (matches!(input_type_part, TAtomic::Object(TObject::Named(_) | TObject::Enum(_)))
        || input_type_part.is_templated_as_object())
        && (matches!(container_type_part, TAtomic::Object(TObject::Named(_) | TObject::Enum(_)))
            || container_type_part.is_templated_as_object())
    {
        if !object_comparator::is_intersection_shallowly_contained_by(
            codebase,
            interner,
            input_type_part,
            container_type_part,
            inside_assertion,
            atomic_comparison_result,
        ) {
            return false;
        }

        if matches!(container_type_part, TAtomic::Object(TObject::Named(obj)) if obj.has_type_parameters())
            && !generic_comparator::is_contained_by(
                codebase,
                interner,
                input_type_part,
                container_type_part,
                inside_assertion,
                atomic_comparison_result,
            )
        {
            return false;
        }

        return true;
    }

    if let TAtomic::Object(TObject::Any) = input_type_part
        && let TAtomic::Object(TObject::Any) = container_type_part
    {
        return true;
    }

    if let TAtomic::GenericParameter(TGenericParameter { constraint: container_constraint, .. }) = container_type_part {
        if let TAtomic::GenericParameter(TGenericParameter { constraint: input_constraint, .. }) = input_type_part {
            return union_comparator::is_contained_by(
                codebase,
                interner,
                input_constraint,
                container_constraint,
                false,
                input_constraint.ignore_falsable_issues,
                inside_assertion,
                atomic_comparison_result,
            );
        }

        for container_extends_type_part in container_constraint.types.iter() {
            if inside_assertion
                && is_contained_by(
                    codebase,
                    interner,
                    input_type_part,
                    container_extends_type_part,
                    inside_assertion,
                    atomic_comparison_result,
                )
            {
                return true;
            }
        }

        return false;
    }

    if let TAtomic::Iterable(TIterable { intersection_types: input_intersection_types, .. }) = input_type_part
        && let Some(input_intersection_types) = input_intersection_types
    {
        for input_intersection_type in input_intersection_types {
            if is_contained_by(
                codebase,
                interner,
                input_intersection_type,
                container_type_part,
                inside_assertion,
                atomic_comparison_result,
            ) {
                return true;
            }
        }
    }

    if let TAtomic::GenericParameter(TGenericParameter {
        intersection_types: input_intersection_types,
        constraint: input_constraint,
        ..
    }) = input_type_part
    {
        if let Some(input_intersection_types) = input_intersection_types {
            for input_intersection_type in input_intersection_types {
                if is_contained_by(
                    codebase,
                    interner,
                    input_intersection_type,
                    container_type_part,
                    inside_assertion,
                    atomic_comparison_result,
                ) {
                    return true;
                }
            }
        }

        for input_constraint_part in input_constraint.types.iter() {
            if matches!(input_constraint_part, TAtomic::Null) && matches!(container_type_part, TAtomic::Null) {
                continue;
            }

            if is_contained_by(
                codebase,
                interner,
                input_constraint_part,
                container_type_part,
                inside_assertion,
                atomic_comparison_result,
            ) {
                return true;
            }
        }

        return false;
    }

    false
}

pub(crate) fn can_be_identical<'a>(
    codebase: &'a CodebaseMetadata,
    interner: &'a ThreadedInterner,
    first_part: &'a TAtomic,
    second_part: &'a TAtomic,
    inside_assertion: bool,
) -> bool {
    if let TAtomic::Variable { .. } = first_part {
        return true;
    }

    if let TAtomic::Variable { .. } = second_part {
        return true;
    }

    if let TAtomic::Iterable(_) = first_part {
        return matches!(
            second_part,
            TAtomic::Mixed(_) | TAtomic::Iterable(_) | TAtomic::Array(_) | TAtomic::Object(_)
        );
    }

    if let (TAtomic::Object(TObject::Enum(first_enum)), TAtomic::Object(TObject::Enum(second_enum))) =
        (first_part, second_part)
        && first_enum.name == second_enum.name
    {
        return true;
    }

    if (first_part.is_list() && second_part.is_non_empty_list())
        || (second_part.is_list() && first_part.is_non_empty_list())
    {
        return union_comparator::can_expression_types_be_identical(
            codebase,
            interner,
            first_part.get_list_element_type().unwrap(),
            second_part.get_list_element_type().unwrap(),
            inside_assertion,
        );
    }

    if let (TAtomic::Array(TArray::Keyed(first_array)), TAtomic::Array(TArray::Keyed(second_array))) =
        (first_part, second_part)
    {
        return keyed_arrays_can_be_identical(interner, first_array, second_array, codebase, inside_assertion);
    }

    let mut first_comparison_result = ComparisonResult::new();
    let mut second_comparison_result = ComparisonResult::new();

    if is_contained_by(codebase, interner, first_part, second_part, inside_assertion, &mut first_comparison_result)
        || is_contained_by(codebase, interner, second_part, first_part, inside_assertion, &mut second_comparison_result)
        || (first_comparison_result.type_coerced.unwrap_or(false)
            && second_comparison_result.type_coerced.unwrap_or(false))
    {
        return true;
    };

    if let TAtomic::GenericParameter(first_generic) = first_part {
        for first_constraint_part in first_generic.constraint.types.iter() {
            if can_be_identical(codebase, interner, first_constraint_part, second_part, inside_assertion) {
                return true;
            }
        }
    }

    if let TAtomic::GenericParameter(second_generic) = second_part {
        for second_constraint_part in second_generic.constraint.types.iter() {
            if can_be_identical(codebase, interner, first_part, second_constraint_part, inside_assertion) {
                return true;
            }
        }
    }

    false
}

pub fn expand_constant_value(v: &ClassLikeConstantMetadata) -> TAtomic {
    v.get_inferred_type().cloned().unwrap_or(
        v.get_type_metadata().map(|t| t.type_union.get_single()).cloned().unwrap_or(TAtomic::Mixed(TMixed::any())),
    )
}

fn keyed_arrays_can_be_identical(
    interner: &ThreadedInterner,
    first_array: &TKeyedArray,
    second_array: &TKeyedArray,
    codebase: &CodebaseMetadata,
    inside_assertion: bool,
) -> bool {
    if first_array.non_empty || second_array.non_empty {
        return match (&first_array.parameters, &second_array.parameters) {
            (None, None) | (None, Some(_)) | (Some(_), None) => true,
            (Some(first_parameters), Some(second_parameters)) => {
                union_comparator::can_expression_types_be_identical(
                    codebase,
                    interner,
                    &first_parameters.0,
                    &second_parameters.0,
                    inside_assertion,
                ) && union_comparator::can_expression_types_be_identical(
                    codebase,
                    interner,
                    &first_parameters.1,
                    &second_parameters.1,
                    inside_assertion,
                )
            }
        };
    }

    match (&first_array.known_items, &second_array.known_items) {
        (Some(first_known_items), Some(second_known_items)) => {
            let mut all_keys = first_known_items.keys().collect_vec();
            all_keys.extend(second_known_items.keys());

            for key in all_keys {
                match (first_known_items.get(key), second_known_items.get(key)) {
                    (Some(first_entry), Some(second_entry)) => {
                        if !union_comparator::can_expression_types_be_identical(
                            codebase,
                            interner,
                            &first_entry.1,
                            &second_entry.1,
                            inside_assertion,
                        ) {
                            return false;
                        }
                    }
                    (Some(first_entry), None) => {
                        if let Some(second_parameters) = &second_array.parameters {
                            if !union_comparator::can_expression_types_be_identical(
                                codebase,
                                interner,
                                &first_entry.1,
                                &second_parameters.1,
                                inside_assertion,
                            ) {
                                return false;
                            }
                        } else if !first_entry.0 {
                            return false;
                        }
                    }
                    (None, Some(second_entry)) => {
                        if let Some(first_parameters) = &first_array.parameters {
                            if !union_comparator::can_expression_types_be_identical(
                                codebase,
                                interner,
                                &first_parameters.1,
                                &second_entry.1,
                                inside_assertion,
                            ) {
                                return false;
                            }
                        } else if !second_entry.0 {
                            return false;
                        }
                    }
                    _ => {
                        panic!("impossible");
                    }
                }
            }
        }
        (Some(first_known_items), None) => {
            for first_entry in first_known_items.values() {
                if let Some(second_parameters) = &second_array.parameters {
                    if !union_comparator::can_expression_types_be_identical(
                        codebase,
                        interner,
                        &first_entry.1,
                        &second_parameters.1,
                        inside_assertion,
                    ) {
                        return false;
                    }
                } else if !first_entry.0 {
                    return false;
                }
            }
        }
        (None, Some(second_known_items)) => {
            for second_entry in second_known_items.values() {
                if let Some(first_parameters) = &first_array.parameters {
                    if !union_comparator::can_expression_types_be_identical(
                        codebase,
                        interner,
                        &first_parameters.1,
                        &second_entry.1,
                        inside_assertion,
                    ) {
                        return false;
                    }
                } else if !second_entry.0 {
                    return false;
                }
            }
        }
        _ => {}
    };

    match (&first_array.parameters, &second_array.parameters) {
        (None, None) | (None, Some(_)) | (Some(_), None) => true,
        (Some(first_parameters), Some(second_parameters)) => {
            union_comparator::can_expression_types_be_identical(
                codebase,
                interner,
                &first_parameters.0,
                &second_parameters.0,
                inside_assertion,
            ) && union_comparator::can_expression_types_be_identical(
                codebase,
                interner,
                &first_parameters.1,
                &second_parameters.1,
                inside_assertion,
            )
        }
    }
}
