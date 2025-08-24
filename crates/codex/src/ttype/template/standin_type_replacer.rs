use std::borrow::Cow;
use std::cmp::Ordering;

use ahash::HashMap;
use ahash::HashSet;
use ahash::RandomState;
use indexmap::IndexMap;

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::empty_atom;
use mago_span::Span;

use crate::class_like_exists;
use crate::get_class_like;
use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::misc::GenericParent;
use crate::ttype::TType;
use crate::ttype::add_union_type;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::mixed::TMixed;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::object::named::TNamedObject;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeStringKind;
use crate::ttype::combiner;
use crate::ttype::comparator::union_comparator;
use crate::ttype::expander;
use crate::ttype::expander::StaticClassType;
use crate::ttype::expander::TypeExpansionOptions;
use crate::ttype::get_iterable_parameters;
use crate::ttype::get_iterable_value_parameter;
use crate::ttype::get_mixed;
use crate::ttype::get_mixed_maybe_from_loop;
use crate::ttype::template::TemplateBound;
use crate::ttype::template::TemplateResult;
use crate::ttype::template::inferred_type_replacer;
use crate::ttype::template::variance::Variance;
use crate::ttype::union::TUnion;
use crate::ttype::wrap_atomic;

#[derive(Copy, Clone, Debug)]
pub struct StandinOptions<'a> {
    pub calling_class: Option<Atom>,
    pub calling_function: Option<&'a FunctionLikeIdentifier>,
    pub add_lower_bound: bool,
    pub iteration_depth: usize,
    pub appearance_depth: usize,
}

impl Default for StandinOptions<'_> {
    fn default() -> Self {
        Self {
            calling_class: None,
            calling_function: None,
            add_lower_bound: true,
            iteration_depth: 1,
            appearance_depth: 1,
        }
    }
}

impl StandinOptions<'_> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_iteration(&self) -> Self {
        Self { iteration_depth: self.iteration_depth + 1, ..*self }
    }

    pub fn with_appearance_depth(&self, appearance_depth: usize) -> Self {
        Self { appearance_depth, ..*self }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn replace(
    parameter_type: &TUnion,
    template_result: &mut TemplateResult,
    codebase: &CodebaseMetadata,
    argument_type: &Option<&TUnion>,
    argument_offset: Option<usize>,
    argument_span: Option<Span>,
    options: StandinOptions<'_>,
) -> TUnion {
    let mut original_parameter_atomics = parameter_type.types.clone().into_owned();
    let mut new_parameter_atomics = Vec::with_capacity(original_parameter_atomics.len());

    let mut argument_type = argument_type.cloned();
    if let Some(ref mut argument_type) = argument_type
        && original_parameter_atomics.len() > 1
        && original_parameter_atomics.iter().any(|t| matches!(t, TAtomic::Null))
        && argument_type.is_mixed()
    {
        original_parameter_atomics.retain(|t| !matches!(t, TAtomic::Null));

        argument_type.types = Cow::Owned(vec![match argument_type.types[0] {
            TAtomic::Mixed(mixed) => TAtomic::Mixed(mixed.with_is_non_null(true)),
            _ => TAtomic::Mixed(TMixed::new().with_is_non_null(true)),
        }]);
    }

    if let Some(ref mut argument_type_inner) = argument_type
        && !argument_type_inner.is_single()
    {
        // here we want to subtract atomic types from the input type
        // when they're also in the union type, so those shared atomic
        // types will never be inferred as part of the generic type
        for original_atomic_type in &original_parameter_atomics {
            argument_type_inner.remove_type(original_atomic_type);
        }

        if argument_type_inner.types.is_empty() {
            return parameter_type.clone();
        }
    }

    let mut had_template = false;
    for atomic_type in original_parameter_atomics.iter() {
        new_parameter_atomics.extend(handle_atomic_standin(
            atomic_type,
            template_result,
            codebase,
            &argument_type.as_ref(),
            argument_offset,
            argument_span,
            options,
            original_parameter_atomics.len() == 1,
            &mut had_template,
        ))
    }

    if new_parameter_atomics.is_empty() {
        return parameter_type.clone();
    }

    let mut new_union_type = TUnion::from_vec(if new_parameter_atomics.len() > 1 {
        combiner::combine(new_parameter_atomics, codebase, false)
    } else {
        new_parameter_atomics
    });

    new_union_type.ignore_falsable_issues = parameter_type.ignore_falsable_issues;

    if had_template {
        new_union_type.had_template = true;
    }

    new_union_type
}

#[allow(clippy::too_many_arguments)]
fn handle_atomic_standin(
    parameter_atomic: &TAtomic,
    template_result: &mut TemplateResult,
    codebase: &CodebaseMetadata,
    argument_type: &Option<&TUnion>,
    argument_offset: Option<usize>,
    argument_span: Option<Span>,
    options: StandinOptions<'_>,
    was_single: bool,
    had_template: &mut bool,
) -> Vec<TAtomic> {
    let normalized_key = if let TAtomic::Object(TObject::Named(named_object)) = parameter_atomic {
        named_object.name
    } else {
        parameter_atomic.get_id()
    };

    if let TAtomic::GenericParameter(TGenericParameter { parameter_name, defining_entity, .. }) = parameter_atomic
        && let Some(template_type) =
            template_types_contains(&template_result.template_types.clone(), parameter_name, defining_entity)
    {
        return handle_template_param_standin(
            parameter_atomic,
            normalized_key,
            template_type,
            template_result,
            codebase,
            argument_type,
            argument_offset,
            argument_span,
            options,
            had_template,
        );
    }

    if let TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic {
        parameter_name, defining_entity, ..
    })) = parameter_atomic
        && template_types_contains(&template_result.template_types.clone(), parameter_name, defining_entity).is_some()
    {
        return handle_template_param_class_standin(
            parameter_atomic,
            template_result,
            codebase,
            argument_type,
            argument_offset,
            argument_span,
            options,
            was_single,
        );
    }

    let mut matching_input_types = Vec::new();

    let mut new_appearance_depth = options.appearance_depth;

    if let Some(argument_type) = argument_type {
        if !argument_type.is_mixed() {
            matching_input_types = find_matching_atomic_types_for_template(
                parameter_atomic,
                normalized_key,
                codebase,
                argument_type,
                &mut new_appearance_depth,
            );
        } else {
            matching_input_types.push(argument_type.get_single().clone());
        }
    }

    if matching_input_types.is_empty() {
        let atomic_type = replace_atomic(
            parameter_atomic,
            template_result,
            codebase,
            None,
            argument_offset,
            argument_span,
            options.next_iteration(),
        );

        return vec![atomic_type];
    }

    let mut atomic_types = Vec::new();

    for matching_input_type in matching_input_types {
        atomic_types.push(replace_atomic(
            parameter_atomic,
            template_result,
            codebase,
            Some(matching_input_type),
            argument_offset,
            argument_span,
            options.with_appearance_depth(new_appearance_depth),
        ))
    }

    atomic_types
}

