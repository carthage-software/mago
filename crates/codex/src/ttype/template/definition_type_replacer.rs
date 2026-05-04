use std::sync::Arc;

use foldhash::HashMap;
use foldhash::fast::RandomState;
use indexmap::IndexMap;

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_atom::empty_atom;
use mago_span::Span;

use crate::identifier::function_like::FunctionLikeIdentifier;
use crate::metadata::CodebaseMetadata;
use crate::misc::GenericParent;
use crate::ttype::TType;
use crate::ttype::add_union_type;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::generic::TGenericParameter;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::combiner;
use crate::ttype::combiner::CombinerOptions;
use crate::ttype::expander;
use crate::ttype::expander::StaticClassType;
use crate::ttype::expander::TypeExpansionOptions;
use crate::ttype::get_mixed;
use crate::ttype::template::GenericTemplate;
use crate::ttype::template::TemplateBound;
use crate::ttype::template::TemplateResult;
use crate::ttype::template::inferred_type_replacer;
use crate::ttype::template::variance::Variance;
use crate::ttype::union::TUnion;
use crate::ttype::wrap_atomic;

/// Knobs for [`replace`].
///
/// `calling_class` / `calling_function` identify the scope of the substitution
/// so we don't recursively widen self-referential templates. `add_lower_bound`
/// is preserved from the legacy walker for the templates-on-templates path.
/// `iteration_depth` guards against runaway recursion; `appearance_depth`
/// tracks how deeply nested we are (invariant params bump this so the bound
/// resolver can prefer the shallowest contribution).
#[derive(Copy, Clone, Debug)]
pub struct DefinitionReplacementOptions<'fn_id> {
    pub calling_class: Option<Atom>,
    pub calling_function: Option<&'fn_id FunctionLikeIdentifier>,
    pub add_lower_bound: bool,
    pub iteration_depth: usize,
    pub appearance_depth: usize,
}

impl Default for DefinitionReplacementOptions<'_> {
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

impl DefinitionReplacementOptions<'_> {
    #[must_use]
    pub fn next_iteration(&self) -> Self {
        Self { iteration_depth: self.iteration_depth + 1, ..*self }
    }
}

/// Walks `parameter_type` and substitutes any `TAtomic::GenericParameter`
/// references with their definitions in `template_result.template_types`.
///
/// This is the substitution-only entry point used by class-extension
/// validation. It does not perform argument-driven inference (see
/// [`crate::ttype::template::inferred_type_replacer::replace`] for the
/// substitution counterpart driven by inferred bounds, and the analyzer's
/// `template_inference` module for argument-driven bound inference).
pub fn replace(
    parameter_type: &TUnion,
    template_result: &mut TemplateResult,
    codebase: &CodebaseMetadata,
    options: DefinitionReplacementOptions<'_>,
) -> TUnion {
    let original_parameter_atomics = parameter_type.types.to_vec();
    let mut new_parameter_atomics = Vec::with_capacity(original_parameter_atomics.len());

    let mut had_template = false;
    for atomic_type in &original_parameter_atomics {
        new_parameter_atomics.extend(handle_atomic_substitution(
            atomic_type,
            template_result,
            codebase,
            options,
            original_parameter_atomics.len() == 1,
            &mut had_template,
        ));
    }

    if new_parameter_atomics.is_empty() {
        return parameter_type.clone();
    }

    let mut new_union_type = TUnion::from_vec(if new_parameter_atomics.len() > 1 {
        combiner::combine(new_parameter_atomics, codebase, combiner::CombinerOptions::default())
    } else {
        new_parameter_atomics
    });

    new_union_type.set_ignore_falsable_issues(parameter_type.ignore_falsable_issues());

    if had_template {
        new_union_type.set_had_template(true);
    }

    new_union_type
}

fn handle_atomic_substitution(
    parameter_atomic: &TAtomic,
    template_result: &mut TemplateResult,
    codebase: &CodebaseMetadata,
    options: DefinitionReplacementOptions<'_>,
    _was_single: bool,
    had_template: &mut bool,
) -> Vec<TAtomic> {
    let normalized_key = if let TAtomic::Object(TObject::Named(named_object)) = parameter_atomic {
        named_object.name
    } else {
        parameter_atomic.get_id()
    };

    if let TAtomic::GenericParameter(TGenericParameter { parameter_name, defining_entity, .. }) = parameter_atomic
        && let Some(template_type) =
            template_types_contains(&template_result.template_types, *parameter_name, defining_entity).cloned()
    {
        return handle_template_param_substitution(
            parameter_atomic,
            normalized_key,
            &template_type,
            template_result,
            codebase,
            options,
            had_template,
        );
    }

    if let TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic {
        parameter_name, defining_entity, ..
    })) = parameter_atomic
        && template_types_contains(&template_result.template_types, *parameter_name, defining_entity).is_some()
    {
        return handle_template_param_class_substitution(parameter_atomic, template_result, options);
    }

    vec![replace_atomic(parameter_atomic, template_result, codebase, options.next_iteration())]
}

