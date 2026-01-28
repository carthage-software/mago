use mago_atom::Atom;
use mago_atom::atom;

use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::r#enum::TEnum;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::get_specialized_template_type;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::wrap_atomic;
use mago_php_version::feature::Feature;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;

use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::resolver::class_name::ResolutionOrigin;
use crate::resolver::class_name::ResolvedClassname;
use crate::resolver::class_name::resolve_classnames_from_expression;
use crate::resolver::method::MethodResolutionResult;
use crate::resolver::method::ResolvedMethod;
use crate::resolver::method::report_magic_call_without_call_method;
use crate::resolver::method::report_non_documented_method;
use crate::resolver::method::report_non_existent_method;
use crate::resolver::selector::resolve_member_selector;
use crate::visibility::check_method_visibility;

/// Resolves all possible static method targets from a class expression and a member selector.
///
/// This utility handles the logic for `ClassName::method` by:
/// 1. Resolving the `ClassName` expression to get all possible class types.
/// 2. Resolving the `method` selector to get potential method names.
/// 3. Finding matching methods and validating them against static access rules.
/// 4. Reporting issues like calling a non-static method, or calling a method on an interface.
pub fn resolve_static_method_targets<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    class_expr: &'ast Expression<'arena>,
    method_selector: &'ast ClassLikeMemberSelector<'arena>,
    access_span: Span,
) -> Result<MethodResolutionResult, AnalysisError> {
    let mut result = MethodResolutionResult::default();

    let class_resolutions = resolve_classnames_from_expression(context, block_context, artifacts, class_expr, false)?;
    if let Some(class_type) = artifacts.get_expression_type(class_expr)
        && class_type.is_nullable()
    {
        result.encountered_null = true;
    }
    let selector_resolutions = resolve_member_selector(context, block_context, artifacts, method_selector)?;

    let mut method_names = vec![];
    for selector in &selector_resolutions {
        if selector.is_dynamic() {
            result.has_dynamic_selector = true;
        }
        if let Some(name) = selector.name() {
            method_names.push(name);
        } else {
            result.has_invalid_target = true;
        }
    }

    for resolved_classname in &class_resolutions {
        if resolved_classname.is_possibly_invalid() {
            result.has_ambiguous_target = true;
            if resolved_classname.origin == ResolutionOrigin::Invalid {
                result.has_invalid_target = true;
            }

            continue;
        }

        for method_name in &method_names {
            let resolved_methods = resolve_method_from_classname(
                context,
                block_context,
                block_context.scope.get_class_like(),
                *method_name,
                class_expr.span(),
                method_selector.span(),
                resolved_classname,
                &mut result,
                method_selector,
                access_span,
            );

            result.resolved_methods.extend(resolved_methods);
        }
    }

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

fn resolve_method_from_classname<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    current_class_metadata: Option<&'ctx ClassLikeMetadata>,
    method_name: Atom,
    class_span: Span,
    method_span: Span,
    classname: &ResolvedClassname,
    result: &mut MethodResolutionResult,
    selector: &ClassLikeMemberSelector<'arena>,
    access_span: Span,
) -> Vec<ResolvedMethod> {
    let mut resolve_method_from_class_id =
        |fq_class_id: Atom,
         is_relative: bool,
         from_instance: bool,
         from_class_string: bool,
         method_name: Atom,
         has_magic_static_call: bool,
         result: Option<&mut MethodResolutionResult>| {
            let Some(defining_class_metadata) = context.codebase.get_class_like(&fq_class_id) else {
                return (false, None);
            };

            if !from_instance && !is_relative && defining_class_metadata.kind.is_interface() && !has_magic_static_call {
                report_static_call_on_interface(
                    context,
                    defining_class_metadata.original_name,
                    class_span,
                    from_class_string,
                );

                if !from_class_string {
                    if let Some(result) = result {
                        result.has_invalid_target = true;
                    }
                    return (true, None);
                }
            }

            let Some(method) = resolve_method_from_metadata(
                context,
                block_context,
                current_class_metadata,
                method_name,
                fq_class_id,
                defining_class_metadata,
                classname,
                result,
                selector,
                access_span,
                class_span,
                has_magic_static_call,
            ) else {
                return (false, None);
            };

            if !method.is_static
                && !is_relative
                && !current_class_metadata.is_some_and(|current_class_metadata| {
                    current_class_metadata.name == defining_class_metadata.name
                        || current_class_metadata.has_parent(&defining_class_metadata.name)
                })
            {
                report_non_static_access(context, &method.method_identifier, method_span);
                return (true, None);
            }

            if !from_instance
                && !is_relative
                && method.is_static
                && defining_class_metadata.kind.is_trait()
                && context.settings.version.is_deprecated(Feature::CallStaticMethodOnTrait)
            {
                report_deprecated_static_access_on_trait(context, defining_class_metadata.original_name, class_span);
            }

            (true, Some(method))
        };

    let mut resolved_methods = vec![];
    let mut could_method_ever_exist = false;
    let mut first_class_id = None;
    let mut magic_call_could_exist = false;

    let magic_method_to_check = if classname.is_parent() { atom("__call") } else { atom("__callStatic") };
    if let Some(fq_class_id) = classname.fqcn {
        let (_, resolved_magic_call_method) = resolve_method_from_class_id(
            fq_class_id,
            classname.is_relative(),
            classname.is_object_instance(),
            classname.is_from_class_string(),
            magic_method_to_check,
            true,
            None,
        );

        magic_call_could_exist |= resolved_magic_call_method.is_some();

        let (could_method_exist, resolved_method) = resolve_method_from_class_id(
            fq_class_id,
            classname.is_relative(),
            classname.is_object_instance(),
            classname.is_from_class_string(),
            method_name,
            resolved_magic_call_method.is_some(),
            Some(result),
        );

        if let Some(resolved_method) = resolved_method {
            resolved_methods.push(resolved_method);
        }

        could_method_ever_exist |= could_method_exist;
        first_class_id = Some(fq_class_id);
    }

    for intersection in &classname.intersections {
        let Some(fq_class_id) = intersection.fqcn else {
            continue;
        };

        let (_, resolved_magic_call_method) = resolve_method_from_class_id(
            fq_class_id,
            intersection.is_relative() || classname.is_relative(),
            intersection.is_object_instance() || classname.is_object_instance(),
            intersection.is_from_class_string(),
            magic_method_to_check,
            true,
            None,
        );

        magic_call_could_exist |= resolved_magic_call_method.is_some();

        let (could_method_exist, resolved_method) = resolve_method_from_class_id(
            fq_class_id,
            intersection.is_relative() || classname.is_relative(),
            intersection.is_object_instance() || classname.is_object_instance(),
            intersection.is_from_class_string(),
            method_name,
            resolved_magic_call_method.is_some(),
            Some(result),
        );

        if let Some(resolved_method) = resolved_method {
            resolved_methods.push(resolved_method);
        }

        could_method_ever_exist |= could_method_exist;
        if first_class_id.is_none() {
            first_class_id = Some(fq_class_id);
        }
    }

    // If method not found, try to find in mixins
    if resolved_methods.is_empty()
        && let Some(fq_class_id) = first_class_id
        && let Some(class_metadata) = context.codebase.get_class_like(&fq_class_id)
        && !class_metadata.mixins.is_empty()
        // Try to find method in mixin types
        && let Some(resolved_method) = find_static_method_in_mixins(
            context,
            block_context,
            &class_metadata.mixins,
            method_name,
            selector,
            access_span,
        )
    {
        if magic_call_could_exist {
            resolved_methods.push(resolved_method);
        } else {
            if class_metadata.flags.is_final() {
                report_non_existent_mixin_static_method(
                    context,
                    class_span,
                    method_span,
                    fq_class_id,
                    method_name,
                    resolved_method.classname,
                );

                result.has_invalid_target = true;
            } else {
                report_possibly_non_existent_mixin_static_method(
                    context,
                    class_span,
                    method_span,
                    fq_class_id,
                    method_name,
                    resolved_method.classname,
                );
            }

            resolved_methods.push(resolved_method);
        }
    }

    if resolved_methods.is_empty() {
        if let Some(fq_class_id) = first_class_id {
            result.has_invalid_target = true;

            if !could_method_ever_exist {
                if magic_call_could_exist {
                    report_non_documented_method(context, class_span, method_span, fq_class_id, method_name);
                } else {
                    report_non_existent_method(context, class_span, method_span, fq_class_id, method_name);
                }
            }
        } else {
            result.has_ambiguous_target = true;
        }
    }

    resolved_methods
}

