use std::borrow::Cow;

use ahash::RandomState;
use indexmap::IndexMap;

use mago_atom::Atom;
use mago_codex::get_class_like;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::misc::GenericParent;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::template::TemplateBound;
use mago_codex::ttype::template::TemplateResult;
use mago_codex::ttype::template::standin_type_replacer::get_most_specific_type_from_bounds;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::*;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::Span;

use crate::code::IssueCode;
use crate::context::Context;
use crate::invocation::Invocation;
use crate::invocation::InvocationTarget;
use crate::invocation::MethodTargetContext;
use crate::invocation::template_inference::infer_templates_for_method_call;
use crate::utils::misc::unique_vec;
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
pub fn populate_template_result_from_invocation<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    invocation: &Invocation<'ctx, 'ast, 'arena>,
    template_result: &mut TemplateResult,
) {
    let InvocationTarget::FunctionLike { metadata, method_context, .. } = &invocation.target else {
        return;
    };

    for (template_name, template_details) in metadata.template_types.iter() {
        template_result.template_types.insert(*template_name, template_details.clone());
    }

    let Some(method_metadata) = &metadata.method_metadata else {
        return;
    };

    if method_metadata.is_static {
        return;
    }

    let Some(method_context) = method_context else {
        return;
    };

    let StaticClassType::Object(TObject::Named(instance_type)) = &method_context.class_type else {
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
        }
    }

    let Some(identifier) = method_context.declaring_method_id else {
        return;
    };

    let Some(metadata) = get_class_like(context.codebase, identifier.get_class_name()) else {
        return;
    };

    infer_templates_for_method_call(context, instance_type, method_context, method_metadata, metadata, template_result);
}