fn replace_atomic(
    atomic_type: &TAtomic,
    template_result: &mut TemplateResult,
    codebase: &CodebaseMetadata,
    opts: DefinitionReplacementOptions<'_>,
) -> TAtomic {
    let mut atomic_type = atomic_type.clone();
    let next_opts = DefinitionReplacementOptions { iteration_depth: opts.iteration_depth + 1, ..opts };

    match &mut atomic_type {
        TAtomic::Array(array_type) => match array_type {
            TArray::Keyed(keyed_data) => {
                if let Some(known_items) = &mut keyed_data.known_items {
                    for (_, item_union) in known_items.values_mut() {
                        *item_union = self::replace(item_union, template_result, codebase, next_opts);
                    }
                } else if let Some(parameters) = &mut keyed_data.parameters {
                    *Arc::make_mut(&mut parameters.0) =
                        self::replace(&parameters.0, template_result, codebase, next_opts);
                    *Arc::make_mut(&mut parameters.1) =
                        self::replace(&parameters.1, template_result, codebase, next_opts);
                }
            }
            TArray::List(list_data) => {
                if let Some(known_elements) = &mut list_data.known_elements {
                    for (_, element_union_arc) in known_elements.values_mut() {
                        *element_union_arc = self::replace(element_union_arc, template_result, codebase, next_opts);
                    }
                } else {
                    *Arc::make_mut(&mut list_data.element_type) =
                        self::replace(&list_data.element_type, template_result, codebase, next_opts);
                }
            }
        },
        TAtomic::Object(TObject::Named(named_object)) => {
            let object_name = named_object.name;

            if let Some(type_parameters) = named_object.get_type_parameters_mut() {
                for (offset, type_param) in type_parameters.iter_mut().enumerate() {
                    let is_covariant = if let Some(class_like_metadata) = codebase.get_class_like(&object_name) {
                        matches!(class_like_metadata.template_variance.get(offset), Some(Variance::Covariant))
                    } else {
                        false
                    };

                    *type_param = self::replace(
                        type_param,
                        template_result,
                        codebase,
                        DefinitionReplacementOptions {
                            appearance_depth: opts.appearance_depth + usize::from(!is_covariant),
                            iteration_depth: opts.iteration_depth + 1,
                            ..opts
                        },
                    );
                }
            }

            return atomic_type;
        }
        TAtomic::Callable(TCallable::Signature(signature)) => {
            for parameter in signature.get_parameters_mut().iter_mut() {
                if let Some(parameter_type) = parameter.get_type_signature_mut() {
                    *parameter_type = self::replace(
                        parameter_type,
                        template_result,
                        codebase,
                        DefinitionReplacementOptions { add_lower_bound: !opts.add_lower_bound, ..opts },
                    );
                }
            }

            if let Some(return_type) = signature.get_return_type_mut() {
                *return_type = self::replace(return_type, template_result, codebase, next_opts);
            }

            return atomic_type;
        }
        TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType { constraint, .. })) => {
            *Arc::make_mut(constraint) = replace_atomic(constraint, template_result, codebase, opts);

            return atomic_type;
        }
        _ => (),
    }

    atomic_type
}

fn handle_template_param_substitution(
    atomic_type: &TAtomic,
    normalized_key: Atom,
    template_type: &TUnion,
    template_result: &mut TemplateResult,
    codebase: &CodebaseMetadata,
    options: DefinitionReplacementOptions<'_>,
    had_template: &mut bool,
) -> Vec<TAtomic> {
    #[allow(clippy::unreachable)]
    let TAtomic::GenericParameter(TGenericParameter { defining_entity, intersection_types, constraint, .. }) =
        atomic_type
    else {
        unreachable!("handle_template_param_substitution called with non-GenericParameter atomic type: {atomic_type:?}",)
    };

    if let Some(calling_class) = options.calling_class
        && defining_entity == &GenericParent::ClassLike(calling_class)
    {
        return vec![atomic_type.clone()];
    }

    if template_type.get_id() == normalized_key {
        return template_type.types.to_vec();
    }

    let mut replacement_type = template_type.clone();

    let mut new_intersection_types = vec![];

    if let Some(intersection_types) = intersection_types {
        for intersection_type in intersection_types {
            let intersection_type_union = self::replace(
                &TUnion::from_vec(vec![intersection_type.clone()]),
                template_result,
                codebase,
                DefinitionReplacementOptions { iteration_depth: options.iteration_depth + 1, ..options },
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
                DefinitionReplacementOptions { iteration_depth: options.iteration_depth + 1, ..options },
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
                    Some(FunctionLikeIdentifier::Closure(_, _)) => true,
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
/// * `options` - Replacement options providing context like appearance depth.
/// * `argument_offset` - Optional index of the argument from which this bound was inferred.
/// * `argument_span` - Optional span of the argument expression.
pub fn insert_bound_type(
    template_result: &mut TemplateResult,
    param_name: Atom,
    defining_entity: &GenericParent,
    bound_type: TUnion,
    options: DefinitionReplacementOptions,
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

fn handle_template_param_class_substitution(
    atomic_type: &TAtomic,
    template_result: &TemplateResult,
    options: DefinitionReplacementOptions<'_>,
) -> Vec<TAtomic> {
    #[allow(clippy::unreachable)]
    let TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic {
        kind,
        parameter_name,
        defining_entity,
        constraint,
    })) = atomic_type
    else {
        unreachable!(
            "handle_template_param_class_substitution called with non-ClassLikeString::Generic atomic type: {atomic_type:?}",
        )
    };

    let atomic_type_as = constraint.as_ref().clone();
    if let Some(calling_class) = options.calling_class
        && defining_entity == &GenericParent::ClassLike(calling_class)
    {
        return vec![atomic_type.clone()];
    }

    let mut atomic_types = vec![];

    let Some(template_type) = template_result
        .template_types
        .get(parameter_name)
        .and_then(|entries| entries.iter().find(|t| &t.defining_entity == defining_entity))
        .map(|t| &t.constraint)
    else {
        return atomic_types;
    };

    for template_atomic_type in template_type.types.as_ref() {
        if let TAtomic::Object(_) = &template_atomic_type {
            atomic_types.push(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType {
                kind: *kind,
                constraint: Arc::new(template_atomic_type.clone()),
            })));
        }
    }

    if atomic_types.is_empty() {
        if let TAtomic::GenericParameter(parameter) = &atomic_type_as {
            atomic_types.push(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::Generic {
                kind: *kind,
                parameter_name: parameter.parameter_name,
                defining_entity: parameter.defining_entity,
                constraint: Arc::new(atomic_type_as.clone()),
            })));
        } else {
            atomic_types.push(TAtomic::Scalar(TScalar::ClassLikeString(TClassLikeString::OfType {
                kind: *kind,
                constraint: Arc::new(atomic_type_as),
            })));
        }
    }

    atomic_types
}

