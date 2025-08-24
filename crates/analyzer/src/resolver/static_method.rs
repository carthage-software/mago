use mago_atom::Atom;
use mago_codex::get_class_like;
use mago_codex::get_declaring_method_identifier;
use mago_codex::get_interface;
use mago_codex::get_method_by_id;
use mago_codex::get_method_identifier;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::generic::TGenericParameter;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::r#enum::TEnum;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::get_specialized_template_type;
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
use crate::resolver::method::report_non_existent_method;
use crate::resolver::selector::resolve_member_selector;

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
) -> Result<MethodResolutionResult, AnalysisError> {
    let mut result = MethodResolutionResult::default();

    let class_resolutions = resolve_classnames_from_expression(context, block_context, artifacts, class_expr, false)?;
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
                block_context.scope.get_class_like(),
                *method_name,
                class_expr.span(),
                method_selector.span(),
                resolved_classname,
                &mut result,
            );

            result.resolved_methods.extend(resolved_methods);
        }
    }

    Ok(result)
}

fn resolve_method_from_classname<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    current_class_metadata: Option<&'ctx ClassLikeMetadata>,
    method_name: Atom,
    class_span: Span,
    method_span: Span,
    classname: &ResolvedClassname,
    result: &mut MethodResolutionResult,
) -> Vec<ResolvedMethod> {
    let mut resolve_method_from_class_id =
        |fq_class_id: Atom,
         is_relative: bool,
         from_instance: bool,
         from_class_string: bool,
         result: &mut MethodResolutionResult| {
            let Some(defining_class_metadata) = get_class_like(context.codebase, &fq_class_id) else {
                return (false, None);
            };

            if !from_instance && !is_relative && defining_class_metadata.kind.is_interface() {
                report_static_call_on_interface(
                    context,
                    &defining_class_metadata.original_name,
                    class_span,
                    from_class_string,
                );

                if !from_class_string {
                    result.has_invalid_target = true;
                    return (true, None);
                }
            }

            let Some(method) = resolve_method_from_metadata(
                context,
                current_class_metadata,
                method_name,
                &fq_class_id,
                defining_class_metadata,
                classname,
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
                report_deprecated_static_access_on_trait(context, &defining_class_metadata.original_name, class_span);
            }

            (true, Some(method))
        };

    let mut resolved_methods = vec![];
    let mut could_method_ever_exist = false;
    let mut first_class_id = None;
    if let Some(fq_class_id) = classname.fqcn {
        let (could_method_exist, resolved_method) = resolve_method_from_class_id(
            fq_class_id,
            classname.is_relative(),
            classname.is_object_instance(),
            classname.is_from_class_string(),
            result,
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

        let (could_method_exist, resolved_method) = resolve_method_from_class_id(
            fq_class_id,
            intersection.is_relative() || classname.is_relative(),
            intersection.is_object_instance() || classname.is_object_instance(),
            intersection.is_from_class_string(),
            result,
        );

        if let Some(resolved_method) = resolved_method {
            resolved_methods.push(resolved_method);
        }

        could_method_ever_exist |= could_method_exist;
        if first_class_id.is_none() {
            first_class_id = Some(fq_class_id);
        }
    }

    if resolved_methods.is_empty() {
        if let Some(fq_class_id) = first_class_id {
            result.has_invalid_target = true;

            if !could_method_ever_exist {
                report_non_existent_method(context, class_span, method_span, &fq_class_id, &method_name);
            }
        } else {
            result.has_ambiguous_target = true;
        }
    }

    resolved_methods
}

fn resolve_method_from_metadata<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    current_class_metadata: Option<&'ctx ClassLikeMetadata>,
    method_name: Atom,
    fq_class_id: &Atom,
    defining_class_metadata: &'ctx ClassLikeMetadata,
    classname: &ResolvedClassname,
) -> Option<ResolvedMethod> {
    let method_id = get_method_identifier(&defining_class_metadata.original_name, &method_name);
    let declaring_method_id = get_declaring_method_identifier(context.codebase, &method_id);
    let function_like = get_method_by_id(context.codebase, &declaring_method_id)?;

    let static_class_type = if let Some(current_class_metadata) = current_class_metadata
        && classname.is_relative()
    {
        let object = if classname.is_parent() {
            get_metadata_object(context, defining_class_metadata, current_class_metadata)
        } else {
            get_metadata_object(context, current_class_metadata, current_class_metadata)
        };

        StaticClassType::Object(object)
    } else {
        StaticClassType::Name(*fq_class_id)
    };

    Some(ResolvedMethod {
        classname: defining_class_metadata.name,
        method_identifier: declaring_method_id,
        static_class_type,
        is_static: function_like.method_metadata.as_ref().is_some_and(|m| m.is_static),
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
        let Some(interface_metadata) = get_interface(context.codebase, required_interface) else {
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
        let Some(parent_class_metadata) = get_class_like(context.codebase, required_class) else {
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
        type_parameters: if !class_like_metadata.template_types.is_empty() {
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
        } else {
            None
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

fn report_static_call_on_interface(context: &mut Context, name: &Atom, span: Span, from_class_string: bool) {
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

fn report_deprecated_static_access_on_trait(context: &mut Context, name: &Atom, span: Span) {
    context.collector.report_with_code(
        IssueCode::DeprecatedFeature,
        Issue::warning(format!("Calling static methods directly on traits (`{name}`) is deprecated."))
            .with_annotation(Annotation::primary(span).with_message("This is a trait"))
            .with_help("Static methods should be called on a class that uses the trait."),
    );
}
