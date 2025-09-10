use mago_atom::Atom;
use mago_atom::ascii_lowercase_atom;
use mago_codex::get_class_like;
use mago_codex::get_declaring_method_identifier;
use mago_codex::get_method_by_id;
use mago_codex::get_method_identifier;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;
use mago_codex::method_identifier_exists;
use mago_codex::misc::GenericParent;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::mixed::TMixed;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator::is_contained_by;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_specialized_template_type;
use mago_codex::ttype::template::TemplateResult;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::resolver::class_name::report_non_existent_class_like;
use crate::resolver::selector::resolve_member_selector;
use crate::visibility::check_method_visibility;

#[derive(Debug)]
pub struct ResolvedMethod {
    /// The name of the class this method is called on, not necessarily the same
    /// as the class of the method itself, especially in cases of inheritance.
    pub classname: Atom,
    /// The method identifiers that were successfully resolved.
    pub method_identifier: MethodIdentifier,
    /// The type of `$this` or the static class type if it's a static method.
    pub static_class_type: StaticClassType,
    /// True if this method is static, meaning it can be called without an instance.
    pub is_static: bool,
}

/// Holds the results of resolving a method call, including valid targets and summary flags.
#[derive(Default, Debug)]
pub struct MethodResolutionResult {
    /// The template result containing any type variables and bounds.
    pub template_result: TemplateResult,
    /// A list of resolved methods, each with its template result and identifiers.
    pub resolved_methods: Vec<ResolvedMethod>,
    /// True if any selector was dynamic (e.g., from a generic string), making the method name unknown.
    pub has_dynamic_selector: bool,
    /// True if any resolution path involved an object with an ambiguous type (e.g., `mixed`, generic `object`).
    pub has_ambiguous_target: bool,
    /// True if any resolution path was definitively invalid (e.g., method not found, call on non-object).
    pub has_invalid_target: bool,
    /// True if an access on a `mixed` type was encountered.
    pub encountered_mixed: bool,
    /// True if an access on a `null` type was encountered.
    pub encountered_null: bool,
}

/// Resolves all possible method targets from an object expression and a member selector.
///
/// This utility handles the logic for `$object->selector` by:
///
/// 1. Analyzing the `$object` expression to find its type.
/// 2. Resolving the `selector` to get potential method names.
/// 3. Finding all matching methods on the object's possible types.
/// 4. Reporting any issues found, such as "method not found" or "call on mixed".
pub fn resolve_method_targets<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    object: &'ast Expression<'arena>,
    selector: &'ast ClassLikeMemberSelector<'arena>,
    is_null_safe: bool,
    access_span: Span,
) -> Result<MethodResolutionResult, AnalysisError> {
    let mut result = MethodResolutionResult::default();

    let was_inside_general_use = block_context.inside_general_use;
    block_context.inside_general_use = true;
    object.analyze(context, block_context, artifacts)?;
    block_context.inside_general_use = was_inside_general_use;

    let resolved_selectors = resolve_member_selector(context, block_context, artifacts, selector)?;
    let mut method_names = Vec::new();

    for resolved_selector in resolved_selectors {
        if resolved_selector.is_dynamic() {
            result.has_dynamic_selector = true;
        }

        if let Some(name) = resolved_selector.name() {
            method_names.push(ascii_lowercase_atom(&name));
        } else {
            result.has_invalid_target = true;
        }
    }

    if let Some(object_type) = artifacts.get_expression_type(object) {
        let mut object_atomics = object_type.types.iter().collect::<Vec<_>>();

        while let Some(object_atomic) = object_atomics.pop() {
            if let TAtomic::GenericParameter(TGenericParameter { constraint, .. }) = object_atomic {
                object_atomics.extend(constraint.types.iter());
                continue;
            }

            if object_atomic.is_never() {
                continue;
            }

            if object_atomic.is_null() {
                result.encountered_null = true;
                if !is_null_safe {
                    result.has_invalid_target = true;

                    context.collector.report_with_code(
                        if object_type.is_null() {
                            IssueCode::MethodAccessOnNull
                        } else {
                            IssueCode::PossibleMethodAccessOnNull
                        },
                        Issue::error("Attempting to call a method on `null`.")
                            .with_annotation(
                                Annotation::primary(object.span()).with_message("This expression can be `null`"),
                            )
                            .with_help("Use the nullsafe operator (`?->`) if `null` is an expected value."),
                    );
                }

                continue;
            }

            let TAtomic::Object(obj_type) = object_atomic else {
                if object_atomic.is_mixed() {
                    result.encountered_mixed = true;
                } else {
                    result.has_invalid_target = true;
                }

                report_call_on_non_object(context, object_atomic, object.span(), selector.span());
                continue;
            };

            for method_name in &method_names {
                let resolved_methods = resolve_method_from_object(
                    context,
                    block_context,
                    object,
                    selector,
                    obj_type,
                    *method_name,
                    access_span,
                    &mut result,
                );

                if resolved_methods.is_empty() {
                    if let Some(classname) = obj_type.get_name() {
                        result.has_invalid_target = true;
                        report_non_existent_method(context, object.span(), selector.span(), classname, method_name);
                    } else {
                        // ambiguous
                    }
                }

                result.resolved_methods.extend(resolved_methods);
            }
        }
    } else {
        result.has_invalid_target = true;
        result.encountered_mixed = true;
        report_call_on_non_object(context, &TAtomic::Mixed(TMixed::new()), object.span(), selector.span());
    }

    Ok(result)
}