fn template_types_contains<'tt>(
    template_types: &'tt IndexMap<Atom, Vec<GenericTemplate>, RandomState>,
    parameter_name: Atom,
    defining_entity: &GenericParent,
) -> Option<&'tt TUnion> {
    template_types.get(&parameter_name).and_then(|mapped_classes| {
        mapped_classes
            .iter()
            .find(|template| &template.defining_entity == defining_entity)
            .map(|template| &template.constraint)
    })
}

#[must_use]
pub fn get_mapped_generic_type_parameters(
    codebase: &CodebaseMetadata,
    input_type_part: &TAtomic,
    container_name: Atom,
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

    let Some(input_class_metadata) = codebase.get_class_like(&input_name) else {
        return vec![];
    };

    if input_name == container_name {
        return input_type_parameters;
    }

    let input_template_types = &input_class_metadata.template_types;

    let mut i = 0;
    let mut replacement_templates: HashMap<Atom, HashMap<GenericParent, TUnion>> = HashMap::default();
    if matches!(input_type_part, TAtomic::Object(TObject::Named(o)) if !o.remapped_parameters)
        && !container_remapped_parameters
    {
        for (template_name, _) in input_template_types {
            if let Some(input_type) = input_type_parameters.get(i) {
                replacement_templates
                    .entry(*template_name)
                    .or_default()
                    .insert(GenericParent::ClassLike(input_name), input_type.clone().1);

                i += 1;
            } else {
                break;
            }
        }
    }

    if let Some(parameters) = input_class_metadata.template_extended_parameters.get(&container_name) {
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
                    && let Some((old_parameters_offset, GenericTemplate { defining_entity, .. })) =
                        input_class_metadata.get_template_type_with_index(parameter.parameter_name)
                    && parameter.defining_entity == *defining_entity
                {
                    let candidate_parameter_type_inner =
                        input_type_parameters.get(old_parameters_offset).unwrap_or(&(None, get_mixed())).clone().1;

                    mapped_input_offset = Some(old_parameters_offset);
                    candidate_parameter_type = Some(candidate_parameter_type_inner);
                }

                let mut candidate_parameter_type =
                    candidate_parameter_type.unwrap_or(wrap_atomic(extended_input_parameter_type.clone()));

                candidate_parameter_type.set_from_template_default(true);

                new_input_parameter = if let Some(new_input_param) = new_input_parameter {
                    Some(add_union_type(
                        new_input_param,
                        &candidate_parameter_type,
                        codebase,
                        CombinerOptions::default().with_overwrite_empty_array(),
                    ))
                } else {
                    Some(candidate_parameter_type.clone())
                };
            }

            if let Some(new_input_parameter) = new_input_parameter {
                new_input_parameters.push((
                    mapped_input_offset,
                    inferred_type_replacer::replace(
                        &new_input_parameter,
                        &TemplateResult::new(
                            IndexMap::with_hasher(RandomState::default()),
                            replacement_templates.clone(),
                        ),
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

#[must_use]
pub fn get_extended_templated_types<'ty>(
    atomic_type: &'ty TAtomic,
    extends: &'ty AtomMap<IndexMap<Atom, TUnion, RandomState>>,
) -> Vec<&'ty TAtomic> {
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
