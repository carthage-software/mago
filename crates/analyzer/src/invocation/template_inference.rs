use std::borrow::Cow;

use ahash::HashMap;

use ahash::HashSet;
use mago_atom::Atom;
use mago_codex::get_class_like;
use mago_codex::is_instance_of;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::MethodMetadata;
use mago_codex::misc::GenericParent;
use mago_codex::ttype::TType;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::add_union_type;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::key::ArrayKey;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::class_like_string::TClassLikeString;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::atomic_comparator;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::expander::get_signature_of_function_like_identifier;
use mago_codex::ttype::get_array_parameters;
use mago_codex::ttype::get_array_value_parameter;
use mago_codex::ttype::get_iterable_parameters;
use mago_codex::ttype::get_specialized_template_type;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::template::standin_type_replacer::StandinOptions;
use mago_codex::ttype::template::standin_type_replacer::insert_bound_type;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;

use crate::code::IssueCode;
use crate::context::Context;
use crate::invocation::MethodTargetContext;

#[derive(Debug, Clone, Copy, Default)]
pub struct InferenceOptions {
    pub infer_only_if_new: bool,
    pub argument_offset: Option<usize>,
    pub source_span: Option<Span>,
}

#[derive(Debug)]
pub struct TemplateInferenceViolation {
    pub template_name: Atom,
    pub inferred_bound: TUnion,
    pub constraint: TUnion,
}