fn resolve_method_from_metadata<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    current_class_metadata: Option<&'ctx ClassLikeMetadata>,
    method_name: Atom,
    fq_class_id: Atom,
    defining_class_metadata: &'ctx ClassLikeMetadata,
    classname: &ResolvedClassname,
    result: Option<&mut MethodResolutionResult>,
    selector: &ClassLikeMemberSelector<'arena>,
    access_span: Span,
    class_span: Span,
    has_magic_static_call: bool,
) -> Option<ResolvedMethod> {
    let method_id = MethodIdentifier::new(atom(&defining_class_metadata.original_name), atom(&method_name));
    let declaring_method_id = context.codebase.get_declaring_method_identifier(&method_id);
    let function_like = context.codebase.get_method_by_id(&declaring_method_id)?;

    if let Some(result) = result
        && !check_method_visibility(
            context,
            block_context,
            method_id.get_class_name(),
            method_id.get_method_name(),
            access_span,
            Some(selector.span()),
        )
    {
        result.has_invalid_target = true;
    }

    if function_like.flags.is_magic_method() && !has_magic_static_call {
        let is_static = !classname.is_parent();

        report_magic_call_without_call_method(
            context,
            class_span,
            selector.span(),
            *method_id.get_class_name(),
            method_name,
            is_static,
        );
    }

    let static_class_type = if let Some(current_class_metadata) = current_class_metadata
        && classname.is_relative()
    {
        let object = get_metadata_object(context, current_class_metadata, current_class_metadata);

        StaticClassType::Object(object)
    } else if defining_class_metadata.kind.is_enum() {
        StaticClassType::Object(TObject::Enum(TEnum { name: defining_class_metadata.original_name, case: None }))
    } else if !classname.intersections.is_empty() {
        if let TAtomic::Object(object) = classname.get_object_type(context.codebase) {
            StaticClassType::Object(object)
        } else {
            StaticClassType::Name(fq_class_id)
        }
    } else {
        StaticClassType::Name(fq_class_id)
    };

    // Get the class it was called on - need original name for callable creation
    let called_on_class_metadata = context.codebase.get_class_like(&fq_class_id)?;

    Some(ResolvedMethod {
        // Use the original name of the class it was called on
        // This is important for first-class callables with static return types
        classname: called_on_class_metadata.original_name,
        method_identifier: declaring_method_id,
        static_class_type,
        is_static: function_like.method_metadata.as_ref().is_some_and(|m| m.is_static),
        mixin_without_magic_method: None,
    })
}

