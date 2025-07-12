use ahash::HashMap;
use ahash::HashSet;
use ahash::RandomState;
use indexmap::IndexMap;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;

use crate::get_class_like;
use crate::metadata::CodebaseMetadata;
use crate::misc::GenericParent;
use crate::ttype::TType;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::combiner;
use crate::ttype::get_never;
use crate::ttype::template::TemplateBound;
use crate::ttype::template::TemplateResult;
use crate::ttype::template::standin_type_replacer;
use crate::ttype::template::standin_type_replacer::get_most_specific_type_from_bounds;
use crate::ttype::union::TUnion;
use crate::ttype::wrap_atomic;

pub fn replace(
    union: &TUnion,
    template_result: &TemplateResult,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
) -> TUnion {
    let mut keys_to_unset = HashSet::default();
    let mut new_types = Vec::new();

    for atomic_type in &union.types {
        let mut atomic_type = atomic_type.clone();
        atomic_type = replace_atomic(atomic_type, template_result, codebase, interner);

        match &atomic_type {
            TAtomic::GenericParameter(TGenericParameter {
                parameter_name,
                defining_entity,
                constraint,
                intersection_types,
            }) => {
                let key = parameter_name;

                let template_type = replace_template_parameter(
                    &template_result.lower_bounds,
                    parameter_name,
                    defining_entity,
                    codebase,
                    interner,
                    constraint,
                    intersection_types,
                    key,
                );

                if let Some(template_type) = template_type {
                    keys_to_unset.insert(*key);

                    for template_type_part in template_type.types {
                        new_types.push(replace_atomic(template_type_part, template_result, codebase, interner));
                    }
                } else {
                    new_types.push(atomic_type);
                }
            }
            TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic {
                kind,
                parameter_name,
                defining_entity,
                ..
            })) => {
                if let Some(bounds) =
                    template_result.lower_bounds.get(parameter_name).unwrap_or(&HashMap::default()).get(defining_entity)
                {
                    let template_type = get_most_specific_type_from_bounds(bounds, codebase, interner);

                    let mut class_template_type = None;

                    for template_type_part in &template_type.types {
                        if template_type_part.is_mixed() || matches!(template_type_part, TAtomic::Object(TObject::Any))
                        {
                            class_template_type =
                                Some(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Any { kind: *kind })));
                        } else if let TAtomic::Object(TObject::Named(_)) = template_type_part {
                            class_template_type =
                                Some(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType {
                                    kind: *kind,
                                    constraint: Box::new(template_type_part.clone()),
                                })));
                        } else if let TAtomic::GenericParameter(TGenericParameter {
                            constraint,
                            parameter_name,
                            defining_entity,
                            ..
                        }) = template_type_part
                        {
                            let first_atomic_type = constraint.get_single();

                            class_template_type =
                                Some(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic {
                                    kind: *kind,
                                    parameter_name: *parameter_name,
                                    constraint: Box::new(first_atomic_type.clone()),
                                    defining_entity: *defining_entity,
                                })))
                        }
                    }

                    if let Some(class_template_type) = class_template_type {
                        keys_to_unset.insert(*parameter_name);
                        new_types.push(class_template_type);
                    }
                }
            }
            _ => {
                new_types.push(atomic_type);
            }
        }
    }

    if new_types.is_empty() {
        return get_never();
    }

    union.clone_with_types(combiner::combine(new_types, codebase, interner, false))
}