#[allow(clippy::too_many_arguments)]
fn replace_atomic(
    atomic_type: &TAtomic,
    template_result: &mut TemplateResult,
    codebase: &CodebaseMetadata,
    input_type: Option<TAtomic>,
    input_arg_offset: Option<usize>,
    input_arg_pos: Option<Span>,
    opts: StandinOptions<'_>,
) -> TAtomic {
    let mut atomic_type = atomic_type.clone();

    match &mut atomic_type {
        TAtomic::Array(array_type) => {
            match array_type {
                TArray::Keyed(keyed_data) => {
                    if let Some(known_items) = &mut keyed_data.known_items {
                        for (key, (_, item_union)) in known_items.iter_mut() {
                            let input_type_parameter =
                                if let Some(TAtomic::Array(TArray::Keyed(input_keyed_data))) = &input_type {
                                    input_keyed_data.get_known_items().and_then(|items| items.get(key)).map(|(_, t)| t)
                                } else {
                                    None
                                };

                            *item_union = self::replace(
                                item_union,
                                template_result,
                                codebase,
                                &input_type_parameter,
                                input_arg_offset,
                                input_arg_pos,
                                StandinOptions { iteration_depth: opts.iteration_depth + 1, ..opts },
                            );
                        }
                    } else if let Some(parameters) = &mut keyed_data.parameters {
                        let input_parameters =
                            if let Some(input) = &input_type { get_iterable_parameters(input, codebase) } else { None };

                        parameters.0 = Box::new(self::replace(
                            &parameters.0,
                            template_result,
                            codebase,
                            &input_parameters.as_ref().map(|(k, _)| k),
                            input_arg_offset,
                            input_arg_pos,
                            StandinOptions { iteration_depth: opts.iteration_depth + 1, ..opts },
                        ));

                        parameters.1 = Box::new(self::replace(
                            &parameters.1, // Pass &TUnion
                            template_result,
                            codebase,
                            &input_parameters.as_ref().map(|(_, v)| v),
                            input_arg_offset,
                            input_arg_pos,
                            StandinOptions { iteration_depth: opts.iteration_depth + 1, ..opts },
                        ));
                    }
                }
                TArray::List(list_data) => {
                    if let Some(known_elements) = &mut list_data.known_elements {
                        for (offset, (_, element_union_arc)) in known_elements.iter_mut() {
                            let input_type_parameter =
                                if let Some(TAtomic::Array(TArray::List(input_list_data))) = &input_type {
                                    input_list_data
                                        .get_known_elements()
                                        .and_then(|elements| elements.get(offset))
                                        .map(|(_, t)| t)
                                } else {
                                    None
                                };

                            *element_union_arc = self::replace(
                                element_union_arc,
                                template_result,
                                codebase,
                                &input_type_parameter,
                                input_arg_offset,
                                input_arg_pos,
                                StandinOptions { iteration_depth: opts.iteration_depth + 1, ..opts },
                            );
                        }
                    } else {
                        let input_param = if let Some(input) = &input_type {
                            get_iterable_value_parameter(input, codebase)
                        } else {
                            None
                        };

                        list_data.element_type = Box::new(self::replace(
                            &list_data.element_type,
                            template_result,
                            codebase,
                            &input_param.as_ref(),
                            input_arg_offset,
                            input_arg_pos,
                            StandinOptions { iteration_depth: opts.iteration_depth + 1, ..opts },
                        ));
                    }
                }
            }
        }
        TAtomic::Object(TObject::Named(named_object)) => {
            let object_name = named_object.name;
            let object_rempped_parameters = named_object.remapped_parameters;

            if let Some(type_parameters) = named_object.get_type_parameters_mut() {
                let mapped_type_parameters = if let Some(
                    object @ TAtomic::Object(TObject::Named(TNamedObject { type_parameters: Some(_), .. })),
                ) = &input_type
                {
                    Some(get_mapped_generic_type_parameters(codebase, object, &object_name, object_rempped_parameters))
                } else {
                    None
                };

                for (offset, type_param) in type_parameters.iter_mut().enumerate() {
                    let input_type_param = match &input_type {
                        Some(input_inner) => match input_inner {
                            TAtomic::Object(TObject::Named(object)) => {
                                object.get_type_parameters().and_then(|parameters| parameters.get(offset)).cloned()
                            }
                            TAtomic::Mixed(mixed) => {
                                if mixed.is_isset_from_loop() {
                                    Some(get_mixed_maybe_from_loop(true))
                                } else {
                                    Some(get_mixed())
                                }
                            }
                            _ => None,
                        },
                        _ => None,
                    };

                    let is_covariant = if let Some(class_like_metadata) = get_class_like(codebase, &object_name) {
                        matches!(class_like_metadata.template_variance.get(&offset), Some(Variance::Covariant))
                    } else {
                        false
                    };

                    *type_param = self::replace(
                        type_param,
                        template_result,
                        codebase,
                        &if let Some(mapped_type_parameters) = &mapped_type_parameters {
                            if let Some(matched) = mapped_type_parameters.get(offset) { Some(&matched.1) } else { None }
                        } else {
                            input_type_param.as_ref()
                        },
                        input_arg_offset,
                        input_arg_pos,
                        StandinOptions {
                            appearance_depth: opts.appearance_depth + if is_covariant { 0 } else { 1 },
                            iteration_depth: opts.iteration_depth + 1,
                            ..opts
                        },
                    );
                }
            }

            return atomic_type;
        }
        TAtomic::Callable(TCallable::Signature(signature)) => {
            let input_parameters = if let Some(TAtomic::Callable(TCallable::Signature(input_closure))) = &input_type {
                Some(input_closure.get_parameters())
            } else {
                None
            };

            for (offset, parameter) in signature.get_parameters_mut().iter_mut().enumerate() {
                let input_parameter_type = if let Some(input_parameters) = input_parameters {
                    input_parameters.get(offset).and_then(|input_parameter| input_parameter.get_type_signature())
                } else {
                    None
                };

                if let Some(parameter_type) = parameter.get_type_signature_mut() {
                    *parameter_type = self::replace(
                        parameter_type,
                        template_result,
                        codebase,
                        &if let Some(input_type_param) = input_parameter_type { Some(input_type_param) } else { None },
                        input_arg_offset,
                        input_arg_pos,
                        StandinOptions { add_lower_bound: !opts.add_lower_bound, ..opts },
                    );
                }
            }

            if let Some(return_type) = signature.get_return_type_mut() {
                *return_type = self::replace(
                    return_type,
                    template_result,
                    codebase,
                    &if let Some(TAtomic::Callable(TCallable::Signature(input_signature))) = &input_type {
                        input_signature.get_return_type()
                    } else {
                        None
                    },
                    input_arg_offset,
                    input_arg_pos,
                    StandinOptions { iteration_depth: opts.iteration_depth + 1, ..opts },
                );
            }

            return atomic_type;
        }
        TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType { constraint, .. })) => {
            *constraint = Box::new(replace_atomic(
                constraint,
                template_result,
                codebase,
                if let Some(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType {
                    constraint: input_constraint,
                    ..
                }))) = input_type
                {
                    Some(*input_constraint)
                } else {
                    None
                },
                input_arg_offset,
                input_arg_pos,
                opts,
            ));

            return atomic_type;
        }
        _ => (),
    }

    atomic_type.clone()
}

