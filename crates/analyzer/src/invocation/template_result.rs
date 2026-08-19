use mago_allocator::Arena;

use foldhash::fast::RandomState;
use indexmap::IndexMap;

use mago_word::Word;

use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::misc::GenericParent;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::get_specialized_template_type;
use mago_codex::ttype::template::GenericTemplate;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::template::bounds::get_most_specific_type_from_bounds;

use crate::context::Context;
use crate::invocation::Invocation;
use crate::invocation::InvocationTarget;
use crate::invocation::MethodTargetContext;
use crate::invocation::template_inference::infer_templates_for_method_call;
use crate::utils::template::get_template_types_for_class_member;

/// Populates the `TemplateResult` with template types from the invocation target.
///
/// This function extracts template types from the metadata of the invocation target,
/// including any method context if applicable. It also adds lower bounds for
/// template types based on the class-like metadata and the type parameters of the class.
///
/// # Arguments
///
/// * `invocation` - The invocation whose target metadata is used to populate the template result.
/// * `template_result` - The mutable `TemplateResult` to be populated with template types and bounds.
///
/// # Note
///
/// This function assumes that the `TemplateResult` is initially empty and will be populated with
/// template types and bounds derived from the invocation's target metadata.
pub fn populate_template_result_from_invocation<'ctx, 'arena, A>(
    context: &Context<'ctx, 'arena, A>,
    invocation: &Invocation<'ctx, '_, 'arena>,
    template_result: &mut TemplateResult,
) where
    A: Arena,
{
    let InvocationTarget::FunctionLike { metadata, method_context, .. } = &invocation.target else {
        return;
    };

    for (template_name, template_details) in &metadata.template_types {
        template_result.template_types.entry(*template_name).or_default().push(template_details.clone());
    }

    let Some(method_metadata) = &metadata.method_metadata else {
        return;
    };

    let Some(method_context) = method_context else {
        return;
    };

    if method_metadata.is_static {
        let Some(identifier) = method_context.declaring_method_id else {
            return;
        };

        let Some(declaring_class_metadata) = context.codebase.get_class_like(identifier.get_class_name().as_bytes())
        else {
            return;
        };

        for (template_name, template_details) in &declaring_class_metadata.template_types {
            if !template_result.template_types.contains_key(template_name) {
                template_result.template_types.entry(*template_name).or_default().push(template_details.clone());
            }
        }

        if declaring_class_metadata.name != method_context.class_like_metadata.name {
            for (template_name, _) in &declaring_class_metadata.template_types {
                let template_type = get_specialized_template_type(
                    context.codebase,
                    *template_name,
                    declaring_class_metadata.name,
                    method_context.class_like_metadata,
                    None,
                );

                if let Some(template_type) = template_type {
                    template_result.add_lower_bound(
                        *template_name,
                        GenericParent::ClassLike(declaring_class_metadata.name),
                        template_type,
                    );
                }
            }
        }

        if let Some(instance_type) = get_named_static_class_type(&method_context.class_type)
            && !instance_type.name.as_bytes().eq_ignore_ascii_case(declaring_class_metadata.original_name.as_bytes())
            && let Some(calling_class_metadata) = context.codebase.get_class_like(instance_type.name.as_bytes())
        {
            for (template_name, _) in &declaring_class_metadata.template_types {
                if template_result.lower_bounds.get(template_name).is_some_and(|m| !m.is_empty()) {
                    continue;
                }

                let template_type = get_specialized_template_type(
                    context.codebase,
                    *template_name,
                    declaring_class_metadata.name,
                    calling_class_metadata,
                    instance_type.type_parameters.as_deref(),
                );

                if let Some(template_type) = template_type {
                    template_result.add_lower_bound(
                        *template_name,
                        GenericParent::ClassLike(declaring_class_metadata.name),
                        template_type,
                    );
                }
            }
        }

        return;
    }

    for (template_name, template_details) in &method_context.class_like_metadata.template_types {
        if !template_result.template_types.contains_key(template_name) {
            template_result.template_types.entry(*template_name).or_default().push(template_details.clone());
        }
    }

    // For `@mixin`-resolved methods, `class_type` is the receiver while the type
    // parameters of `class_like_metadata` live on the mixin object.
    let instance_type = if let Some(declaring_object) = &method_context.declaring_object_type {
        declaring_object
    } else if let StaticClassType::Object(TObject::Named(instance_type)) = &method_context.class_type {
        instance_type
    } else {
        return;
    };

    if let Some(type_parameters) = &instance_type.type_parameters {
        for (template_index, template_type) in type_parameters.iter().enumerate() {
            let Some(template_name) = method_context
                .class_like_metadata
                .template_types
                .iter()
                .enumerate()
                .find_map(|(index, (name, _))| if index == template_index { Some(*name) } else { None })
            else {
                break;
            };

            template_result.add_lower_bound(
                template_name,
                GenericParent::ClassLike(method_context.class_like_metadata.name),
                template_type.clone(),
            );

            if let Some(variance) = instance_type.get_variance(template_index)
                && !variance.is_invariant()
            {
                template_result.projections.insert(template_name, variance);
            }
        }
    }

    if !instance_type.name.as_bytes().eq_ignore_ascii_case(method_context.class_like_metadata.original_name.as_bytes())
        && let Some(calling_class_metadata) = context.codebase.get_class_like(instance_type.name.as_bytes())
    {
        for (template_name, _) in &method_context.class_like_metadata.template_types {
            if template_result.lower_bounds.get(template_name).is_some_and(|m| !m.is_empty()) {
                continue;
            }

            let template_type = get_specialized_template_type(
                context.codebase,
                *template_name,
                method_context.class_like_metadata.name,
                calling_class_metadata,
                instance_type.type_parameters.as_deref(),
            );

            if let Some(template_type) = template_type {
                template_result.add_lower_bound(
                    *template_name,
                    GenericParent::ClassLike(method_context.class_like_metadata.name),
                    template_type,
                );
            }
        }
    }

    let Some(identifier) = method_context.declaring_method_id else {
        return;
    };

    let Some(metadata) = context.codebase.get_class_like(identifier.get_class_name().as_bytes()) else {
        return;
    };

    infer_templates_for_method_call(context, instance_type, method_context, method_metadata, metadata, template_result);
}