fn get_metadata_object<'ctx>(
    context: &Context<'ctx, '_>,
    class_like_metadata: &'ctx ClassLikeMetadata,
    current_class_metadata: &'ctx ClassLikeMetadata,
) -> TObject {
    if class_like_metadata.kind.is_enum() {
        return TObject::Enum(TEnum { name: class_like_metadata.original_name, case: None });
    }

    let mut intersections = vec![];
    for required_interface in &class_like_metadata.require_implements {
        let Some(interface_metadata) = context.codebase.get_interface(required_interface) else {
            continue;
        };

        let TObject::Named(mut interface_type) =
            get_metadata_object(context, interface_metadata, current_class_metadata)
        else {
            continue;
        };

        let interface_intersactions = std::mem::take(&mut interface_type.intersection_types);

        interface_type.is_this = false;
        intersections.push(TAtomic::Object(TObject::Named(interface_type)));
        if let Some(interface_intersactions) = interface_intersactions {
            intersections.extend(interface_intersactions);
        }
    }

    for required_class in &class_like_metadata.require_extends {
        let Some(parent_class_metadata) = context.codebase.get_class_like(required_class) else {
            continue;
        };

        let TObject::Named(mut parent_type) =
            get_metadata_object(context, parent_class_metadata, current_class_metadata)
        else {
            continue;
        };

        let parent_intersections = std::mem::take(&mut parent_type.intersection_types);

        parent_type.is_this = false;
        intersections.push(TAtomic::Object(TObject::Named(parent_type)));
        if let Some(parent_intersections) = parent_intersections {
            intersections.extend(parent_intersections);
        }
    }

    TObject::Named(TNamedObject {
        name: class_like_metadata.original_name,
        type_parameters: if class_like_metadata.template_types.is_empty() {
            None
        } else {
            Some(
                class_like_metadata
                    .template_types
                    .iter()
                    .map(|(parameter_name, template_map)| {
                        if let Some(parameter) = get_specialized_template_type(
                            context.codebase,
                            parameter_name,
                            &class_like_metadata.name,
                            current_class_metadata,
                            None,
                        ) {
                            parameter
                        } else {
                            let (defining_entry, constraint) = unsafe {
                                // SAFETY: `template_map` is guaranteed to have at least one entry
                                template_map.iter().next().unwrap_unchecked()
                            };

                            wrap_atomic(TAtomic::GenericParameter(TGenericParameter {
                                parameter_name: *parameter_name,
                                constraint: Box::new(constraint.clone()),
                                defining_entity: *defining_entry,
                                intersection_types: None,
                            }))
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        },
        is_this: true,
        intersection_types: if intersections.is_empty() { None } else { Some(intersections) },
        remapped_parameters: false,
    })
}

fn report_non_static_access(context: &mut Context, method_id: &MethodIdentifier, span: Span) {
    let method_name = method_id.get_method_name();
    let class_name = method_id.get_class_name();

    context.collector.report_with_code(
        IssueCode::InvalidStaticMethodAccess,
        Issue::error(format!("Cannot call non-static method `{class_name}::{method_name}` statically."))
            .with_annotation(Annotation::primary(span).with_message("This is a non-static method"))
            .with_help("To call this method, you must first create an instance of the class (e.g., `$obj = new MyClass(); $obj->method();`)."),
    );
}

fn report_static_call_on_interface(context: &mut Context, name: Atom, span: Span, from_class_string: bool) {
    if from_class_string {
        context.collector.report_with_code(
            IssueCode::PossiblyStaticAccessOnInterface,
            Issue::warning(format!("Potential static method call on interface `{name}` via `class-string`."))
                .with_annotation(
                    Annotation::primary(span)
                        .with_message("This `class-string` could resolve to an interface name at runtime"),
                )
                .with_note(
                    format!("While a `class-string<{name}>` can hold a concrete class name (which is valid), it can also hold the interface name itself, which would cause a fatal error.")
                )
                .with_help("Ensure the variable or expression always holds the name of a concrete class, not an interface."),
        );
    } else {
        context.collector.report_with_code(
            IssueCode::StaticAccessOnInterface,
            Issue::error(format!("Cannot call a static method directly on an interface (`{name}`)."))
                .with_annotation(Annotation::primary(span).with_message("This is a direct static call on an interface"))
                .with_note(
                    "Static methods belong to classes that implement behavior, not interfaces that only define contracts.",
                )
                .with_help("Call this method on a concrete class that implements this interface instead."),
        );
    }
}

fn report_deprecated_static_access_on_trait(context: &mut Context, name: Atom, span: Span) {
    context.collector.report_with_code(
        IssueCode::DeprecatedFeature,
        Issue::warning(format!("Calling static methods directly on traits (`{name}`) is deprecated."))
            .with_annotation(Annotation::primary(span).with_message("This is a trait"))
            .with_help("Static methods should be called on a class that uses the trait."),
    );
}

/// Reports a warning when a static method is found in a mixin but the target class lacks __callStatic.
/// This is a warning because a subclass might implement __callStatic.
fn report_possibly_non_existent_mixin_static_method(
    context: &mut Context,
    class_span: Span,
    selector_span: Span,
    classname: Atom,
    method_name: Atom,
    mixin_classname: Atom,
) {
    context.collector.report_with_code(
        IssueCode::PossiblyNonExistentMethod,
        Issue::warning(format!(
            "Static method `{method_name}` might not exist on type `{classname}` at runtime."
        ))
        .with_annotation(
            Annotation::primary(selector_span).with_message("Method might not exist"),
        )
        .with_annotation(
            Annotation::secondary(class_span).with_message(format!("On class `{classname}`")),
        )
        .with_note(format!(
            "The method `{method_name}` is defined in mixin class `{mixin_classname}`, but `{classname}` does not have a `__callStatic` method to forward the call."
        ))
        .with_note(
            "A subclass of this class could implement `__callStatic` to handle this, so the call might succeed at runtime."
        )
        .with_help(format!(
            "Add a `__callStatic` method to `{classname}`, or make `{classname}` final if this should be an error."
        )),
    );
}

/// Reports an error when a static method is found in a mixin but the target final class lacks __callStatic.
/// This is an error because no subclass can exist to implement __callStatic.
fn report_non_existent_mixin_static_method(
    context: &mut Context,
    class_span: Span,
    selector_span: Span,
    classname: Atom,
    method_name: Atom,
    mixin_classname: Atom,
) {
    context.collector.report_with_code(
        IssueCode::NonExistentMethod,
        Issue::error(format!(
            "Static method `{method_name}` does not exist on final type `{classname}`."
        ))
        .with_annotation(
            Annotation::primary(selector_span).with_message("Method does not exist"),
        )
        .with_annotation(
            Annotation::secondary(class_span).with_message(format!("On final class `{classname}`")),
        )
        .with_note(format!(
            "The method `{method_name}` is defined in mixin class `{mixin_classname}`, but `{classname}` is final and does not have a `__callStatic` method to forward the call."
        ))
        .with_help(format!(
            "Add a `__callStatic` method to `{classname}` to handle mixin method calls."
        )),
    );
}

/// Searches for a static method in mixin types.
/// Returns Some(ResolvedMethod) if found, None otherwise.
///
/// Note: For static method calls, we cannot resolve generic mixin types (e.g., `@mixin T`)
/// to concrete types because static calls don't have an instance with type parameters.
/// In such cases, we fall back to using the constraint type.
fn find_static_method_in_mixins<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    mixins: &[TUnion],
    method_name: Atom,
    selector: &ClassLikeMemberSelector<'arena>,
    access_span: Span,
) -> Option<ResolvedMethod> {
    for mixin_type in mixins {
        for mixin_atomic in mixin_type.types.as_ref() {
            match mixin_atomic {
                TAtomic::Object(TObject::Named(named)) => {
                    if let Some(result) = find_static_method_in_single_mixin(
                        context,
                        block_context,
                        named.name,
                        method_name,
                        selector,
                        access_span,
                    ) {
                        return Some(result);
                    }
                }
                TAtomic::Object(TObject::Enum(enum_type)) => {
                    if let Some(result) = find_static_method_in_single_mixin(
                        context,
                        block_context,
                        enum_type.name,
                        method_name,
                        selector,
                        access_span,
                    ) {
                        return Some(result);
                    }
                }
                TAtomic::GenericParameter(TGenericParameter { constraint, .. }) => {
                    // For static calls, we cannot resolve generic parameters since there's no instance.
                    // Fall back to using the constraint type.
                    for constraint_atomic in constraint.types.as_ref() {
                        let constraint_class_name = match constraint_atomic {
                            TAtomic::Object(TObject::Named(named)) => named.name,
                            TAtomic::Object(TObject::Enum(enum_type)) => enum_type.name,
                            _ => continue,
                        };

                        if let Some(result) = find_static_method_in_single_mixin(
                            context,
                            block_context,
                            constraint_class_name,
                            method_name,
                            selector,
                            access_span,
                        ) {
                            return Some(result);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    None
}

/// Searches for a static method in a single mixin class.
fn find_static_method_in_single_mixin<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    mixin_class_name: Atom,
    method_name: Atom,
    selector: &ClassLikeMemberSelector<'arena>,
    access_span: Span,
) -> Option<ResolvedMethod> {
    let mixin_metadata = context.codebase.get_class_like(&mixin_class_name)?;

    let method_id = MethodIdentifier::new(atom(&mixin_metadata.original_name), atom(&method_name));
    let declaring_method_id = context.codebase.get_declaring_method_identifier(&method_id);
    let function_like = context.codebase.get_method_by_id(&declaring_method_id)?;

    // Mixin methods should only be accessible if public
    if let Some(method_metadata) = &function_like.method_metadata
        && !method_metadata.visibility.is_public()
    {
        return None;
    }

    // Check visibility
    if !check_method_visibility(
        context,
        block_context,
        method_id.get_class_name(),
        method_id.get_method_name(),
        access_span,
        Some(selector.span()),
    ) {
        return None;
    }

    let static_class_type = if mixin_metadata.kind.is_enum() {
        StaticClassType::Object(TObject::Enum(TEnum { name: mixin_metadata.original_name, case: None }))
    } else {
        StaticClassType::Name(mixin_class_name)
    };

    Some(ResolvedMethod {
        classname: mixin_metadata.original_name,
        method_identifier: declaring_method_id,
        static_class_type,
        is_static: function_like.method_metadata.as_ref().is_some_and(|m| m.is_static),
        mixin_without_magic_method: None,
    })
}