#[allow(clippy::too_many_arguments)]
fn handle_template_param_standin(
    atomic_type: &TAtomic,
    normalized_key: Atom,
    template_type: &TUnion,
    template_result: &mut TemplateResult,
    codebase: &CodebaseMetadata,
    input_type: &Option<&TUnion>,
    input_arg_offset: Option<usize>,
    input_arg_pos: Option<Span>,
    options: StandinOptions<'_>,
    had_template: &mut bool,
) -> Vec<TAtomic> {
    let (parameter_name, defining_entity, intersection_types, constraint) =
        if let TAtomic::GenericParameter(TGenericParameter {
            parameter_name,
            defining_entity,
            intersection_types,
            constraint,
            ..
        }) = atomic_type
        {
            (parameter_name, defining_entity, intersection_types, constraint)
        } else {
            panic!()
        };

    if let Some(calling_class) = options.calling_class
        && defining_entity == &GenericParent::ClassLike(calling_class)
    {
        return vec![atomic_type.clone()];
    }

    if template_type.get_id() == normalized_key {
        return template_type.types.clone().into_owned();
    }

    let mut replacement_type = template_type.clone();

    let parameter_name_key = *parameter_name;

    let mut new_intersection_types = vec![];

    if let Some(intersection_types) = intersection_types {
        for intersection_type in intersection_types {
            let intersection_type_union = self::replace(
                &TUnion::from_vec(vec![intersection_type.clone()]),
                template_result,
                codebase,
                input_type,
                input_arg_offset,
                input_arg_pos,
                StandinOptions { iteration_depth: options.iteration_depth + 1, ..options },
            );

            if intersection_type_union.is_single() {
                let intersection_type = intersection_type_union.get_single().clone();

                if let TAtomic::Object(TObject::Named(_)) | TAtomic::GenericParameter(_) = intersection_type {
                    new_intersection_types.push(intersection_type);
                }
            }
        }
    }

    let mut atomic_types = Vec::new();

    if replacement_type.is_mixed() && !constraint.is_mixed() {
        atomic_types.extend(constraint.types.iter().cloned());
    } else {
        expander::expand_union(
            codebase,
            &mut replacement_type,
            &TypeExpansionOptions {
                self_class: options.calling_class,
                static_class_type: if let Some(c) = options.calling_class {
                    StaticClassType::Name(c)
                } else {
                    StaticClassType::None
                },

                expand_templates: false,

                ..Default::default()
            },
        );

        if options.iteration_depth < 15 && replacement_type.has_template_types() {
            replacement_type = self::replace(
                &replacement_type,
                template_result,
                codebase,
                input_type,
                input_arg_offset,
                input_arg_pos,
                StandinOptions { iteration_depth: options.iteration_depth + 1, ..options },
            );
        }

        for replacement_atomic_type in replacement_type.types.as_ref() {
            let mut replacements_found = false;

            if let TAtomic::GenericParameter(TGenericParameter {
                defining_entity: replacement_defining_entity,
                constraint: replacement_as_type,
                ..
            }) = replacement_atomic_type
                && options
                    .calling_class
                    .is_none_or(|calling_class| replacement_defining_entity != &GenericParent::ClassLike(calling_class))
                && match options.calling_function {
                    Some(FunctionLikeIdentifier::Function(calling_function)) => {
                        replacement_defining_entity != &GenericParent::FunctionLike((*calling_function, empty_atom()))
                    }
                    Some(FunctionLikeIdentifier::Method(_, _)) => true,
                    Some(_) => {
                        panic!()
                    }
                    None => true,
                }
            {
                for nested_type_atomic in replacement_as_type.types.as_ref() {
                    replacements_found = true;
                    atomic_types.push(nested_type_atomic.clone());
                }
            }

            if !replacements_found {
                atomic_types.push(replacement_atomic_type.clone());
            }

            *had_template = true;
        }
    }

    let mut matching_input_keys: Vec<Atom> = Vec::new();

    let mut as_type = constraint.clone();

    expander::expand_union(
        codebase,
        &mut as_type,
        &TypeExpansionOptions {
            expand_templates: false,
            self_class: options.calling_class,
            static_class_type: if let Some(c) = options.calling_class {
                StaticClassType::Name(c)
            } else {
                StaticClassType::None
            },
            ..Default::default()
        },
    );

    let as_type = self::replace(
        &as_type,
        template_result,
        codebase,
        input_type,
        input_arg_offset,
        input_arg_pos,
        options.next_iteration(),
    );

    if let Some(input_type) = input_type
        && !template_result.readonly
        && (as_type.is_mixed()
            || union_comparator::can_be_contained_by(
                codebase,
                input_type,
                &as_type,
                false,
                false,
                &mut matching_input_keys,
            ))
    {
        let mut input_type = (*input_type).clone();

        if !matching_input_keys.is_empty() {
            for atomic in input_type.types.clone().as_ref() {
                if !matching_input_keys.contains(&atomic.get_id()) {
                    input_type.remove_type(atomic);
                }
            }
        }

        if !options.add_lower_bound {
            return input_type.types.into_owned();
        }

        if let Some(existing_lower_bounds) =
            if let Some(mapped_bounds) = template_result.lower_bounds.get(&parameter_name_key) {
                mapped_bounds.get(defining_entity)
            } else {
                None
            }
        {
            let mut has_matching_lower_bound = false;

            for existing_lower_bound in existing_lower_bounds {
                let existing_depth = &existing_lower_bound.appearance_depth;
                let existing_arg_offset = if existing_lower_bound.argument_offset.is_none() {
                    &input_arg_offset
                } else {
                    &existing_lower_bound.argument_offset
                };

                if existing_depth == &options.appearance_depth
                    && &input_arg_offset == existing_arg_offset
                    && existing_lower_bound.bound_type == input_type
                    && existing_lower_bound.equality_bound_classlike.is_none()
                {
                    has_matching_lower_bound = true;
                    break;
                }
            }

            if !has_matching_lower_bound {
                insert_bound_type(
                    template_result,
                    parameter_name_key,
                    defining_entity,
                    input_type,
                    options,
                    input_arg_offset,
                    input_arg_pos,
                );
            }
        } else {
            insert_bound_type(
                template_result,
                parameter_name_key,
                defining_entity,
                input_type.clone(),
                options,
                input_arg_offset,
                input_arg_pos,
            );
        }
    }

    let mut new_atomic_types = Vec::new();

    for mut atomic_type in atomic_types {
        match &mut atomic_type {
            TAtomic::Object(TObject::Named(named_object)) => {
                named_object.intersection_types =
                    if new_intersection_types.is_empty() { None } else { Some(new_intersection_types.clone()) };
            }
            TAtomic::GenericParameter(parameter) => {
                parameter.intersection_types =
                    if new_intersection_types.is_empty() { None } else { Some(new_intersection_types.clone()) };
            }
            _ => {}
        }

        new_atomic_types.push(atomic_type);
    }

    new_atomic_types
}