/// Extracts and resolves concrete types for class-level template parameters based on inferred lower bounds.
///
/// This function iterates through the `lower_bounds` collected in a `TemplateResult`.
/// For each template parameter that is defined by a class (`GenericParent::ClassLike`),
/// it calculates the most specific type derived from its lower bounds using
/// `get_most_specific_type_from_bounds`.
///
/// The result is a map where keys are template parameter names (`Atom`) and
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
pub(super) fn get_class_template_parameters_from_result<'ctx, 'arena>(
    template_result: &TemplateResult,
    context: &Context<'ctx, 'arena>,
) -> IndexMap<Atom, Vec<(GenericParent, TUnion)>, RandomState> {
    let mut class_generic_parameters: IndexMap<Atom, Vec<(GenericParent, TUnion)>, RandomState> =
        IndexMap::with_hasher(RandomState::new());

    for (template_name, type_map) in &template_result.lower_bounds {
        for (generic_parent, lower_bounds) in type_map {
            if matches!(generic_parent, GenericParent::ClassLike(_)) && !lower_bounds.is_empty() {
                let specific_bound_type = get_most_specific_type_from_bounds(lower_bounds, context.codebase);

                class_generic_parameters
                    .entry(*template_name)
                    .or_default()
                    .push((*generic_parent, specific_bound_type));
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
pub(super) fn refine_template_result_for_function_like<'ctx, 'arena>(
    template_result: &mut TemplateResult,
    context: &Context<'ctx, 'arena>,
    method_target_context: Option<&MethodTargetContext<'ctx>>,
    base_class_metadata: Option<&'ctx ClassLikeMetadata>,
    calling_class_like_metadata: Option<&'ctx ClassLikeMetadata>,
    function_like_metadata: &'ctx FunctionLikeMetadata,
    class_template_parameters: &IndexMap<Atom, Vec<(GenericParent, TUnion)>, RandomState>,
) {
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
        .map(|(template_name, type_map)| (template_name, type_map.into_iter().collect()))
        .collect::<IndexMap<_, _, RandomState>>();
}

/// Checks the consistency of inferred template parameter bounds.
///
/// This function analyzes the collected lower bounds (`T >: X`) and upper bounds (`T <: Y`, `T = Z`)
/// for each template parameter (`T`) within a `TemplateResult`. It reports errors if conflicting
/// bounds are found, such as:
///
/// - A lower bound that is not a subtype of an upper bound (`X` not assignable to `Y`).
/// - Multiple incompatible equality bounds (`T = int` and `T = string`).
/// - A lower bound that is not compatible with an equality bound (`T >: float` and `T = int`).
///
/// # Arguments
///
/// * `context` - The analysis context, providing access to the codebase metadata.
/// * `template_result` - The result containing the bounds to check (will be mutated if bounds are added).
/// * `span` - The span (location) to associate with any reported errors (e.g., the call site).
pub(super) fn check_template_result<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    template_result: &mut TemplateResult,
    span: Span,
) {
    if template_result.lower_bounds.is_empty() {
        return;
    }

    let codebase = context.codebase;

    for (template_name, defining_map) in &template_result.upper_bounds {
        for (defining_entity, upper_bound) in defining_map {
            let lower_bounds = template_result
                .lower_bounds
                .entry(*template_name)
                .or_default()
                .entry(*defining_entity)
                .or_insert_with(|| vec![TemplateBound::of_type(upper_bound.bound_type.clone())]);

            let (lower_bound_type, upper_bound_type) = if template_result.upper_bounds_unintersectable_types.len() > 1 {
                (
                    Cow::Borrowed(&template_result.upper_bounds_unintersectable_types[0]),
                    Cow::Borrowed(&template_result.upper_bounds_unintersectable_types[1]),
                )
            } else {
                (
                    Cow::Owned(get_most_specific_type_from_bounds(lower_bounds, codebase)),
                    Cow::Borrowed(&upper_bound.bound_type),
                )
            };

            let mut comparison_result = ComparisonResult::new();
            let is_contained = union_comparator::is_contained_by(
                codebase,
                &lower_bound_type,
                &upper_bound_type,
                false,
                false,
                false,
                &mut comparison_result,
            );

            if !is_contained {
                let issue_kind = if comparison_result.type_coerced.unwrap_or(false)
                    && comparison_result.type_coerced_from_as_mixed.unwrap_or(false)
                {
                    IssueCode::MixedArgument
                } else {
                    IssueCode::InvalidArgument
                };

                context.collector.report_with_code(
                    issue_kind,
                    Issue::error(format!("Incompatible template bounds for `{template_name}`."))
                        .with_annotation(Annotation::primary(span).with_message(format!(
                            "Inferred type `{}` is not compatible with declared bound `{}`",
                            lower_bound_type.get_id(),
                            upper_bound_type.get_id(),
                        )))
                        .with_note(format!("Could not reconcile bounds for template parameter `{template_name}`."))
                        .with_help(
                            "Check the types used for arguments or properties related to this template parameter.",
                        ),
                );
            }
        }
    }

    for (template_name, lower_bounds_map) in &template_result.lower_bounds {
        for lower_bounds in lower_bounds_map.values() {
            if lower_bounds.len() <= 1 {
                continue;
            }

            let bounds_with_equality: Vec<_> =
                lower_bounds.iter().filter(|bound| bound.equality_bound_classlike.is_some()).collect();

            if !bounds_with_equality.is_empty() {
                let equality_types: Vec<_> =
                    unique_vec(bounds_with_equality.iter().map(|bound| bound.bound_type.get_id().as_str()));

                if equality_types.len() > 1 {
                    context.collector.report_with_code(
                        IssueCode::ConflictingTemplateEqualityBounds,
                        Issue::error(format!(
                            "Conflicting equality requirements found for template `{template_name}`.",
                        ))
                        .with_annotation(Annotation::primary(span).with_message(format!(
                            "Template `{template_name}` cannot be equal to all of: `{}`.",
                            equality_types.join("`, `"),
                        )))
                        .with_help(
                            "Check the argument types provided for this template parameter; they must resolve to a single compatible type."
                        ),
                    );

                    continue;
                }
            }

            if let Some(first_equality_bound) = bounds_with_equality.first() {
                for lower_bound in lower_bounds {
                    if lower_bound.equality_bound_classlike.is_some() {
                        continue;
                    }

                    let is_contained = union_comparator::is_contained_by(
                        codebase,
                        &lower_bound.bound_type,
                        &first_equality_bound.bound_type,
                        false,
                        false,
                        false,
                        &mut ComparisonResult::new(),
                    );

                    if !is_contained {
                        context.collector.report_with_code(
                            IssueCode::IncompatibleTemplateLowerBound,
                            Issue::error(format!(
                                "Incompatible bounds found for template `{template_name}`.",
                            ))
                            .with_annotation(Annotation::primary(span).with_message(format!(
                                "Type `{}` required by a lower bound is not compatible with the required equality type `{}`.",
                                lower_bound.bound_type.get_id(),
                                first_equality_bound.bound_type.get_id(),
                            )))
                            .with_help(
                                "Check the argument types provided; they must satisfy all lower and equality bounds simultaneously."
                            ),
                        );
                    }
                }
            }
        }
    }
}
