use crate::get_class_like;
use crate::is_instance_of;
use crate::metadata::CodebaseMetadata;
use crate::misc::GenericParent;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::object::TObject;
use crate::ttype::comparator::ComparisonResult;
use crate::ttype::comparator::union_comparator;
use crate::ttype::wrap_atomic;
use crate::uses_trait;

pub(crate) fn is_shallowly_contained_by(
    codebase: &CodebaseMetadata,
    input_type_part: &TAtomic,
    container_type_part: &TAtomic,
    inside_assertion: bool,
    atomic_comparison_result: &mut ComparisonResult,
) -> bool {
    // `T <= A & B`
    if let Some(container_intersection_types) = container_type_part.get_intersection_types()
        && !container_intersection_types.is_empty()
    {
        for part in container_intersection_types {
            if !is_shallowly_contained_by(codebase, input_type_part, part, inside_assertion, atomic_comparison_result) {
                return false;
            }
        }
    }
    // `A & B <= T`
    else if let Some(input_intersection_types) = input_type_part.get_intersection_types()
        && !input_intersection_types.is_empty()
    {
        for part in input_intersection_types {
            if is_shallowly_contained_by(
                codebase,
                part,
                container_type_part,
                inside_assertion,
                atomic_comparison_result,
            ) {
                return true;
            }
        }
    }

    is_intersection_shallowly_contained_by(
        codebase,
        input_type_part,
        container_type_part,
        inside_assertion,
        atomic_comparison_result,
    )
}

pub(super) fn is_intersection_shallowly_contained_by(
    codebase: &CodebaseMetadata,
    intersection_input_type: &TAtomic,
    intersection_container_type: &TAtomic,
    inside_assertion: bool,
    atomic_comparison_result: &mut ComparisonResult,
) -> bool {
    if let TAtomic::GenericParameter(TGenericParameter {
        defining_entity: container_defining_entity,
        parameter_name: container_parameter_name,
        ..
    }) = intersection_container_type
    {
        if let TAtomic::GenericParameter(TGenericParameter {
            defining_entity: input_defining_entity,
            parameter_name: input_parameter_name,
            constraint: input_constraint,
            ..
        }) = intersection_input_type
        {
            if input_parameter_name == container_parameter_name && input_defining_entity == container_defining_entity {
                return true;
            }

            match (input_defining_entity, container_defining_entity) {
                (
                    GenericParent::ClassLike(input_defining_class),
                    GenericParent::ClassLike(container_defining_class),
                ) => {
                    if input_defining_class != container_defining_class
                        && let Some(input_class_metadata) = get_class_like(codebase, input_defining_class)
                        && let Some(defining_entity_params) =
                            &input_class_metadata.template_extended_parameters.get(container_defining_class)
                        && defining_entity_params.get(container_parameter_name).is_some()
                    {
                        return true;
                    }
                }
                (GenericParent::ClassLike(_), _) | (_, GenericParent::ClassLike(_)) => {
                    for input_as_atomic in input_constraint.types.as_ref() {
                        // todo use type equality
                        if input_as_atomic == intersection_container_type {
                            return true;
                        }
                    }
                }
                _ => {
                    if input_parameter_name != container_parameter_name {
                        return false;
                    }
                    if input_defining_entity != container_defining_entity {
                        return true;
                    }
                }
            };

            return false;
        }

        return false;
    }

    if let TAtomic::GenericParameter(TGenericParameter { constraint: input_constraint, .. }) = intersection_input_type {
        let mut intersection_container_type = intersection_container_type.clone();

        if let TAtomic::Object(TObject::Named(named_object)) = &mut intersection_container_type {
            named_object.is_this = false;
        }

        return union_comparator::is_contained_by(
            codebase,
            input_constraint,
            &wrap_atomic(intersection_container_type),
            false,
            input_constraint.ignore_falsable_issues,
            inside_assertion,
            atomic_comparison_result,
        );
    }

    let (container_name, container_is_this) = match intersection_container_type {
        TAtomic::Object(TObject::Named(o)) => (o.name, o.is_this),
        TAtomic::Object(TObject::Enum(e)) => (e.name, false),
        _ => {
            return false;
        }
    };

    let (input_name, input_is_this) = match intersection_input_type {
        TAtomic::Object(TObject::Named(o)) => (o.name, o.is_this),
        TAtomic::Object(TObject::Enum(e)) => (e.name, false),
        _ => {
            return false;
        }
    };

    if container_is_this && !input_is_this && !inside_assertion {
        atomic_comparison_result.type_coerced = Some(true);
        return false;
    }

    if is_instance_of(codebase, &input_name, &container_name) || uses_trait(codebase, &input_name, &container_name) {
        return true;
    }

    if is_instance_of(codebase, &container_name, &input_name) {
        atomic_comparison_result.type_coerced = Some(true);
    }

    false
}