/// Inserts a new lower bound (`bound_type`) for a specific template parameter
/// (`param_name` defined in `defining_entity`) into the `template_result`.
///
/// This function handles adding the bound to the nested map structure within
/// `template_result.lower_bounds`. It avoids adding exact duplicates based on
/// the bound type, appearance depth, and argument offset.
///
/// # Arguments
///
/// * `template_result` - The mutable collection of template bounds being populated.
/// * `param_name` - The identifier of the template parameter (e.g., `T`).
/// * `defining_entity` - The context (class or function) where the template parameter is defined.
/// * `bound_type` - The inferred type (`TUnion`) that acts as a lower bound.
/// * `options` - Standin options providing context like appearance depth.
/// * `argument_offset` - Optional index of the argument from which this bound was inferred.
/// * `argument_span` - Optional span of the argument expression.
pub fn insert_bound_type(
    template_result: &mut TemplateResult,
    param_name: Atom,
    defining_entity: &GenericParent,
    bound_type: TUnion,
    options: StandinOptions,
    argument_offset: Option<usize>,
    argument_span: Option<Span>,
) {
    let bounds = template_result.lower_bounds.entry(param_name).or_default().entry(*defining_entity).or_default();

    if bounds.iter().any(|existing_bound| {
        existing_bound.bound_type == bound_type
            && existing_bound.appearance_depth == options.appearance_depth
            && existing_bound.argument_offset == argument_offset
    }) {
        return; // Exact duplicate found, do nothing.
    }

    bounds.push(TemplateBound {
        bound_type,
        appearance_depth: options.appearance_depth,
        argument_offset,
        equality_bound_classlike: None,
        span: argument_span,
    });
}