#[allow(clippy::too_many_arguments)]
fn replace_template_parameter(
    inferred_lower_bounds: &IndexMap<StringIdentifier, HashMap<GenericParent, Vec<TemplateBound>>, RandomState>,
    parameter_name: &StringIdentifier,
    defining_entity: &GenericParent,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
    constraint: &TUnion,
    intersection_types: &Option<Vec<TAtomic>>,
    key: &StringIdentifier,
) -> Option<TUnion> {
    let mut template_type = None;
    let traversed_type = standin_type_replacer::get_root_template_type(
        inferred_lower_bounds,
        parameter_name,
        defining_entity,
        HashSet::default(),
        codebase,
        interner,
    );

    if let Some(traversed_type) = traversed_type {
        let template_type_inner = if !constraint.is_mixed() && traversed_type.is_mixed() {
            if constraint.is_array_key() { wrap_atomic(TAtomic::Scalar(TScalar::ArrayKey)) } else { constraint.clone() }
        } else {
            traversed_type.clone()
        };

        if let Some(_intersection_types) = intersection_types {
            for _atomic_template_type in &template_type_inner.types {
                // todo handle extra types
            }
        }

        template_type = Some(template_type_inner);
    } else {
        for (_, template_type_map) in inferred_lower_bounds {
            for map_defining_entity in template_type_map.keys() {
                let classlike_name = match map_defining_entity {
                    GenericParent::ClassLike(e) => e,
                    _ => {
                        continue;
                    }
                };

                if let Some(metadata) = get_class_like(codebase, interner, classlike_name)
                    && let Some(templated_extended_parameter_map) =
                        metadata.template_extended_parameters.get(&metadata.name)
                    && let Some(inner_parameter) = templated_extended_parameter_map.get(key)
                {
                    let template_name = if let TAtomic::GenericParameter(TGenericParameter { parameter_name, .. }) =
                        inner_parameter.get_single()
                    {
                        parameter_name
                    } else {
                        panic!()
                    };

                    if let Some(bounds_map) = inferred_lower_bounds.get(template_name)
                        && let Some(bounds) = bounds_map.get(map_defining_entity)
                    {
                        template_type =
                            Some(standin_type_replacer::get_most_specific_type_from_bounds(bounds, codebase, interner));
                    }
                }
            }
        }
    }

    template_type
}

fn replace_atomic(
    mut atomic: TAtomic,
    template_result: &TemplateResult,
    codebase: &CodebaseMetadata,
    interner: &ThreadedInterner,
) -> TAtomic {
    match &mut atomic {
        TAtomic::Conditional(conditional) => {
            conditional.subject = Box::new(replace(&conditional.subject, template_result, codebase, interner));
            conditional.target = Box::new(replace(&conditional.target, template_result, codebase, interner));
            conditional.then = Box::new(replace(&conditional.then, template_result, codebase, interner));
            conditional.otherwise = Box::new(replace(&conditional.otherwise, template_result, codebase, interner));
        }
        TAtomic::Array(array_type) => match array_type {
            TArray::List(list_data) => {
                list_data.element_type =
                    Box::new(replace(&list_data.element_type, template_result, codebase, interner));

                if let Some(known_elements) = &mut list_data.known_elements {
                    for (_, element_type) in known_elements.values_mut() {
                        *element_type = replace(element_type, template_result, codebase, interner);
                    }
                }
            }
            TArray::Keyed(keyed_data) => {
                if let Some((key_parameter, value_parameter)) = &mut keyed_data.parameters {
                    *key_parameter = Box::new(replace(key_parameter, template_result, codebase, interner));
                    *value_parameter = Box::new(replace(value_parameter, template_result, codebase, interner));
                }

                if let Some(known_items) = &mut keyed_data.known_items {
                    for (_, item_type) in known_items.values_mut() {
                        *item_type = replace(item_type, template_result, codebase, interner);
                    }
                }
            }
        },
        TAtomic::Iterable(iterable) => {
            let key_type = iterable.get_key_type_mut();
            *key_type = replace(key_type, template_result, codebase, interner);

            let value_type = iterable.get_value_type_mut();
            *value_type = replace(value_type, template_result, codebase, interner);

            if let Some(intersection_types) = iterable.get_intersection_types_mut() {
                let old_intersection_types = TUnion::new(intersection_types.clone());

                *intersection_types = replace(&old_intersection_types, template_result, codebase, interner).types;
            }
        }
        TAtomic::Object(TObject::Named(named_object)) => {
            if let Some(type_parameters) = named_object.get_type_parameters_mut() {
                for parameter in type_parameters {
                    *parameter = replace(parameter, template_result, codebase, interner);
                }
            }

            if let Some(intersection_types) = named_object.get_intersection_types_mut() {
                let old_intersection_types = TUnion::new(intersection_types.clone());

                *intersection_types = replace(&old_intersection_types, template_result, codebase, interner).types;
            }
        }
        TAtomic::Callable(TCallable::Signature(signature)) => {
            for parameter in signature.get_parameters_mut() {
                if let Some(t) = parameter.get_type_signature_mut() {
                    *t = replace(t, template_result, codebase, interner);
                }
            }

            if let Some(return_type) = signature.get_return_type_mut() {
                *return_type = replace(return_type, template_result, codebase, interner);
            }
        }
        _ => (),
    }

    atomic
}
