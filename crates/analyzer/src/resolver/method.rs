use mago_atom::Atom;
use mago_atom::ascii_lowercase_atom;
use mago_atom::atom;

use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::metadata::function_like::FunctionLikeMetadata;

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
use mago_codex::ttype::union::TUnion;
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
    /// If Some, this method was found in a mixin but the target class lacks the magic method
    /// needed to forward the call. Contains the mixin class name and whether the target is final.
    pub mixin_without_magic_method: Option<MixinWithoutMagicMethod>,
}

/// Represents a method found in a mixin where the calling class lacks the required magic method.
#[derive(Debug, Clone)]
pub struct MixinWithoutMagicMethod {
    /// The name of the mixin class where the method was found.
    pub mixin_class_name: Atom,
    /// Whether the target class (that has the mixin) is final.
    pub target_is_final: bool,
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
    /// True if all resolved methods have non-nullable return types.
    /// When combined with `encountered_null` and nullsafe access, indicates
    /// the null in the result type came ONLY from nullsafe short-circuit.
    pub all_methods_non_nullable_return: bool,
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

    let was_inside_general_use = block_context.flags.inside_general_use();
    block_context.flags.set_inside_general_use(true);
    object.analyze(context, block_context, artifacts)?;
    block_context.flags.set_inside_general_use(was_inside_general_use);

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
                if !object_type.ignore_nullable_issues() && !is_null_safe && !object_type.has_nullsafe_null() {
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

            let resolved_magic_call_method = resolve_method_from_object(
                context,
                block_context,
                object,
                selector,
                obj_type,
                atom("__call"),
                access_span,
                true,
                &mut result,
            );

            for method_name in &method_names {
                let resolved_methods = resolve_method_from_object(
                    context,
                    block_context,
                    object,
                    selector,
                    obj_type,
                    *method_name,
                    access_span,
                    !resolved_magic_call_method.is_empty(),
                    &mut result,
                );

                if resolved_methods.is_empty() {
                    if let Some(classname) = obj_type.get_name() {
                        let method_name_str: &str = method_name.as_ref();
                        let has_method_assertion = type_has_method_assertion(obj_type, method_name_str);

                        if !has_method_assertion {
                            if resolved_magic_call_method.is_empty() {
                                report_non_existent_method(
                                    context,
                                    object.span(),
                                    selector.span(),
                                    *classname,
                                    *method_name,
                                );
                            } else {
                                report_non_documented_method(
                                    context,
                                    object.span(),
                                    selector.span(),
                                    *classname,
                                    *method_name,
                                );
                            }

                            result.has_invalid_target = true;
                        }
                    } else {
                        // ambiguous
                    }
                } else {
                    // Check if any resolved method was found in a mixin without magic method support
                    for resolved_method in &resolved_methods {
                        if let Some(mixin_info) = &resolved_method.mixin_without_magic_method
                            && let Some(classname) = obj_type.get_name()
                        {
                            if mixin_info.target_is_final {
                                // Final class - error, method can never work at runtime
                                report_non_existent_mixin_method(
                                    context,
                                    object.span(),
                                    selector.span(),
                                    *classname,
                                    *method_name,
                                    mixin_info.mixin_class_name,
                                );
                                result.has_invalid_target = true;
                            } else {
                                // Non-final class - warning, subclass might implement __call
                                report_possibly_non_existent_mixin_method(
                                    context,
                                    object.span(),
                                    selector.span(),
                                    *classname,
                                    *method_name,
                                    mixin_info.mixin_class_name,
                                );
                            }
                        }
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

    // Compute whether all resolved methods have non-nullable return types.
    // This is used to determine if null in the result type came only from nullsafe short-circuit.
    result.all_methods_non_nullable_return = !result.resolved_methods.is_empty()
        && result.resolved_methods.iter().all(|resolved_method| {
            context
                .codebase
                .get_method_by_id(&resolved_method.method_identifier)
                .and_then(|method| method.return_type_metadata.as_ref())
                .is_some_and(|return_type| !return_type.type_union.is_nullable())
        });

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
    has_magic_call: bool,
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
        has_magic_call,
        result,
    );

    for (metadata, declaring_method_id, object, classname, mixin_without_magic_method) in method_ids {
        let declaring_class_metadata =
            context.codebase.get_class_like(declaring_method_id.get_class_name()).unwrap_or(metadata);

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
            mixin_without_magic_method,
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
    has_magic_call: bool,
    result: &mut MethodResolutionResult,
) -> Vec<(&'ctx ClassLikeMetadata, MethodIdentifier, &'object TObject, Atom, Option<MixinWithoutMagicMethod>)> {
    let mut ids = vec![];

    let Some(name) = object_type.get_name() else {
        result.has_ambiguous_target = true;

        if !has_magic_call {
            let method_name_str: &str = method_name.as_ref();
            let has_method_assertion = type_has_method_assertion(object_type, method_name_str);
            if !has_method_assertion {
                report_call_on_ambiguous_object(context, object.span(), selector.span());
            }
        }

        return ids;
    };

    let Some(class_metadata) = context.codebase.get_class_like(name) else {
        result.has_invalid_target = true;
        report_non_existent_class_like(context, object.span(), *name);
        return ids;
    };

    let mut method_id = MethodIdentifier::new(atom(&class_metadata.original_name), atom(&method_name));
    if !context.codebase.method_identifier_exists(&method_id) {
        method_id = context.codebase.get_declaring_method_identifier(&method_id);
    }

    if let Some(function_like_metadata) = context.codebase.get_method_by_id(&method_id) {
        if !check_method_visibility(
            context,
            block_context,
            &class_metadata.original_name,
            &method_name,
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
            class_metadata.original_name,
        ) {
            result.has_invalid_target = true;
        }

        if function_like_metadata.flags.is_magic_method() {
            let lowercase_method = ascii_lowercase_atom(&method_name);
            let is_pseudo = class_metadata.pseudo_methods.contains(&lowercase_method)
                || class_metadata.all_parent_classes.iter().any(|parent_name| {
                    context
                        .codebase
                        .get_class_like(parent_name)
                        .is_some_and(|parent| parent.pseudo_methods.contains(&lowercase_method))
                });

            let mut is_inherited = false;

            if is_pseudo {
                for parent_class_name in &class_metadata.all_parent_classes {
                    if let Some(parent_metadata) = context.codebase.get_class_like(parent_class_name)
                        && parent_metadata.methods.contains(&lowercase_method)
                        && !parent_metadata.pseudo_methods.contains(&lowercase_method)
                    {
                        is_inherited = true;
                        break;
                    }
                }
            }

            if function_like_metadata.flags.is_static() {
                result.has_invalid_target = true;

                report_dynamic_static_method_call(
                    context,
                    object.span(),
                    selector.span(),
                    class_metadata.original_name,
                    method_name,
                    has_magic_call,
                );
            } else if !has_magic_call && !is_inherited {
                report_magic_call_without_call_method(
                    context,
                    object.span(),
                    selector.span(),
                    class_metadata.original_name,
                    method_name,
                    false,
                );
            }
        }

        ids.push((class_metadata, method_id, outer_object, *name, None));
    } else if !class_metadata.mixins.is_empty() {
        // Search mixins for the method. If has_magic_call is false, we track that
        // the method was found in a mixin without the required magic method.
        let mixin_class_names = collect_mixin_class_names(class_metadata, outer_object, &class_metadata.mixins);

        for mixin_class_name in mixin_class_names {
            let Some(mixin_metadata) = context.codebase.get_class_like(&mixin_class_name) else {
                continue;
            };

            let mut mixin_method_id = MethodIdentifier::new(atom(&mixin_metadata.original_name), atom(&method_name));
            if !context.codebase.method_identifier_exists(&mixin_method_id) {
                mixin_method_id = context.codebase.get_declaring_method_identifier(&mixin_method_id);
            }

            if let Some(function_like_metadata) = context.codebase.get_method_by_id(&mixin_method_id) {
                if !check_method_visibility(
                    context,
                    block_context,
                    &mixin_metadata.original_name,
                    &method_name,
                    access_span,
                    Some(selector.span()),
                ) {
                    result.has_invalid_target = true;
                    continue;
                }

                if let Some(method_metadata) = &function_like_metadata.method_metadata
                    && !method_metadata.visibility.is_public()
                {
                    continue;
                }

                // Track if this method was found without magic call support
                let mixin_info = if has_magic_call {
                    None
                } else {
                    Some(MixinWithoutMagicMethod { mixin_class_name, target_is_final: class_metadata.flags.is_final() })
                };

                ids.push((mixin_metadata, mixin_method_id, outer_object, mixin_class_name, mixin_info));
            }
        }
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
                        has_magic_call,
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
                                has_magic_call,
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
    defining_class_id: Atom,
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
            &defining_class_id,
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
    classname: Atom,
    method_name: Atom,
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

pub(super) fn report_non_documented_method(
    context: &mut Context,
    obj_span: Span,
    selector_span: Span,
    classname: Atom,
    method_name: Atom,
) {
    context.collector.report_with_code(
        IssueCode::NonDocumentedMethod,
        Issue::warning(format!(
            "Ambiguous method call to `{method_name}` on class `{classname}`."
        ))
        .with_annotation(
            Annotation::primary(selector_span).with_message("This method is not explicitly defined"),
        )
        .with_annotation(
            Annotation::secondary(obj_span).with_message(format!("On an object of type `{classname}`")),
        )
        .with_note(
            "While this call might be handled by `__call()` or `__callStatic()`, Mago cannot verify its arguments or return type without a corresponding `@method` docblock tag.",
        )
        .with_help(format!(
            "To enable full analysis, add a `@method` tag to the docblock of the `{classname}` class. For example: `/** @method returnType {method_name}(argType $argName) */`"
        )),
    );
}

/// Reports a warning when a method is found in a mixin but the target class lacks __call.
/// This is a warning because a subclass might implement __call.
fn report_possibly_non_existent_mixin_method(
    context: &mut Context,
    obj_span: Span,
    selector_span: Span,
    classname: Atom,
    method_name: Atom,
    mixin_classname: Atom,
) {
    context.collector.report_with_code(
        IssueCode::PossiblyNonExistentMethod,
        Issue::warning(format!(
            "Method `{method_name}` might not exist on type `{classname}` at runtime."
        ))
        .with_annotation(
            Annotation::primary(selector_span).with_message("Method might not exist"),
        )
        .with_annotation(
            Annotation::secondary(obj_span).with_message(format!("On an instance of `{classname}`")),
        )
        .with_note(format!(
            "The method `{method_name}` is defined in mixin class `{mixin_classname}`, but `{classname}` does not have a `__call` method to forward the call."
        ))
        .with_note(
            "A subclass of this class could implement `__call` to handle this, so the call might succeed at runtime."
        )
        .with_help(format!(
            "Add a `__call` method to `{classname}`, or make `{classname}` final if this should be an error."
        )),
    );
}

/// Reports an error when a method is found in a mixin but the target final class lacks __call.
/// This is an error because no subclass can exist to implement __call.
fn report_non_existent_mixin_method(
    context: &mut Context,
    obj_span: Span,
    selector_span: Span,
    classname: Atom,
    method_name: Atom,
    mixin_classname: Atom,
) {
    context.collector.report_with_code(
        IssueCode::NonExistentMethod,
        Issue::error(format!(
            "Method `{method_name}` does not exist on final type `{classname}`."
        ))
        .with_annotation(
            Annotation::primary(selector_span).with_message("Method does not exist"),
        )
        .with_annotation(
            Annotation::secondary(obj_span).with_message(format!("On an instance of final class `{classname}`")),
        )
        .with_note(format!(
            "The method `{method_name}` is defined in mixin class `{mixin_classname}`, but `{classname}` is final and does not have a `__call` method to forward the call."
        ))
        .with_help(format!(
            "Add a `__call` method to `{classname}` to handle mixin method calls."
        )),
    );
}

pub(super) fn report_magic_call_without_call_method(
    context: &mut Context,
    obj_span: Span,
    selector_span: Span,
    classname: Atom,
    method_name: Atom,
    is_static: bool,
) {
    let magic_method_name = if is_static { "__callStatic" } else { "__call" };

    context.collector.report_with_code(
        IssueCode::MissingMagicMethod,
        Issue::error(format!(
            "Call to documented magic method `{method_name}()` on a class that cannot handle it."
        ))
        .with_annotation(
            Annotation::primary(selector_span)
                .with_message("This magic method is documented but cannot be called"),
        )
        .with_annotation(
            Annotation::secondary(obj_span).with_message(format!("Class `{classname}` is missing the `{magic_method_name}` method")),
        )
        .with_note(
            format!("The class `{classname}` has a `@method` tag for `{method_name}` but does not have a `{magic_method_name}` method to handle the call. This will cause a fatal `Error` at runtime.")
        )
        .with_help(
            format!("Add a `{magic_method_name}` method to the `{classname}` class to handle calls to magic methods.")
        ),
    );
}

pub(super) fn report_dynamic_static_method_call(
    context: &mut Context,
    obj_span: Span,
    selector_span: Span,
    classname: Atom,
    method_name: Atom,
    has_magic_call: bool,
) {
    let mut issue =
        Issue::error(format!("Cannot call magic static method `{classname}::{method_name}` on an instance."))
            .with_annotation(
                Annotation::primary(selector_span)
                    .with_message("This magic method is static and must be called statically"),
            )
            .with_annotation(
                Annotation::secondary(obj_span).with_message(format!("Called on an instance of `{classname}`")),
            );

    if has_magic_call {
        issue = issue
            .with_note(format!(
                "The magic method `{method_name}` is documented as `static` and is intended to be handled by `__callStatic()`."
            ))
            .with_note(
                "However, because it's being called on an instance (`->`), the call will be routed to the existing `__call()` method instead."
            )
            .with_note(
                "This is likely not the intended behavior and may lead to unexpected errors."
            );
    } else {
        issue = issue
            .with_note(
                "Magic methods defined with `@method static` are handled by `__callStatic()`."
            )
            .with_note(
                "When called on an instance (`->`), PHP attempts to route the call to a `__call()` method."
            )
            .with_note(format!(
                "Since the class `{classname}` is missing a `__call()` method, this will cause a fatal `Error` at runtime."
            ));
    }

    context.collector.report_with_code(
        IssueCode::DynamicStaticMethodCall,
        issue.with_help(format!("Call this method statically instead: `{classname}::{method_name}`.")),
    );
}

/// Checks if a type has a known method assertion for the given method name.
///
/// This checks for `HasMethod` types or intersection types containing them.
/// Comparison is case-insensitive since PHP method names are case-insensitive.
fn type_has_method_assertion(object_type: &TObject, method_name: &str) -> bool {
    match object_type {
        TObject::HasMethod(has_method) => {
            if has_method.has_method(method_name) {
                return true;
            }

            has_method.intersection_types.as_ref().is_some_and(|types| {
                types.iter().any(|atomic| {
                    if let TAtomic::Object(obj) = atomic { type_has_method_assertion(obj, method_name) } else { false }
                })
            })
        }
        TObject::HasProperty(has_property) => has_property.intersection_types.as_ref().is_some_and(|types| {
            types.iter().any(|atomic| {
                if let TAtomic::Object(obj) = atomic { type_has_method_assertion(obj, method_name) } else { false }
            })
        }),
        TObject::Named(named_object) => named_object.get_intersection_types().is_some_and(|intersection_types| {
            intersection_types.iter().any(|atomic| {
                if let TAtomic::Object(obj) = atomic { type_has_method_assertion(obj, method_name) } else { false }
            })
        }),
        _ => false,
    }
}

fn collect_mixin_class_names(
    class_metadata: &ClassLikeMetadata,
    outer_object: &TObject,
    mixins: &[TUnion],
) -> Vec<Atom> {
    let mut class_names = Vec::new();

    for mixin_type in mixins {
        for mixin_atomic in mixin_type.types.as_ref() {
            match mixin_atomic {
                TAtomic::Object(TObject::Named(named)) => {
                    class_names.push(ascii_lowercase_atom(&named.name));
                }
                TAtomic::Object(TObject::Enum(enum_type)) => {
                    class_names.push(ascii_lowercase_atom(&enum_type.name));
                }
                TAtomic::GenericParameter(TGenericParameter {
                    parameter_name, constraint, defining_entity, ..
                }) => {
                    let mut resolved = false;

                    if let TObject::Named(named_object) = outer_object
                        && let Some(type_params) = named_object.get_type_parameters()
                        && let GenericParent::ClassLike(defining_class) = defining_entity
                        && named_object.name.eq_ignore_ascii_case(defining_class)
                        && let Some(index) = class_metadata.get_template_index_for_name(parameter_name)
                        && let Some(concrete_type) = type_params.get(index)
                    {
                        for atomic in concrete_type.types.as_ref() {
                            match atomic {
                                TAtomic::Object(TObject::Named(named)) => {
                                    class_names.push(ascii_lowercase_atom(&named.name));
                                    resolved = true;
                                }
                                TAtomic::Object(TObject::Enum(enum_type)) => {
                                    class_names.push(ascii_lowercase_atom(&enum_type.name));
                                    resolved = true;
                                }
                                _ => {}
                            }
                        }
                    }

                    // Fallback to constraint if we couldn't resolve
                    if !resolved {
                        for constraint_atomic in constraint.types.as_ref() {
                            match constraint_atomic {
                                TAtomic::Object(TObject::Named(named)) => {
                                    class_names.push(ascii_lowercase_atom(&named.name));
                                }
                                TAtomic::Object(TObject::Enum(enum_type)) => {
                                    class_names.push(ascii_lowercase_atom(&enum_type.name));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    class_names
}