#[allow(clippy::too_many_arguments)]
fn handle_template_param_class_standin(
    atomic_type: &TAtomic,
    template_result: &mut TemplateResult,
    codebase: &CodebaseMetadata,
    input_type: &Option<&TUnion>,
    input_argument_offset: Option<usize>,
    input_argument_span: Option<Span>,
    options: StandinOptions<'_>,
    was_single: bool,
) -> Vec<TAtomic> {
    if let TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic {
        kind,
        parameter_name,
        defining_entity,
        constraint,
    })) = atomic_type
    {
        let mut atomic_type_as = *constraint.clone();
        if let Some(calling_class) = options.calling_class
            && defining_entity == &GenericParent::ClassLike(calling_class)
        {
            return vec![atomic_type.clone()];
        }

        let mut atomic_types = vec![];

        if let Some(input_type) = if let Some(input_type) = input_type {
            if !template_result.readonly { Some(input_type) } else { None }
        } else {
            None
        } {
            let mut valid_input_atomic_types = vec![];

            for input_atomic_type in input_type.types.as_ref() {
                if let TAtomic::Scalar(TScalar::ClassLikeString(input_class_string)) = input_atomic_type {
                    let valid_input_type = match input_class_string {
                        TClassLikeString::Generic { parameter_name, defining_entity, constraint, .. } => {
                            TAtomic::GenericParameter(TGenericParameter {
                                parameter_name: *parameter_name,
                                constraint: Box::new(wrap_atomic(*constraint.clone())),
                                defining_entity: *defining_entity,
                                intersection_types: None,
                            })
                        }
                        TClassLikeString::Literal { value } => {
                            TAtomic::Object(TObject::Named(TNamedObject::new(*value)))
                        }
                        TClassLikeString::OfType { constraint, .. } => (**constraint).clone(),
                        _ => {
                            continue;
                        }
                    };

                    valid_input_atomic_types.push(valid_input_type);
                }
            }

            let generic_param = if !valid_input_atomic_types.is_empty() {
                Some(TUnion::from_vec(valid_input_atomic_types))
            } else if was_single {
                Some(get_mixed())
            } else {
                None
            };

            let as_type_union = self::replace(
                &TUnion::from_vec(vec![atomic_type_as.clone()]),
                template_result,
                codebase,
                &generic_param.as_ref(),
                input_argument_offset,
                input_argument_span,
                options.next_iteration(),
            );

            atomic_type_as = if as_type_union.is_single() {
                as_type_union.get_single().clone()
            } else {
                TAtomic::Object(TObject::Any)
            };

            if let Some(generic_param) = generic_param {
                if let Some(template_bounds) = template_result
                    .lower_bounds
                    .get_mut(parameter_name)
                    .unwrap_or(&mut HashMap::default())
                    .get_mut(defining_entity)
                {
                    *template_bounds = vec![TemplateBound {
                        bound_type: add_union_type(
                            generic_param,
                            &get_most_specific_type_from_bounds(template_bounds, codebase),
                            codebase,
                            false,
                        ),
                        appearance_depth: options.appearance_depth,
                        argument_offset: input_argument_offset,
                        span: input_argument_span,
                        equality_bound_classlike: None,
                    }]
                } else {
                    template_result.lower_bounds.entry(*parameter_name).or_default().insert(
                        *defining_entity,
                        vec![TemplateBound {
                            bound_type: generic_param,
                            appearance_depth: options.appearance_depth,
                            argument_offset: input_argument_offset,
                            span: input_argument_span,
                            equality_bound_classlike: None,
                        }],
                    );
                }
            }
        } else {
            let template_type = template_result
                .template_types
                .get(parameter_name)
                .unwrap()
                .iter()
                .filter(|(e, _)| e == defining_entity)
                .map(|(_, v)| v)
                .next()
                .unwrap();

            for template_atomic_type in template_type.types.as_ref() {
                if let TAtomic::Object(_) = &template_atomic_type {
                    atomic_types.push(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType {
                        kind: *kind,
                        constraint: Box::new(template_atomic_type.clone()),
                    })));
                }
            }
        }

        if atomic_types.is_empty() {
            if let TAtomic::GenericParameter(parameter) = &atomic_type_as {
                atomic_types.push(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic {
                    kind: *kind,
                    parameter_name: parameter.parameter_name,
                    defining_entity: parameter.defining_entity,
                    constraint: Box::new(atomic_type_as.clone()),
                })));
            } else {
                atomic_types.push(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType {
                    kind: *kind,
                    constraint: Box::new(atomic_type_as.clone()),
                })));
            }
        }

        atomic_types
    } else {
        panic!();
    }
}

