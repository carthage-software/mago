use mago_codex::get_class_like;
use mago_codex::get_declaring_method_id;
use mago_codex::get_method_by_id;
use mago_codex::get_method_id;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::metadata::class_like::ClassLikeMetadata;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::object::named::TNamedObject;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::get_specialized_template_type;
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

    for class_res in &class_resolutions {
        if class_res.is_possibly_invalid() {
            result.has_ambiguous_target = true;
            if class_res.origin == crate::resolver::class_name::ResolutionOrigin::Invalid {
                result.has_invalid_target = true;
            }
            continue;
        }

        let Some(fq_class_id) = class_res.fq_class_id else {
            result.has_ambiguous_target = true;
            continue;
        };

        let Some(metadata) = get_class_like(context.codebase, context.interner, &fq_class_id) else {
            result.has_invalid_target = true;
            continue;
        };

        if !class_res.is_object_instance() && metadata.is_interface() {
            report_static_access_on_interface(context, &metadata.original_name, class_expr.span());
            result.has_invalid_target = true;
            continue;
        }

        for method_name in &method_names {
            match find_static_method_in_class(
                context,
                block_context.scope.get_class_like(),
                metadata,
                *method_name,
                &fq_class_id,
                class_expr.span(),
                method_selector.span(),
                class_res.is_relative(),
            ) {
                Some(resolved_method) => {
                    result.resolved_methods.push(resolved_method);
                }
                None => {
                    result.has_invalid_target = true;
                }
            }
        }
    }

    Ok(result)
}

/// Finds a single static method in a class and validates it.
fn find_static_method_in_class<'a>(
    context: &mut Context<'a>,
    current_class_metadata: Option<&'a ClassLikeMetadata>,
    defining_class_metadata: &'a ClassLikeMetadata,
    method_name: StringIdentifier,
    fq_class_id: &StringIdentifier,
    class_span: Span,
    method_span: Span,
    is_relative: bool,
) -> Option<ResolvedMethod> {
    let method_id = get_method_id(&defining_class_metadata.original_name, &method_name);
    let declaring_method_id = get_declaring_method_id(context.codebase, context.interner, &method_id);

    let Some(function_like_metadata) = get_method_by_id(context.codebase, context.interner, &declaring_method_id)
    else {
        report_non_existent_method(context, class_span, method_span, method_name, fq_class_id);
        return None;
    };

    let is_method_static = function_like_metadata.get_method_metadata().is_some_and(|m| m.is_static());

    if !is_method_static
        && !is_relative
        && !current_class_metadata.is_some_and(|current_class_metadata| {
            current_class_metadata.name == defining_class_metadata.name
                || current_class_metadata.has_parent(&defining_class_metadata.name)
        })
    {
        report_non_static_access(context, &declaring_method_id, method_span);
        return None;
    } else if is_method_static
        && defining_class_metadata.is_trait()
        && context.settings.version.is_deprecated(Feature::CallStaticMethodOnTrait)
    {
        report_deprecated_static_access_on_trait(context, &defining_class_metadata.original_name, class_span);
    }

    let static_class_type = if let Some(current_class_metadata) = current_class_metadata
        && is_relative
    {
        let mut type_parameters = vec![];
        for (template_name, _) in &defining_class_metadata.template_types {
            let template_name_str = context.interner.lookup(template_name);

            let Some(parameter) = get_specialized_template_type(
                context.codebase,
                context.interner,
                template_name_str,
                &defining_class_metadata.name,
                current_class_metadata,
                None,
            ) else {
                let defining_class_str = context.interner.lookup(&defining_class_metadata.original_name);

                panic!("Failed to get specialized template type for {template_name_str} in {defining_class_str}");
            };

            type_parameters.push(parameter);
        }

        let object = if type_parameters.is_empty() {
            TObject::Named(TNamedObject::new(defining_class_metadata.original_name))
        } else {
            TObject::Named(TNamedObject::new_with_type_parameters(
                defining_class_metadata.original_name,
                Some(type_parameters),
            ))
        };

        StaticClassType::Object(object)
    } else {
        StaticClassType::Name(*fq_class_id)
    };

    Some(ResolvedMethod {
        classname: defining_class_metadata.name,
        method_identifier: declaring_method_id,
        static_class_type,
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