pub fn resolve_method_from_object<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &BlockContext<'ctx>,
    object: &'ast Expression<'arena>,
    selector: &'ast ClassLikeMemberSelector<'arena>,
    object_type: &TObject,
    method_name: Atom,
    access_span: Span,
    result: &mut MethodResolutionResult,
) -> Vec<ResolvedMethod> {
    let mut resolved_methods = vec![];

    let method_ids = get_method_ids_from_object(
        context,
        block_context,
        object,
        selector,
        object_type,
        object_type,
        method_name,
        access_span,
        result,
    );

    for (metadata, declaring_method_id, object, classname) in method_ids {
        let declaring_class_metadata =
            get_class_like(context.codebase, declaring_method_id.get_class_name()).unwrap_or(metadata);

        let class_template_parameters = super::class_template_type_collector::collect(
            context.codebase,
            metadata,
            declaring_class_metadata,
            Some(object_type),
        );

        if let Some(class_template_parameters) = class_template_parameters {
            result.template_result.add_lower_bounds(class_template_parameters);
        }

        for (index, parameter) in object.get_type_parameters().unwrap_or_default().iter().enumerate() {
            let Some(template_name) = metadata.get_template_name_for_index(index) else {
                continue;
            };

            result
                .template_result
                .template_types
                .entry(template_name)
                .or_default()
                .push((GenericParent::ClassLike(metadata.name), parameter.clone()));
        }

        resolved_methods.push(ResolvedMethod {
            method_identifier: declaring_method_id,
            static_class_type: StaticClassType::Object(object.clone()),
            classname,
            is_static: false,
        });
    }

    resolved_methods
}

pub fn get_method_ids_from_object<'ctx, 'ast, 'arena, 'object>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &BlockContext<'ctx>,
    object: &'ast Expression<'arena>,
    selector: &'ast ClassLikeMemberSelector<'arena>,
    object_type: &'object TObject,
    outer_object: &'object TObject,
    method_name: Atom,
    access_span: Span,
    result: &mut MethodResolutionResult,
) -> Vec<(&'ctx ClassLikeMetadata, MethodIdentifier, &'object TObject, Atom)> {
    let mut ids = vec![];

    let Some(name) = object_type.get_name() else {
        result.has_ambiguous_target = true;
        report_call_on_ambiguous_object(context, object.span(), selector.span());

        return ids;
    };

    let Some(class_metadata) = get_class_like(context.codebase, name) else {
        result.has_invalid_target = true;
        report_non_existent_class_like(context, object.span(), name);
        return ids;
    };

    let mut method_id = get_method_identifier(&class_metadata.original_name, &method_name);
    if !method_identifier_exists(context.codebase, &method_id) {
        method_id = get_declaring_method_identifier(context.codebase, &method_id);
    }

    if let Some(function_like_metadata) = get_method_by_id(context.codebase, &method_id) {
        if !check_method_visibility(
            context,
            block_context,
            method_id.get_class_name(),
            method_id.get_method_name(),
            access_span,
            Some(selector.span()),
        ) {
            result.has_invalid_target = true;
        }

        if !check_where_method_constraints(
            context,
            object_type,
            object,
            selector,
            class_metadata,
            function_like_metadata,
            method_id.get_class_name(),
        ) {
            result.has_invalid_target = true;
        }

        ids.push((class_metadata, method_id, outer_object, *name));
    }

    if let Some(intersection_types) = object_type.get_intersection_types() {
        for intersected_atomic in intersection_types {
            match intersected_atomic {
                TAtomic::Object(intersected_object) => {
                    // Recursively search in the intersection types
                    ids.extend(get_method_ids_from_object(
                        context,
                        block_context,
                        object,
                        selector,
                        intersected_object,
                        object_type,
                        method_name,
                        access_span,
                        result,
                    ));
                }
                TAtomic::GenericParameter(generic_parameter) => {
                    // If the intersection type is a generic parameter, we need to check its constraint
                    for constraint_atomic in generic_parameter.constraint.types.as_ref() {
                        if let TAtomic::Object(intersected_object) = constraint_atomic {
                            // Recursively search in the intersection types
                            ids.extend(get_method_ids_from_object(
                                context,
                                block_context,
                                object,
                                selector,
                                intersected_object,
                                object_type,
                                method_name,
                                access_span,
                                result,
                            ));
                        }
                    }
                }
                _ => {
                    // For other atomic types, we do not need to do anything special
                }
            }
        }
    }

    ids
}