pub fn get_actual_type_from_literal(name: &Atom, codebase: &CodebaseMetadata) -> Vec<TAtomic> {
    if class_like_exists(codebase, name) {
        vec![TAtomic::Object(TObject::Named(TNamedObject::new(*name)))]
    } else {
        vec![]
    }
}

fn template_types_contains<'a>(
    template_types: &'a IndexMap<Atom, Vec<(GenericParent, TUnion)>, RandomState>,
    parameter_name: &Atom,
    defining_entity: &GenericParent,
) -> Option<&'a TUnion> {
    if let Some(mapped_classes) = template_types.get(parameter_name) {
        return mapped_classes.iter().filter(|(e, _)| e == defining_entity).map(|(_, v)| v).next();
    }

    None
}

fn find_matching_atomic_types_for_template(
    base_type: &TAtomic,
    normalized_key: Atom,
    codebase: &CodebaseMetadata,
    input_type: &TUnion,
    depth: &mut usize,
) -> Vec<TAtomic> {
    let mut matching_atomic_types = Vec::new();

    for atomic_input_type in input_type.types.as_ref() {
        match (atomic_input_type, base_type) {
            (TAtomic::Callable(TCallable::Signature(_)), TAtomic::Callable(TCallable::Signature(_))) => {
                matching_atomic_types.push(atomic_input_type.clone());
                continue;
            }
            (TAtomic::Array(_), traversable_object)
                if !traversable_object.is_array() && traversable_object.is_array_or_traversable(codebase) =>
            {
                matching_atomic_types.push(atomic_input_type.clone());
                continue;
            }
            (
                TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Literal { value: atomic_class_name })),
                TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType { constraint, .. })),
            ) => {
                if let TAtomic::Object(TObject::Named(constraint_object)) = &**constraint {
                    let base_as_value = &constraint_object.name;

                    if let Some(class_metadata) = get_class_like(codebase, atomic_class_name)
                        && let Some(extended_parameters) =
                            class_metadata.template_extended_parameters.get(base_as_value)
                    {
                        *depth += 1;

                        let constraint_object =
                            TAtomic::Object(TObject::Named(TNamedObject::new(*base_as_value).with_type_parameters(
                                Some(extended_parameters.clone().into_iter().map(|(_, v)| v).collect::<Vec<_>>()),
                            )));

                        matching_atomic_types.push(TAtomic::Scalar(TScalar::ClassLikeString(
                            TClassLikeString::OfType {
                                kind: TClassLikeStringKind::Class,
                                constraint: Box::new(constraint_object),
                            },
                        )));
                        continue;
                    }
                }
            }
            (TAtomic::Object(TObject::Named(input_object)), TAtomic::Object(TObject::Named(base_object))) => {
                let input_name = input_object.get_name();
                let base_name = base_object.get_name();
                let input_type_parameters = input_object.get_type_parameters();

                if input_name == base_name {
                    matching_atomic_types.push(atomic_input_type.clone());
                    continue;
                }

                let class_metadata = if let Some(metadata) = get_class_like(codebase, &input_name) {
                    metadata
                } else {
                    matching_atomic_types.push(TAtomic::Object(TObject::Any));
                    continue;
                };

                if input_type_parameters.is_some() && class_metadata.has_template_extended_parameter(&base_name) {
                    matching_atomic_types.push(atomic_input_type.clone());
                    continue;
                }

                if let Some(extended_parameters) = class_metadata.template_extended_parameters.get(&base_name) {
                    matching_atomic_types.push(TAtomic::Object(TObject::Named(
                        TNamedObject::new(input_name).with_type_parameters(Some(
                            extended_parameters.clone().into_iter().map(|(_, v)| v).collect::<Vec<TUnion>>(),
                        )),
                    )));
                    continue;
                }
            }
            (
                TAtomic::GenericParameter(TGenericParameter {
                    parameter_name: input_name,
                    defining_entity: input_defining_entity,
                    constraint: input_as_type,
                    ..
                }),
                TAtomic::GenericParameter(TGenericParameter {
                    parameter_name: base_name,
                    defining_entity: base_defining_entity,
                    ..
                }),
            ) => {
                if input_name == base_name && input_defining_entity == base_defining_entity {
                    matching_atomic_types.push(atomic_input_type.clone());
                    continue;
                }

                matching_atomic_types.extend(find_matching_atomic_types_for_template(
                    base_type,
                    normalized_key,
                    codebase,
                    input_as_type,
                    depth,
                ));
            }
            (TAtomic::GenericParameter(TGenericParameter { constraint, .. }), _) => {
                matching_atomic_types.extend(find_matching_atomic_types_for_template(
                    base_type,
                    normalized_key,
                    codebase,
                    constraint,
                    depth,
                ));
            }
            (TAtomic::Variable(input_name), TAtomic::Variable(base_name)) => {
                if input_name == base_name {
                    matching_atomic_types.push(atomic_input_type.clone());
                    continue;
                }

                // todo we can probably do better here
                matching_atomic_types.push(TAtomic::Mixed(TMixed::new()));
            }
            (TAtomic::Variable { .. }, _) => {
                // todo we can probably do better here
                matching_atomic_types.push(TAtomic::Mixed(TMixed::new()));
            }
            _ => {
                let input_key = &if let TAtomic::Object(TObject::Named(o)) = atomic_input_type {
                    o.name
                } else {
                    atomic_input_type.get_id()
                };

                if *input_key == normalized_key {
                    matching_atomic_types.push(atomic_input_type.clone());
                    continue;
                }
            }
        }
    }
    matching_atomic_types
}