fn get_named_static_class_type(class_type: &StaticClassType) -> Option<&TNamedObject> {
    match class_type {
        StaticClassType::Object(TObject::Named(instance_type)) => Some(instance_type),
        StaticClassType::Generic(parameter) => parameter.constraint.types.iter().find_map(|atomic| match atomic {
            TAtomic::Object(TObject::Named(instance_type)) => Some(instance_type),
            _ => None,
        }),
        _ => None,
    }
}

/// Extracts and resolves concrete types for class-level template parameters based on inferred lower bounds.
///
/// This function iterates through the `lower_bounds` collected in a `TemplateResult`.
/// For each template parameter that is defined by a class (`GenericParent::ClassLike`),
/// it calculates the most specific type derived from its lower bounds using
/// `get_most_specific_type_from_bounds`.
///
/// The result is a map where keys are template parameter names (`Word`) and
/// values are vectors containing pairs of the defining class (`GenericParent`) and the
/// resolved concrete type (`TUnion`) for that template in the context of that class.
///
/// This map is typically used later to refine template standins within method/property signatures
/// belonging to the class or its children.
///
/// # Arguments
///
/// * `template_result` - The template result containing the inferred lower bounds.
/// * `context` - The analysis context, providing access to codebase metadata needed for type resolution.
///
/// # Returns
///
/// An `IndexMap` mapping class template parameter names to a vector of (Defining Entity, Resolved Type).
pub(super) fn get_class_template_parameters_from_result<A>(
    template_result: &TemplateResult,
    context: &Context<'_, '_, A>,
) -> IndexMap<Word, Vec<GenericTemplate>, RandomState>
where
    A: Arena,
{
    let mut class_generic_parameters: IndexMap<Word, Vec<GenericTemplate>, RandomState> =
        IndexMap::with_hasher(RandomState::default());

    for (template_name, type_map) in &template_result.lower_bounds {
        for (generic_parent, lower_bounds) in type_map {
            if matches!(generic_parent, GenericParent::ClassLike(_)) && !lower_bounds.is_empty() {
                let specific_bound_type = get_most_specific_type_from_bounds(lower_bounds, context.codebase);

                class_generic_parameters
                    .entry(*template_name)
                    .or_default()
                    .push(GenericTemplate::new(*generic_parent, specific_bound_type));
            }
        }
    }

    class_generic_parameters
}

/// Refines the template result by incorporating template definitions specific to the called function or method.
///
/// This function retrieves the applicable template type definitions (e.g., `@template T as array-key`
/// defined on the function/method itself or inherited) considering the class context.
///
/// If the `template_result` provided does not already contain template type definitions
/// (i.e., `template_result.template_types` is empty), this function populates it with
/// the definitions resolved by `get_template_types_for_class_member`.
///
/// **Note:** If `template_result.template_types` already contains entries (perhaps from
/// analyzing generic class types), this function currently does *not* merge or overwrite them.
/// It only initializes the map if it's empty.
pub(super) fn refine_template_result_for_function_like<'ctx, A>(
    template_result: &mut TemplateResult,
    context: &Context<'ctx, '_, A>,
    method_target_context: Option<&MethodTargetContext<'ctx>>,
    base_class_metadata: Option<&'ctx ClassLikeMetadata>,
    calling_class_like_metadata: Option<&'ctx ClassLikeMetadata>,
    function_like_metadata: &'ctx FunctionLikeMetadata,
    class_template_parameters: &IndexMap<Word, Vec<GenericTemplate>, RandomState>,
) where
    A: Arena,
{
    if !template_result.template_types.is_empty() {
        return;
    }

    let resolved_template_types = get_template_types_for_class_member(
        context,
        base_class_metadata,
        method_target_context.as_ref().map(|mci| mci.class_like_metadata.name),
        calling_class_like_metadata,
        &function_like_metadata.template_types,
        class_template_parameters,
    );

    if resolved_template_types.is_empty() {
        return;
    }

    template_result.template_types = resolved_template_types
        .into_iter()
        .map(|(template_name, type_map)| {
            (
                template_name,
                type_map
                    .into_iter()
                    .map(|(source, template_type)| GenericTemplate::new(source, template_type))
                    .collect(),
            )
        })
        .collect::<IndexMap<_, _, RandomState>>();
}
