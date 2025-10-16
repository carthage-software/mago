use mago_atom::atom;
use mago_atom::concat_atom;

use crate::metadata::CodebaseMetadata;
use crate::metadata::class_like_constant::ClassLikeConstantMetadata;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::keyed::TKeyedArray;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::iterable::TIterable;
use crate::ttype::atomic::mixed::TMixed;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::r#enum::TEnum;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::array_comparator;
use crate::ttype::comparator::callable_comparator;
use crate::ttype::comparator::derived_comparator;
use crate::ttype::comparator::generic_comparator;
use crate::ttype::comparator::object_comparator;
use crate::ttype::comparator::resource_comparator;
use crate::ttype::comparator::scalar_comparator;
use crate::ttype::comparator::union_comparator;

use super::iterable_comparator;

pub fn is_contained_by(
    codebase: &CodebaseMetadata,
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
                input_intersection_type,
                container_type_part,
                inside_assertion,
                atomic_comparison_result,
            ) {
                return true;
            }
        }
    }

    if container_type_part.is_vanilla_mixed() || container_type_part.is_templated_as_vanilla_mixed() {
        return true;
    }

    if container_type_part.is_mixed() || container_type_part.is_templated_as_mixed() {
        if matches!(container_type_part, TAtomic::Mixed(mixed) if mixed.is_non_null())
            && (matches!(input_type_part, TAtomic::Null)
                || matches!(input_type_part, TAtomic::Mixed(mixed) if !mixed.is_non_null()))
        {
            return false;
        }

        return true;
    }

    if input_type_part.is_derived() || container_type_part.is_derived() {
        return derived_comparator::is_contained_by(
            codebase,
            input_type_part,
            container_type_part,
            inside_assertion,
            &mut ComparisonResult::new(),
        );
    }

    if input_type_part.is_some_scalar() {
        if container_type_part.is_generic_scalar() {
            return true;
        }

        if container_type_part.is_some_scalar() {
            return scalar_comparator::is_contained_by(
                codebase,
                input_type_part,
                container_type_part,
                inside_assertion,
                atomic_comparison_result,
            );
        }
    }

    if matches!(container_type_part, TAtomic::Placeholder) {
        return true;
    }

    if matches!(input_type_part, TAtomic::Never) {
        return true;
    }

    if let TAtomic::Object(TObject::Enum(enum_container)) = container_type_part {
        return match input_type_part {
            TAtomic::Object(TObject::Enum(enum_input)) => {
                if !codebase.is_instance_of(enum_input.get_name_ref(), enum_container.get_name_ref()) {
                    return false;
                }

                if let Some(container_case) = enum_container.case.as_ref() {
                    if let Some(input_case) = enum_input.case.as_ref() {
                        return container_case == input_case;
                    } else {
                        return false;
                    }
                }

                true
            }
            TAtomic::Object(TObject::Named(named_object)) if enum_container.case.is_none() => {
                if !codebase.is_instance_of(named_object.get_name_ref(), enum_container.get_name_ref()) {
                    return false;
                }

                if named_object.has_type_parameters() {
                    atomic_comparison_result.type_coerced = Some(true);
                }

                true
            }
            _ => false,
        };
    }

    if input_type_part.is_mixed() || input_type_part.is_templated_as_mixed() {
        atomic_comparison_result.type_coerced = Some(true);
        atomic_comparison_result.type_coerced_from_nested_mixed = Some(true);
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

    if let TAtomic::Callable(TCallable::Signature(_)) = container_type_part {
        if input_type_part.can_be_callable() {
            return callable_comparator::is_contained_by(
                codebase,
                input_type_part,
                container_type_part,
                atomic_comparison_result,
            );
        }

        return false;
    }

    if let TAtomic::Resource(_) = container_type_part {
        return resource_comparator::is_contained_by(input_type_part, container_type_part);
    }

    if let TAtomic::Array(_) = container_type_part
        && let TAtomic::Array(_) = input_type_part
    {
        return array_comparator::is_contained_by(
            codebase,
            input_type_part,
            container_type_part,
            inside_assertion,
            atomic_comparison_result,
        );
    }

    if let TAtomic::Iterable(_) = container_type_part {
        return iterable_comparator::is_contained_by(
            codebase,
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

    if let TAtomic::Object(TObject::WithProperties(container_object_with_properties)) = container_type_part
        && let TAtomic::Object(input_object) = input_type_part
    {
        return match input_object {
            TObject::Any => {
                !container_object_with_properties.sealed && container_object_with_properties.known_properties.is_empty()
            }
            TObject::WithProperties(input_object_with_properties) => {
                if container_object_with_properties.sealed && !input_object_with_properties.sealed {
                    return false;
                }

                for (container_property_name, (container_property_indefinite, container_property_type)) in
                    &container_object_with_properties.known_properties
                {
                    let Some((input_property_indefinite, input_property_type)) =
                        input_object_with_properties.known_properties.get(container_property_name)
                    else {
                        if *container_property_indefinite && input_object_with_properties.sealed {
                            continue;
                        }

                        return false;
                    };

                    if !*container_property_indefinite && *input_property_indefinite {
                        return false;
                    }

                    if !union_comparator::is_contained_by(
                        codebase,
                        input_property_type,
                        container_property_type,
                        false,
                        false,
                        inside_assertion,
                        atomic_comparison_result,
                    ) {
                        return false;
                    }
                }

                true
            }
            TObject::Named(TNamedObject { name: input_object_name, .. })
            | TObject::Enum(TEnum { name: input_object_name, .. }) => {
                let Some(class_like_metadata) = codebase.get_class_like(input_object_name) else {
                    return false;
                };

                for (container_property_name, (container_property_indefinite, container_property_type)) in
                    &container_object_with_properties.known_properties
                {
                    let property_name = concat_atom!("$", container_property_name);

                    let Some(declaring_class) = class_like_metadata.declaring_property_ids.get(&property_name) else {
                        if !*container_property_indefinite {
                            return false;
                        } else {
                            continue;
                        }
                    };

                    let Some(declaring_class_metadata) = codebase.get_class_like(declaring_class) else {
                        return false; // should not happen
                    };

                    let Some(declared_property) = declaring_class_metadata.properties.get(&property_name) else {
                        return false; // should not happen
                    };

                    match declared_property.type_metadata.as_ref() {
                        Some(property_type_metadata) => {
                            if !union_comparator::is_contained_by(
                                codebase,
                                container_property_type,
                                &property_type_metadata.type_union,
                                false,
                                false,
                                inside_assertion,
                                atomic_comparison_result,
                            ) {
                                return false;
                            }
                        }
                        None => {
                            if !container_property_type.is_mixed() {
                                return false;
                            }
                        }
                    };
                }

                if container_object_with_properties.sealed {
                    // For sealed objects, we need to ensure the input object doesn't have
                    // properties that aren't in the container type
                    for property_name in class_like_metadata.declaring_property_ids.keys() {
                        let actual_property_name =
                            atom(property_name.as_str().strip_prefix('$').unwrap_or(property_name.as_str()));

                        // Check if this property exists in our container's known properties
                        if !container_object_with_properties.known_properties.contains_key(&actual_property_name) {
                            return false; // Input object has a property not allowed in sealed container
                        }
                    }
                }

                true
            }
        };
    }

    if let TAtomic::Object(TObject::Any) = input_type_part
        && let TAtomic::Object(TObject::Named(_) | TObject::Enum(_)) = container_type_part
    {
        atomic_comparison_result.type_coerced = Some(true);
        return false;
    }

    if let TAtomic::GenericParameter(TGenericParameter { constraint: container_constraint, .. }) = container_type_part
        && let TAtomic::GenericParameter(TGenericParameter { constraint: input_constraint, .. }) = input_type_part
    {
        return union_comparator::is_contained_by(
            codebase,
            input_constraint,
            container_constraint,
            false,
            input_constraint.ignore_falsable_issues,
            inside_assertion,
            atomic_comparison_result,
        );
    }

    if (matches!(input_type_part, TAtomic::Object(TObject::Named(_) | TObject::Enum(_)))
        || input_type_part.is_templated_as_object())
        && (matches!(container_type_part, TAtomic::Object(TObject::Named(_) | TObject::Enum(_)))
            || container_type_part.is_templated_as_object())
    {
        if !object_comparator::is_intersection_shallowly_contained_by(
            codebase,
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
        for container_extends_type_part in container_constraint.types.iter() {
            if inside_assertion
                && is_contained_by(
                    codebase,
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
    first_part: &'a TAtomic,
    second_part: &'a TAtomic,
    inside_assertion: bool,
    allow_type_coercion: bool,
) -> bool {
    if matches!(
        (first_part, second_part),
        // If either part is a variable, they can be identical
        (TAtomic::Variable(_), _) | (_, TAtomic::Variable(_)) |
        // If either part is `mixed`, they can be identical
        (TAtomic::Mixed(_), _) | (_, TAtomic::Mixed(_))
        // If one is `iterable` and other is `array`, `object`, or `iterable`, they can be identical
        | (TAtomic::Iterable(_), TAtomic::Iterable(_) | TAtomic::Array(_) | TAtomic::Object(_))
        | (TAtomic::Array(_) | TAtomic::Object(_), TAtomic::Iterable(_))
        // If one is `numeric` or `array-key` and other is `string`, they can be identical
        | (TAtomic::Scalar(TScalar::Numeric | TScalar::ArrayKey), TAtomic::Scalar(TScalar::String(_)))
        | (TAtomic::Scalar(TScalar::String(_)), TAtomic::Scalar(TScalar::Numeric | TScalar::ArrayKey))
        // If one is `int`|`float`, and the other is `numeric`, they can be identical
        | (TAtomic::Scalar(TScalar::Integer(_) | TScalar::Float(_)), TAtomic::Scalar(TScalar::Numeric))
        | (TAtomic::Scalar(TScalar::Numeric), TAtomic::Scalar(TScalar::Integer(_) | TScalar::Float(_)))
    ) {
        return true;
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
        return if let Some(first_element_type) = first_part.get_list_element_type()
            && let Some(second_element_type) = first_part.get_list_element_type()
        {
            union_comparator::can_expression_types_be_identical(
                codebase,
                first_element_type,
                second_element_type,
                inside_assertion,
                false,
            )
        } else {
            false
        };
    }

    if let (TAtomic::Array(TArray::Keyed(first_array)), TAtomic::Array(TArray::Keyed(second_array))) =
        (first_part, second_part)
    {
        return keyed_arrays_can_be_identical(first_array, second_array, codebase, inside_assertion);
    }

    let mut first_comparison_result = ComparisonResult::new();
    let mut second_comparison_result = ComparisonResult::new();

    if is_contained_by(codebase, first_part, second_part, inside_assertion, &mut first_comparison_result)
        || is_contained_by(codebase, second_part, first_part, inside_assertion, &mut second_comparison_result)
        || (first_comparison_result.type_coerced.unwrap_or(false)
            && second_comparison_result.type_coerced.unwrap_or(false))
        || (allow_type_coercion && first_part.is_some_scalar() && second_part.is_some_scalar())
    {
        return true;
    };

    if let TAtomic::GenericParameter(first_generic) = first_part {
        for first_constraint_part in first_generic.constraint.types.iter() {
            if can_be_identical(codebase, first_constraint_part, second_part, inside_assertion, allow_type_coercion) {
                return true;
            }
        }
    }

    if let TAtomic::GenericParameter(second_generic) = second_part {
        for second_constraint_part in second_generic.constraint.types.iter() {
            if can_be_identical(codebase, first_part, second_constraint_part, inside_assertion, allow_type_coercion) {
                return true;
            }
        }
    }

    false
}

pub fn expand_constant_value(v: &ClassLikeConstantMetadata) -> TAtomic {
    v.inferred_type.clone().unwrap_or(
        v.type_metadata.as_ref().map(|t| t.type_union.get_single()).cloned().unwrap_or(TAtomic::Mixed(TMixed::new())),
    )
}

fn keyed_arrays_can_be_identical(
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
                    &first_parameters.0,
                    &second_parameters.0,
                    inside_assertion,
                    false,
                ) && union_comparator::can_expression_types_be_identical(
                    codebase,
                    &first_parameters.1,
                    &second_parameters.1,
                    inside_assertion,
                    false,
                )
            }
        };
    }

    match (&first_array.known_items, &second_array.known_items) {
        (Some(first_known_items), Some(second_known_items)) => {
            let mut all_keys = first_known_items.keys().collect::<Vec<_>>();
            all_keys.extend(second_known_items.keys());

            for key in all_keys {
                match (first_known_items.get(key), second_known_items.get(key)) {
                    (Some(first_entry), Some(second_entry)) => {
                        if !union_comparator::can_expression_types_be_identical(
                            codebase,
                            &first_entry.1,
                            &second_entry.1,
                            inside_assertion,
                            false,
                        ) {
                            return false;
                        }
                    }
                    (Some(first_entry), None) => {
                        if let Some(second_parameters) = &second_array.parameters {
                            if !union_comparator::can_expression_types_be_identical(
                                codebase,
                                &first_entry.1,
                                &second_parameters.1,
                                inside_assertion,
                                false,
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
                                &first_parameters.1,
                                &second_entry.1,
                                inside_assertion,
                                false,
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
                        &first_entry.1,
                        &second_parameters.1,
                        inside_assertion,
                        false,
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
                        &first_parameters.1,
                        &second_entry.1,
                        inside_assertion,
                        false,
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
                &first_parameters.0,
                &second_parameters.0,
                inside_assertion,
                false,
            ) && union_comparator::can_expression_types_be_identical(
                codebase,
                &first_parameters.1,
                &second_parameters.1,
                inside_assertion,
                false,
            )
        }
    }
}