pub fn get_mapped_generic_type_parameters(
    codebase: &CodebaseMetadata,
    input_type_part: &TAtomic,
    container_name: &Atom,
    container_remapped_parameters: bool,
) -> Vec<(Option<usize>, TUnion)> {
    let mut input_type_parameters = match input_type_part {
        TAtomic::Object(TObject::Named(named_object)) => named_object
            .get_type_parameters()
            .unwrap_or_default()
            .iter()
            .enumerate()
            .map(|(k, v)| (Some(k), v.clone()))
            .collect::<Vec<_>>(),
        _ => {
            return vec![];
        }
    };

    let input_name = match input_type_part {
        TAtomic::Object(TObject::Named(o)) => o.name,
        _ => {
            return vec![];
        }
    };

    let Some(input_class_metadata) = get_class_like(codebase, &input_name) else {
        return vec![];
    };

    if input_name == *container_name {
        return input_type_parameters;
    }

    let input_template_types = &input_class_metadata.template_types;

    let mut i = 0;
    let mut replacement_templates = IndexMap::with_hasher(RandomState::new());
    if matches!(input_type_part, TAtomic::Object(TObject::Named(o)) if !o.remapped_parameters)
        && !container_remapped_parameters
    {
        for (template_name, _) in input_template_types {
            if let Some(input_type) = input_type_parameters.get(i) {
                replacement_templates
                    .entry(*template_name)
                    .or_insert_with(HashMap::default)
                    .insert(GenericParent::ClassLike(input_name), input_type.clone().1);

                i += 1;
            } else {
                break;
            }
        }
    }

    if let Some(parameters) = input_class_metadata.template_extended_parameters.get(container_name) {
        let mut new_input_parameters = Vec::new();

        for (_, extended_input_parameter) in parameters {
            let mut mapped_input_offset = None;
            let mut new_input_parameter = None;

            for extended_input_parameter_type in extended_input_parameter.types.as_ref() {
                let extended_input_parameter_types = get_extended_templated_types(
                    extended_input_parameter_type,
                    &input_class_metadata.template_extended_parameters,
                );

                let mut candidate_parameter_type: Option<_> = None;

                if let Some(TAtomic::GenericParameter(parameter)) = extended_input_parameter_types.first()
                    && let Some((old_parameters_offset, defining_classes)) =
                        input_class_metadata.get_template_type_with_index(&parameter.parameter_name)
                    && defining_classes.iter().any(|(e, _)| parameter.defining_entity == *e)
                {
                    let candidate_parameter_type_inner =
                        input_type_parameters.get(old_parameters_offset).unwrap_or(&(None, get_mixed())).clone().1;

                    mapped_input_offset = Some(old_parameters_offset);
                    candidate_parameter_type = Some(candidate_parameter_type_inner);
                }

                let mut candidate_parameter_type =
                    candidate_parameter_type.unwrap_or(wrap_atomic(extended_input_parameter_type.clone()));

                candidate_parameter_type.from_template_default = true;

                new_input_parameter = if let Some(new_input_param) = new_input_parameter {
                    Some(add_union_type(new_input_param, &candidate_parameter_type, codebase, true))
                } else {
                    Some(candidate_parameter_type.clone())
                };
            }

            if let Some(new_input_parameter) = new_input_parameter {
                new_input_parameters.push((
                    mapped_input_offset,
                    inferred_type_replacer::replace(
                        &new_input_parameter,
                        &TemplateResult::new(IndexMap::with_hasher(RandomState::new()), replacement_templates.clone()),
                        codebase,
                    ),
                ));
            } else {
                new_input_parameters.push((mapped_input_offset, get_mixed()));
            }
        }

        input_type_parameters = new_input_parameters
            .into_iter()
            .map(|mut v| {
                expander::expand_union(codebase, &mut v.1, &TypeExpansionOptions::default());

                v
            })
            .collect::<Vec<_>>();
    }

    input_type_parameters
}

