use mago_codex::get_class_like;
use mago_codex::get_declaring_method_id;
use mago_codex::get_interface;
use mago_codex::get_method_by_id;
use mago_codex::get_method_id;
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
use mago_interner::StringIdentifier;
use mago_php_version::feature::Feature;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::ClassLikeMemberSelector;
use mago_syntax::ast::Expression;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;
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
pub fn resolve_static_method_targets<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    class_expr: &Expression,
    method_selector: &ClassLikeMemberSelector,
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

fn resolve_method_from_classname<'a>(
    context: &mut Context<'a>,
    current_class_metadata: Option<&'a ClassLikeMetadata>,
    method_name: StringIdentifier,
    class_span: Span,
    method_span: Span,
    classname: &ResolvedClassname,
    result: &mut MethodResolutionResult,
) -> Vec<ResolvedMethod> {
    let mut resolve_method_from_class_id = |fq_class_id, result: &mut MethodResolutionResult| {
        let defining_class_metadata = get_class_like(context.codebase, context.interner, &fq_class_id)?;

        if !classname.is_object_instance() && defining_class_metadata.kind.is_interface() {
            report_static_access_on_interface(context, &defining_class_metadata.original_name, class_span);
            result.has_invalid_target = true;
            return None;
        }

        let method = resolve_method_from_metadata(
            context,
            current_class_metadata,
            method_name,
            &fq_class_id,
            defining_class_metadata,
            classname,
        )?;

        if !method.is_static
            && !classname.is_relative()
            && !current_class_metadata.is_some_and(|current_class_metadata| {
                current_class_metadata.name == defining_class_metadata.name
                    || current_class_metadata.has_parent(&defining_class_metadata.name)
            })
        {
            report_non_static_access(context, &method.method_identifier, method_span);
            return None;
        }

        if method.is_static
            && defining_class_metadata.kind.is_trait()
            && context.settings.version.is_deprecated(Feature::CallStaticMethodOnTrait)
        {
            report_deprecated_static_access_on_trait(context, &defining_class_metadata.original_name, class_span);
        }

        Some(method)
    };

    let mut resolved_methods = vec![];
    if let Some(fq_class_id) = classname.fq_class_id
        && let Some(resolved_method) = resolve_method_from_class_id(fq_class_id, result)
    {
        resolved_methods.push(resolved_method);
    }

    for intersection in classname.intersections.iter().filter_map(|c| c.fq_class_id) {
        if let Some(resolved_method) = resolve_method_from_class_id(intersection, result) {
            resolved_methods.push(resolved_method);
        }
    }

    if resolved_methods.is_empty() {
        let fq_class_id = classname
            .fq_class_id
            .or_else(|| classname.intersections.iter().find_map(|c| c.fq_class_id).iter().next().copied());

        if let Some(fq_class_id) = fq_class_id {
            result.has_invalid_target = true;
            report_non_existent_method(context, class_span, method_span, method_name, &fq_class_id);
        } else {
            result.has_ambiguous_target = true;
        }
    }

    resolved_methods
}

fn resolve_method_from_metadata<'a>(
    context: &mut Context<'a>,
    current_class_metadata: Option<&'a ClassLikeMetadata>,
    method_name: StringIdentifier,
    fq_class_id: &StringIdentifier,
    defining_class_metadata: &'a ClassLikeMetadata,
    classname: &ResolvedClassname,
) -> Option<ResolvedMethod> {
    let method_id = get_method_id(&defining_class_metadata.original_name, &method_name);
    let declaring_method_id = get_declaring_method_id(context.codebase, context.interner, &method_id);
    let function_like = get_method_by_id(context.codebase, context.interner, &declaring_method_id)?;

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
        is_static: function_like.get_method_metadata().is_some_and(|m| m.is_static()),
    })
}

fn get_metadata_object<'a>(
    context: &Context<'a>,
    class_like_metadata: &'a ClassLikeMetadata,
    current_class_metadata: &'a ClassLikeMetadata,
) -> TObject {
    if class_like_metadata.kind.is_enum() {
        return TObject::Enum(TEnum { name: class_like_metadata.original_name, case: None });
    }

    let mut intersections = vec![];
    for required_interface in &class_like_metadata.require_implements {
        let Some(interface_metadata) = get_interface(context.codebase, context.interner, required_interface) else {
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
        let Some(parent_class_metadata) = get_class_like(context.codebase, context.interner, required_class) else {
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
                            context.interner,
                            parameter_name,
                            &class_like_metadata.name,
                            current_class_metadata,
                            None,
                        ) {
                            parameter
                        } else {
                            let (defining_entry, constraint) = template_map.iter().next().unwrap();

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
    let method_name = context.interner.lookup(method_id.get_method_name());
    let class_name = context.interner.lookup(method_id.get_class_name());
    context.buffer.report(
        TypingIssueKind::InvalidStaticMethodAccess,
        Issue::error(format!("Cannot call non-static method `{class_name}::{method_name}` statically."))
            .with_annotation(Annotation::primary(span).with_message("This is a non-static method"))
            .with_help("To call this method, you must first create an instance of the class (e.g., `$obj = new MyClass(); $obj->method();`)."),
    );
}

fn report_static_access_on_interface(context: &mut Context, name: &StringIdentifier, span: Span) {
    let name_str = context.interner.lookup(name);
    context.buffer.report(
        TypingIssueKind::StaticAccessOnInterface,
        Issue::error(format!("Cannot make a static access on an interface (`{name_str}`)."))
            .with_annotation(Annotation::primary(span).with_message("This is an interface"))
            .with_note(
                "Static methods belong to classes that implement behavior, not interfaces that define contracts.",
            ),
    );
}

fn report_deprecated_static_access_on_trait(context: &mut Context, name: &StringIdentifier, span: Span) {
    let name_str = context.interner.lookup(name);
    context.buffer.report(
        TypingIssueKind::DeprecatedFeature,
        Issue::warning(format!("Calling static methods directly on traits (`{name_str}`) is deprecated."))
            .with_annotation(Annotation::primary(span).with_message("This is a trait"))
            .with_help("Static methods should be called on a class that uses the trait."),
    );
}