fn check_where_method_constraints(
    context: &mut Context,
    object_type: &TObject,
    object: &Expression,
    selector: &ClassLikeMemberSelector,
    class_like_metadata: &ClassLikeMetadata,
    function_like_metadata: &FunctionLikeMetadata,
    defining_class_id: &Atom,
) -> bool {
    let Some(method_metadata) = function_like_metadata.method_metadata.as_ref() else {
        return true;
    };

    if method_metadata.where_constraints.is_empty() {
        return true;
    }

    for (template_name, constraint) in &method_metadata.where_constraints {
        let actual_template_type = get_specialized_template_type(
            context.codebase,
            template_name,
            defining_class_id,
            class_like_metadata,
            object_type.get_type_parameters(),
        )
        .unwrap_or_else(get_mixed);

        if is_contained_by(
            context.codebase,
            &actual_template_type,
            &constraint.type_union,
            false,
            false,
            false,
            &mut ComparisonResult::default(),
        ) {
            continue;
        }

        let required_constraint_str = constraint.type_union.get_id();
        let actual_template_type_str = actual_template_type.get_id();

        context.collector.report_with_code(
            IssueCode::WhereConstraintViolation,
            Issue::error(format!(
                "Method call violates `@where` constraint for template `{template_name}`.",
            ))
            .with_annotation(
                Annotation::primary(selector.span())
                    .with_message("This method cannot be called here..."),
            )
            .with_annotation(
                Annotation::secondary(object.span())
                    .with_message(format!(
                        "...because this object's template parameter `{template_name}` is type `{actual_template_type_str}`...",
                    )),
            )
            .with_annotation(
                Annotation::secondary(constraint.span)
                    .with_message(format!(
                        "...but this `@where` clause requires it to be `{required_constraint_str}`.",
                    )),
            )
            .with_note(
                "The `@where` tag on a method adds a constraint that must be satisfied by the object's generic types at the time of the call."
            )
            .with_help(
                format!("Ensure the object's template parameter `{template_name}` satisfies the `{required_constraint_str}` constraint before calling this method.")
            ),
        );

        return false;
    }

    true
}

fn report_call_on_non_object(context: &mut Context, atomic_type: &TAtomic, obj_span: Span, selector_span: Span) {
    let type_str = atomic_type.get_id();

    context.collector.report_with_code(
        if atomic_type.is_mixed() { IssueCode::MixedMethodAccess } else { IssueCode::InvalidMethodAccess },
        Issue::error(format!("Attempting to access a method on a non-object type (`{type_str}`)."))
            .with_annotation(Annotation::primary(selector_span).with_message("Cannot call method here"))
            .with_annotation(
                Annotation::secondary(obj_span).with_message(format!("This expression has type `{type_str}`")),
            ),
    );
}

fn report_call_on_ambiguous_object(context: &mut Context, obj_span: Span, selector_span: Span) {
    context.collector.report_with_code(
        IssueCode::AmbiguousObjectMethodAccess,
        Issue::warning("Cannot statically verify method call on a generic `object` type.")
            .with_annotation(Annotation::primary(selector_span).with_message("Cannot verify this method call"))
            .with_annotation(
                Annotation::secondary(obj_span).with_message("This expression has the general type `object`"),
            )
            .with_help("Provide a more specific type hint for the object for robust analysis."),
    );
}

pub(super) fn report_non_existent_method(
    context: &mut Context,
    obj_span: Span,
    selector_span: Span,
    classname: &Atom,
    method_name: &Atom,
) {
    context.collector.report_with_code(
        IssueCode::NonExistentMethod,
        Issue::error(format!("Method `{method_name}` does not exist on type `{classname}`."))
            .with_annotation(Annotation::primary(selector_span).with_message("This method selection is invalid"))
            .with_annotation(
                Annotation::secondary(obj_span).with_message(format!("This expression has type `{classname}`")),
            )
            .with_help(format!("Ensure the `{method_name}` method is defined in the `{classname}` class-like.")),
    );
}