pub fn get_extended_templated_types<'a>(
    atomic_type: &'a TAtomic,
    extends: &'a AtomMap<IndexMap<Atom, TUnion, RandomState>>,
) -> Vec<&'a TAtomic> {
    let mut extra_added_types = Vec::new();

    if let TAtomic::GenericParameter(TGenericParameter {
        parameter_name,
        defining_entity: GenericParent::ClassLike(defining_class),
        ..
    }) = atomic_type
    {
        if let Some(defining_parameters) = extends.get(defining_class) {
            if let Some(extended_parameter) = defining_parameters.get(parameter_name) {
                for extended_atomic_type in extended_parameter.types.as_ref() {
                    if let TAtomic::GenericParameter(_) = extended_atomic_type {
                        extra_added_types.extend(get_extended_templated_types(extended_atomic_type, extends));
                    } else {
                        extra_added_types.push(extended_atomic_type);
                    }
                }
            } else {
                extra_added_types.push(atomic_type);
            }
        } else {
            extra_added_types.push(atomic_type);
        }
    }

    extra_added_types
}

pub(crate) fn get_root_template_type(
    lower_bounds: &IndexMap<Atom, HashMap<GenericParent, Vec<TemplateBound>>, RandomState>,
    parameter_name: &Atom,
    defining_entity: &GenericParent,
    mut visited_entities: HashSet<GenericParent>,
    codebase: &CodebaseMetadata,
) -> Option<TUnion> {
    if visited_entities.contains(defining_entity) {
        return None;
    }

    if let Some(mapped) = lower_bounds.get(parameter_name)
        && let Some(bounds) = mapped.get(defining_entity)
    {
        let mapped_type = get_most_specific_type_from_bounds(bounds, codebase);

        if !mapped_type.is_single() {
            return Some(mapped_type);
        }

        let first_template = &mapped_type.get_single();

        if let TAtomic::GenericParameter(TGenericParameter { parameter_name, defining_entity, .. }) = first_template {
            visited_entities.insert(*defining_entity);

            return Some(
                get_root_template_type(lower_bounds, parameter_name, defining_entity, visited_entities, codebase)
                    .unwrap_or(mapped_type),
            );
        }

        return Some(mapped_type);
    }

    None
}

pub fn get_most_specific_type_from_bounds(lower_bounds: &[TemplateBound], codebase: &CodebaseMetadata) -> TUnion {
    let relevant_bounds = get_relevant_bounds(lower_bounds);

    if relevant_bounds.is_empty() {
        return get_mixed();
    }

    if relevant_bounds.len() == 1 {
        return relevant_bounds[0].bound_type.clone();
    }

    let mut specific_type = relevant_bounds[0].bound_type.clone();

    for bound in relevant_bounds {
        specific_type = add_union_type(specific_type, &bound.bound_type, codebase, false);
    }

    specific_type
}

pub fn get_relevant_bounds(lower_bounds: &[TemplateBound]) -> Vec<&TemplateBound> {
    let mut lower_bounds = lower_bounds.iter().collect::<Vec<_>>();

    if lower_bounds.len() == 1 {
        return lower_bounds;
    }

    lower_bounds.sort_by(|a, b| a.appearance_depth.partial_cmp(&b.appearance_depth).unwrap_or(Ordering::Equal));

    let mut current_depth = None;
    let mut had_invariant = false;
    let mut last_argument_offset = None;

    let mut applicable_bounds = vec![];

    for template_bound in lower_bounds {
        if let Some(inner) = current_depth {
            if inner != template_bound.appearance_depth && !applicable_bounds.is_empty() {
                if !had_invariant || last_argument_offset == template_bound.argument_offset {
                    // escape switches when matching on invariant generic parameters
                    // and when matching
                    break;
                }

                current_depth = Some(template_bound.appearance_depth);
            }
        } else {
            current_depth = Some(template_bound.appearance_depth);
        }

        had_invariant = if had_invariant { true } else { template_bound.equality_bound_classlike.is_some() };

        applicable_bounds.push(template_bound);

        last_argument_offset = template_bound.argument_offset;
    }

    applicable_bounds
}