fn infer_templates_from_input_and_container_types(
    context: &Context<'_, '_>,
    container_type: &TUnion,
    input_type: &TUnion,
    template_result: &mut TemplateResult,
    options: InferenceOptions,
    violations: &mut Vec<TemplateInferenceViolation>,
) {
    if input_type.is_mixed() {
        return;
    }

    let (generic_container_parts, concrete_container_parts) = container_type.types.iter().partition::<Vec<_>, _>(|t| {
        matches!(
            t,
            TAtomic::GenericParameter(_)
                | TAtomic::Array(_)
                | TAtomic::Iterable(_)
                | TAtomic::Object(TObject::Named(_))
                | TAtomic::Callable(_)
                | TAtomic::Scalar(TScalar::ClassLikeString(_))
        )
    });

    let residual_input_types = input_type
        .types
        .iter()
        .filter(|argument_atomic| {
            !argument_atomic.is_empty_array()
                && !concrete_container_parts.iter().any(|container_atomic| {
                    atomic_comparator::is_contained_by(
                        context.codebase,
                        container_atomic,
                        argument_atomic,
                        false,
                        &mut ComparisonResult::default(),
                    )
                })
        })
        .cloned()
        .collect::<Vec<_>>();

    let residual_input_type = if residual_input_types.is_empty() {
        return;
    } else {
        TUnion::from_vec(residual_input_types)
    };

    let mut potential_template_violations = HashMap::default();

    for container_atomic_part in &generic_container_parts {
        match container_atomic_part {
            TAtomic::Array(container_array) => {
                for input_atomic in residual_input_type.types.as_ref() {
                    if let TAtomic::Array(input_array) = input_atomic {
                        match (container_array, input_array) {
                            (TArray::List(container_list), TArray::List(input_list)) => {
                                let mut inferred_input_elements = vec![];
                                if let Some(container_elements) = &container_list.known_elements {
                                    for (container_index, (_, container_element)) in container_elements {
                                        let input_element = input_list
                                            .known_elements
                                            .as_ref()
                                            .and_then(|elements| {
                                                let (_, input_element) = elements.get(container_index)?;

                                                inferred_input_elements.push(container_index);

                                                Some(input_element)
                                            })
                                            .or_else(|| {
                                                if input_list.element_type.is_never() {
                                                    None
                                                } else {
                                                    Some(input_list.element_type.as_ref())
                                                }
                                            });

                                        let Some(input_element) = input_element else {
                                            continue;
                                        };

                                        infer_templates_from_input_and_container_types(
                                            context,
                                            container_element,
                                            input_element,
                                            template_result,
                                            options,
                                            violations,
                                        );
                                    }
                                }

                                if !container_list.element_type.is_never() {
                                    let mut input_value_type = Cow::Borrowed(input_list.element_type.as_ref());

                                    if let Some(known_input_elements) = &input_list.known_elements {
                                        for (input_index, (_, input_element)) in known_input_elements {
                                            if !inferred_input_elements.contains(&input_index) {
                                                input_value_type = Cow::Owned(add_union_type(
                                                    input_element.clone(),
                                                    input_value_type.as_ref(),
                                                    context.codebase,
                                                    false,
                                                ))
                                            }
                                        }
                                    }

                                    infer_templates_from_input_and_container_types(
                                        context,
                                        &container_list.element_type,
                                        &input_value_type,
                                        template_result,
                                        options,
                                        violations,
                                    );
                                }
                            }
                            (TArray::Keyed(container_array), TArray::Keyed(input_array)) => {
                                let mut inferred_input_keys = vec![];
                                if let Some(known_items) = &container_array.known_items {
                                    for (container_key, (_, container_item)) in known_items {
                                        let input_item = input_array
                                            .known_items
                                            .as_ref()
                                            .and_then(|items| {
                                                let (_, input_item) = items.get(container_key)?;

                                                inferred_input_keys.push(container_key);

                                                Some(input_item)
                                            })
                                            .or_else(|| {
                                                input_array.parameters.as_ref().map(|params| params.1.as_ref())
                                            });

                                        let Some(input_item) = input_item else {
                                            continue;
                                        };

                                        infer_templates_from_input_and_container_types(
                                            context,
                                            container_item,
                                            input_item,
                                            template_result,
                                            options,
                                            violations,
                                        );
                                    }
                                }

                                if let Some(container_parameter) = container_array.parameters.as_ref() {
                                    let mut input_key_type =
                                        if let Some(input_parameter) = input_array.parameters.as_ref() {
                                            Some(Cow::Borrowed(input_parameter.0.as_ref()))
                                        } else {
                                            None
                                        };

                                    let mut input_value_type =
                                        if let Some(input_parameter) = input_array.parameters.as_ref() {
                                            Some(Cow::Borrowed(input_parameter.1.as_ref()))
                                        } else {
                                            None
                                        };

                                    if let Some(known_input_items) = &input_array.known_items {
                                        for (input_key, (_, input_item)) in known_input_items {
                                            if !inferred_input_keys.contains(&input_key) {
                                                input_key_type = Some(Cow::Owned(add_optional_union_type(
                                                    input_key.to_union(),
                                                    input_key_type.as_deref(),
                                                    context.codebase,
                                                )));

                                                input_value_type = Some(Cow::Owned(add_optional_union_type(
                                                    input_item.clone(),
                                                    input_value_type.as_deref(),
                                                    context.codebase,
                                                )));
                                            }
                                        }
                                    }

                                    if let Some(input_key_type) = input_key_type {
                                        infer_templates_from_input_and_container_types(
                                            context,
                                            &container_parameter.0,
                                            &input_key_type,
                                            template_result,
                                            options,
                                            violations,
                                        );
                                    }

                                    if let Some(input_value_type) = input_value_type {
                                        infer_templates_from_input_and_container_types(
                                            context,
                                            &container_parameter.1,
                                            &input_value_type,
                                            template_result,
                                            options,
                                            violations,
                                        );
                                    }
                                }
                            }
                            (TArray::List(container_list), TArray::Keyed(input_keyed_array)) => {
                                let mut matched_input_keys: HashSet<ArrayKey> = HashSet::default();

                                if let Some(container_elements) = &container_list.known_elements {
                                    for (container_index, (_, container_element)) in container_elements.iter() {
                                        if let Some(known_items) = &input_keyed_array.known_items {
                                            let key = ArrayKey::Integer(*container_index as i64);
                                            if let Some((_, input_element)) = known_items.get(&key) {
                                                matched_input_keys.insert(key);

                                                infer_templates_from_input_and_container_types(
                                                    context,
                                                    container_element,
                                                    input_element,
                                                    template_result,
                                                    options,
                                                    violations,
                                                );

                                                continue;
                                            }
                                        }

                                        let input_element = input_keyed_array
                                            .parameters
                                            .as_ref()
                                            .map(|params| params.1.as_ref())
                                            .map(Cow::Borrowed)
                                            .unwrap_or_else(|| {
                                                Cow::Owned(get_array_value_parameter(input_array, context.codebase))
                                            });

                                        infer_templates_from_input_and_container_types(
                                            context,
                                            container_element,
                                            &input_element,
                                            template_result,
                                            options,
                                            violations,
                                        );
                                    }
                                }

                                if !container_list.element_type.is_never() {
                                    let mut input_value_type =
                                        if let Some(params) = input_keyed_array.parameters.as_ref() {
                                            Cow::Borrowed(params.1.as_ref())
                                        } else {
                                            Cow::Owned(get_array_value_parameter(input_array, context.codebase))
                                        };

                                    if let Some(known_input_items) = &input_keyed_array.known_items {
                                        for (input_key, (_, input_item)) in known_input_items.iter() {
                                            if !matched_input_keys.contains(input_key) {
                                                input_value_type = Cow::Owned(add_union_type(
                                                    input_item.clone(),
                                                    input_value_type.as_ref(),
                                                    context.codebase,
                                                    false,
                                                ));
                                            }
                                        }
                                    }

                                    infer_templates_from_input_and_container_types(
                                        context,
                                        &container_list.element_type,
                                        &input_value_type,
                                        template_result,
                                        options,
                                        violations,
                                    );
                                }
                            }
                            (TArray::Keyed(container_array), TArray::List(input_list)) => {
                                let mut matched_input_indices: HashSet<usize> = HashSet::default();

                                if let Some(known_items) = &container_array.known_items {
                                    for (container_key, (_, container_item)) in known_items.iter() {
                                        match container_key {
                                            ArrayKey::Integer(i) if *i >= 0 => {
                                                let idx = *i as usize;
                                                if let Some(known_elems) = &input_list.known_elements
                                                    && let Some((_, input_elem)) = known_elems.get(&idx)
                                                {
                                                    matched_input_indices.insert(idx);

                                                    infer_templates_from_input_and_container_types(
                                                        context,
                                                        container_item,
                                                        input_elem,
                                                        template_result,
                                                        options,
                                                        violations,
                                                    );

                                                    continue;
                                                }

                                                // no exact element at that index -> fall back to list element_type
                                                if !input_list.element_type.is_never() {
                                                    infer_templates_from_input_and_container_types(
                                                        context,
                                                        container_item,
                                                        input_list.element_type.as_ref(),
                                                        template_result,
                                                        options,
                                                        violations,
                                                    );
                                                }
                                            }
                                            _ => {
                                                if !input_list.element_type.is_never() {
                                                    infer_templates_from_input_and_container_types(
                                                        context,
                                                        container_item,
                                                        input_list.element_type.as_ref(),
                                                        template_result,
                                                        options,
                                                        violations,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }

                                if let Some(container_parameter) = container_array.parameters.as_ref() {
                                    let input_params = get_array_parameters(input_array, context.codebase);
                                    let mut input_key_type = Some(Cow::Owned(input_params.0));
                                    let mut input_value_type = Some(Cow::Borrowed(&input_params.1));

                                    if let Some(known_input_elements) = &input_list.known_elements {
                                        for (input_index, (_, input_element)) in known_input_elements.iter() {
                                            if !matched_input_indices.contains(input_index) {
                                                let int_key = ArrayKey::Integer(*input_index as i64);

                                                input_key_type = Some(Cow::Owned(add_optional_union_type(
                                                    int_key.to_union(),
                                                    input_key_type.as_deref(),
                                                    context.codebase,
                                                )));

                                                input_value_type = Some(Cow::Owned(add_optional_union_type(
                                                    input_element.clone(),
                                                    input_value_type.as_deref(),
                                                    context.codebase,
                                                )));
                                            }
                                        }
                                    }

                                    if let Some(input_key_type) = input_key_type {
                                        infer_templates_from_input_and_container_types(
                                            context,
                                            &container_parameter.0,
                                            &input_key_type,
                                            template_result,
                                            options,
                                            violations,
                                        );
                                    }

                                    if let Some(input_value_type) = input_value_type {
                                        infer_templates_from_input_and_container_types(
                                            context,
                                            &container_parameter.1,
                                            &input_value_type,
                                            template_result,
                                            options,
                                            violations,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            TAtomic::Iterable(container_iterable) => {
                for input_atomic in residual_input_type.types.as_ref() {
                    let Some(input_params) = get_iterable_parameters(input_atomic, context.codebase) else {
                        return;
                    };

                    infer_templates_from_input_and_container_types(
                        context,
                        container_iterable.get_key_type(),
                        &input_params.0,
                        template_result,
                        options,
                        violations,
                    );

                    infer_templates_from_input_and_container_types(
                        context,
                        container_iterable.get_value_type(),
                        &input_params.1,
                        template_result,
                        options,
                        violations,
                    );
                }
            }
            TAtomic::Callable(container_callable) => {
                let container_signature = match container_callable {
                    TCallable::Signature(signature) => Cow::Borrowed(signature),
                    TCallable::Alias(id) => {
                        let Some(signature) = get_signature_of_function_like_identifier(id, context.codebase) else {
                            continue;
                        };

                        Cow::Owned(signature)
                    }
                };

                for input_atomic in residual_input_type.types.as_ref() {
                    let input_signature = match input_atomic {
                        TAtomic::Callable(TCallable::Signature(argument_signature)) => {
                            Cow::Borrowed(argument_signature)
                        }
                        TAtomic::Callable(TCallable::Alias(id)) => {
                            let Some(signature) = get_signature_of_function_like_identifier(id, context.codebase)
                            else {
                                continue;
                            };

                            Cow::Owned(signature)
                        }
                        _ => continue,
                    };

                    let container_parameters = container_signature.get_parameters();
                    let input_parameters = input_signature.get_parameters();

                    let container_count = container_parameters.len();
                    let input_count = input_parameters.iter().filter(|s| !s.has_default()).count();
                    let minimum_count = std::cmp::min(container_count, input_count);
                    for i in 0..minimum_count {
                        let Some(container_parameter) = container_parameters.get(i) else {
                            continue;
                        };

                        let Some(input_parameter) = input_parameters.get(i) else {
                            continue;
                        };

                        let Some(container_parameter_type) = container_parameter.get_type_signature() else {
                            continue;
                        };

                        let Some(input_parameter_type) = input_parameter.get_type_signature() else {
                            continue;
                        };

                        infer_templates_from_input_and_container_types(
                            context,
                            container_parameter_type,
                            input_parameter_type,
                            template_result,
                            InferenceOptions { infer_only_if_new: true, ..options },
                            violations,
                        );
                    }

                    let Some(container_return) = container_signature.get_return_type() else {
                        continue;
                    };

                    let Some(input_return) = input_signature.get_return_type() else {
                        continue;
                    };

                    infer_templates_from_input_and_container_types(
                        context,
                        container_return,
                        input_return,
                        template_result,
                        InferenceOptions { infer_only_if_new: false, ..options },
                        violations,
                    );
                }
            }
            TAtomic::Object(TObject::Named(container_obj)) => {
                let Some(container_params) = container_obj.get_type_parameters() else {
                    continue;
                };

                let Some(container_meta) = get_class_like(context.codebase, &container_obj.name) else {
                    continue;
                };

                for input_atomic in residual_input_type.types.as_ref() {
                    let TAtomic::Object(TObject::Named(input_obj)) = input_atomic else {
                        continue;
                    };

                    let Some(input_meta) = get_class_like(context.codebase, &input_obj.name) else {
                        continue;
                    };

                    if !is_instance_of(context.codebase, &input_obj.name, &container_obj.name) {
                        continue;
                    }

                    for (index, parameter_template_union) in container_params.iter().enumerate() {
                        let generic_parameters =
                            parameter_template_union.types.iter().filter_map(|atomic| match atomic {
                                TAtomic::GenericParameter(generic_parameter) => Some(generic_parameter),
                                _ => None,
                            });

                        for generic_parameter in generic_parameters {
                            let Some((template_name, _)) = container_meta.template_types.get(index) else {
                                continue;
                            };

                            if let Some(inferred_bound) = get_specialized_template_type(
                                context.codebase,
                                template_name,
                                &container_meta.name,
                                input_meta,
                                input_obj.get_type_parameters(),
                            ) {
                                if !union_comparator::is_contained_by(
                                    context.codebase,
                                    &inferred_bound,
                                    &generic_parameter.constraint,
                                    false,
                                    false,
                                    false,
                                    &mut ComparisonResult::default(),
                                ) {
                                    violations.push(TemplateInferenceViolation {
                                        template_name: *template_name,
                                        inferred_bound: inferred_bound.clone(),
                                        constraint: generic_parameter.constraint.as_ref().clone(),
                                    });
                                }

                                insert_bound_type(
                                    template_result,
                                    generic_parameter.parameter_name,
                                    &generic_parameter.defining_entity,
                                    inferred_bound,
                                    StandinOptions { appearance_depth: 1, ..Default::default() },
                                    options.argument_offset,
                                    options.source_span,
                                );
                            }
                        }
                    }
                }
            }
            TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic {
                parameter_name,
                defining_entity,
                ..
            })) => {
                let should_add_bound = !options.infer_only_if_new
                    || template_result
                        .lower_bounds
                        .get(parameter_name)
                        .and_then(|map| map.get(defining_entity))
                        .is_none_or(|bounds| bounds.is_empty());

                let mut input_objects = vec![];
                for input_atomic in residual_input_type.types.iter() {
                    let TAtomic::Scalar(TScalar::ClassLikeString(class_string)) = input_atomic else {
                        continue;
                    };

                    input_objects.push(class_string.get_object_type(context.codebase));
                }

                if input_objects.is_empty() || !should_add_bound {
                    continue;
                }

                let mut lower_bound_type = TUnion::from_vec(input_objects);
                if let Some(template_types) = template_result.template_types.get_mut(parameter_name) {
                    for (_, template_type) in template_types {
                        if !union_comparator::is_contained_by(
                            context.codebase,
                            &lower_bound_type,
                            template_type,
                            false,
                            false,
                            false,
                            &mut ComparisonResult::default(),
                        ) {
                            lower_bound_type = template_type.clone();

                            violations.push(TemplateInferenceViolation {
                                template_name: *parameter_name,
                                inferred_bound: lower_bound_type.clone(),
                                constraint: template_type.clone(),
                            });
                        }
                    }
                }

                insert_bound_type(
                    template_result,
                    *parameter_name,
                    defining_entity,
                    lower_bound_type,
                    StandinOptions { appearance_depth: 1, ..Default::default() },
                    options.argument_offset,
                    options.source_span,
                );
            }
            _ => {}
        }
    }

    for container_atomic_part in generic_container_parts {
        let TAtomic::GenericParameter(container_generic) = container_atomic_part else {
            continue;
        };

        let template_parameter_name = &container_generic.parameter_name;

        let should_add_bound = !options.infer_only_if_new
            || template_result
                .lower_bounds
                .get(template_parameter_name)
                .and_then(|map| map.get(&container_generic.defining_entity))
                .is_none_or(|bounds| bounds.is_empty());

        if !should_add_bound {
            continue;
        }

        let mut has_violation = false;

        if let Some(template_types) = template_result.template_types.get_mut(template_parameter_name) {
            for (_, template_type) in template_types {
                if !union_comparator::is_contained_by(
                    context.codebase,
                    &residual_input_type,
                    template_type,
                    false,
                    false,
                    false,
                    &mut ComparisonResult::default(),
                ) {
                    potential_template_violations
                        .entry((*template_parameter_name, container_generic.defining_entity))
                        .or_insert_with(|| {
                            (residual_input_type.clone(), template_type.clone(), container_generic.clone())
                        });

                    has_violation = true;
                    break;
                }
            }
        }

        if has_violation {
            continue;
        }

        insert_bound_type(
            template_result,
            *template_parameter_name,
            &container_generic.defining_entity,
            residual_input_type.clone(),
            StandinOptions { appearance_depth: 1, ..Default::default() },
            options.argument_offset,
            options.source_span,
        );
    }

    for ((template_parameter_name, defining_entity), (inferred_type, constraint, _)) in potential_template_violations {
        let is_unresolved = template_result
            .lower_bounds
            .get(&template_parameter_name)
            .and_then(|map| map.get(&defining_entity))
            .is_none_or(|bounds| bounds.is_empty());

        if is_unresolved {
            violations.push(TemplateInferenceViolation {
                template_name: template_parameter_name,
                inferred_bound: inferred_type,
                constraint: constraint.clone(),
            });

            insert_bound_type(
                template_result,
                template_parameter_name,
                &defining_entity,
                constraint,
                StandinOptions { appearance_depth: 1, ..Default::default() },
                options.argument_offset,
                options.source_span,
            );
        }
    }
}

pub fn infer_templates_for_method_call<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    object_type: &TNamedObject,
    method_target_context: &MethodTargetContext<'ctx>,
    method_metadata: &'ctx MethodMetadata,
    declaring_class_like_metadata: &'ctx ClassLikeMetadata,
    template_result: &mut TemplateResult,
) {
    if declaring_class_like_metadata.name != method_target_context.class_like_metadata.name {
        for (template_name, _) in &declaring_class_like_metadata.template_types {
            let template_type = get_specialized_template_type(
                context.codebase,
                template_name,
                &declaring_class_like_metadata.name,
                method_target_context.class_like_metadata,
                object_type.get_type_parameters(),
            );

            if let Some(template_type) = template_type {
                template_result.add_lower_bound(
                    *template_name,
                    GenericParent::ClassLike(declaring_class_like_metadata.name),
                    template_type,
                );
            }
        }
    }

    for (template_name, where_constraint) in &method_metadata.where_constraints {
        let Some(actual_type) = get_specialized_template_type(
            context.codebase,
            template_name,
            &declaring_class_like_metadata.name,
            method_target_context.class_like_metadata,
            object_type.get_type_parameters(),
        ) else {
            continue;
        };

        if !union_comparator::is_contained_by(
            context.codebase,
            &actual_type,
            &where_constraint.type_union,
            false,
            false,
            false,
            &mut ComparisonResult::default(),
        ) {
            continue;
        }

        infer_templates_from_input_and_container_types(
            context,
            &where_constraint.type_union,
            &actual_type,
            template_result,
            InferenceOptions { source_span: Some(where_constraint.span), ..Default::default() },
            &mut Default::default(),
        );
    }
}

/// Infers template types for a parameter based on a **passed argument**.
///
/// This function is the primary mechanism for generic type inference. It compares a
/// parameter's declared type with the type of the actual argument passed to it to
/// determine what the template types should be for the function call.
///
/// It also validates that the inferred argument type satisfies any template
/// constraints (e.g., `<T as SomeInterface>`), reporting an error if it does not.
///
/// # Arguments
///
/// * `context`: The analysis context.
/// * `parameter_type`: The declared type of the parameter (the "container").
/// * `argument_type`: The type of the argument that was passed (the "input").
/// * `template_result`: The map where inferred template types are stored.
/// * `argument_offset`: The numerical position of the argument in the call.
/// * `argument_span`: The source code location of the argument, for error reporting.
/// * `is_callable_argument`: A flag indicating if the argument is a callable, which
///   can influence inference strategy.
pub fn infer_parameter_templates_from_argument<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    parameter_type: &TUnion,
    argument_type: &TUnion,
    template_result: &mut TemplateResult,
    argument_offset: usize,
    argument_span: Span,
    is_callable_argument: bool,
) {
    let mut violations = vec![];
    infer_templates_from_input_and_container_types(
        context,
        parameter_type,
        argument_type,
        template_result,
        InferenceOptions {
            infer_only_if_new: is_callable_argument,
            argument_offset: Some(argument_offset),
            source_span: Some(argument_span),
        },
        &mut violations,
    );

    for violation in violations {
        context.collector.report_with_code(
            IssueCode::TemplateConstraintViolation,
            Issue::error(format!("Argument type mismatch for template `{}`.", violation.template_name,))
                .with_annotation(Annotation::primary(argument_span).with_message(format!(
                    "This argument has type `{}`, which is not compatible with the required template constraint `{}`.",
                    violation.inferred_bound.get_id(),
                    violation.constraint.get_id()
                )))
                .with_note(format!(
                    "Template parameter `{}` is constrained with `{}`.",
                    violation.template_name,
                    violation.constraint.get_id()
                ))
                .with_help("Ensure the argument's type satisfies the template constraint."),
        );
    }
}

/// Infers template types for a parameter based on its **default value**.
///
/// This function is used when an argument for a generic parameter is omitted in a
/// function call. It reconciles the parameter's declared type (which contains
/// templates) with the known type of its default value to determine what the
/// templates should resolve to.
///
/// For example, if a function `add<A, B>(A $a, B $b = 1): A|B` is called as `add(2)`,
/// this function would be used on parameter `$b` to infer that `T` is `int`.
///
/// # Arguments
///
/// * `context`: The analysis context.
/// * `parameter_type`: The declared type of the parameter (the "container").
/// * `default_type`: The type of the parameter's default value (the "input").
/// * `template_result`: The map where inferred template types are stored.
pub fn infer_parameter_templates_from_default<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    parameter_type: &TUnion,
    default_type: &TUnion,
    template_result: &mut TemplateResult,
) {
    infer_templates_from_input_and_container_types(
        context,
        parameter_type,
        default_type,
        template_result,
        InferenceOptions { infer_only_if_new: default_type.is_callable(), argument_offset: None, source_span: None },
        &mut Vec::new(),
    );
}
