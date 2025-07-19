use mago_codex::get_class_like;
use mago_codex::get_declaring_class_for_property;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::{self};
use mago_codex::ttype::get_mixed;
use mago_interner::StringIdentifier;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;
use crate::resolver::class_name::resolve_classnames_from_expression;
use crate::resolver::property::PropertyResolutionResult;
use crate::resolver::property::ResolvedProperty;
use crate::visibility::check_property_read_visibility;

/// Resolves all possible static properties from a class expression and a member selector.
pub fn resolve_static_properties<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    class_expression: &Expression,
    property_variable: &Variable,
) -> Result<PropertyResolutionResult, AnalysisError> {
    let mut result = PropertyResolutionResult::default();

    let classnames = resolve_classnames_from_expression(context, block_context, artifacts, class_expression, false)?;

    let mut property_names = vec![];

    'resolve_names: {
        let variable_type = match property_variable {
            Variable::Direct(direct_variable) => {
                property_names.push(direct_variable.name);

                break 'resolve_names;
            }
            Variable::Indirect(indirect_variable) => {
                let was_inside_general_use = block_context.inside_general_use;
                block_context.inside_general_use = true;
                indirect_variable.expression.analyze(context, block_context, artifacts)?;
                block_context.inside_general_use = was_inside_general_use;

                artifacts.get_rc_expression_type(indirect_variable.expression.as_ref())
            }
            Variable::Nested(nested_variable) => {
                let was_inside_general_use = block_context.inside_general_use;
                block_context.inside_general_use = true;
                nested_variable.variable.analyze(context, block_context, artifacts)?;
                block_context.inside_general_use = was_inside_general_use;

                artifacts.get_rc_expression_type(nested_variable.variable.as_ref())
            }
        };

        let Some(variable_type) = variable_type else {
            result.has_invalid_path = true;
            break 'resolve_names;
        };

        for variable_atomic_type in &variable_type.types {
            let Some(property_name) = variable_atomic_type.get_literal_string_value() else {
                if variable_atomic_type.is_any_string() {
                    result.has_ambiguous_path = true;
                } else {
                    result.has_invalid_path = true;
                }

                continue;
            };

            let property_name = "$".to_string() + property_name;
            let property_name_id = context.interner.intern(property_name);

            property_names.push(property_name_id);
        }
    };

    for resolved_classname in classnames {
        if resolved_classname.is_from_mixed() {
            result.encountered_mixed = true;
            continue;
        }

        if resolved_classname.is_possibly_invalid() {
            result.has_invalid_path = true;
            continue;
        }

        let Some(fq_class_id) = resolved_classname.fq_class_id else {
            result.has_ambiguous_path = true;
            continue;
        };

        for property_name in &property_names {
            if let Some(resolved_property) = find_static_property_in_class(
                context,
                block_context,
                &fq_class_id,
                property_name,
                property_variable,
                class_expression,
                &mut result,
            )? {
                result.properties.push(resolved_property);
            }
        }
    }

    Ok(result)
}

/// Finds a static property in a class, gets its type, and handles template localization.
fn find_static_property_in_class<'a>(
    context: &mut Context<'a>,
    block_context: &BlockContext<'a>,
    class_id: &StringIdentifier,
    prop_name: &StringIdentifier,
    variable: &Variable,
    class_expr: &Expression,
    result: &mut PropertyResolutionResult,
) -> Result<Option<ResolvedProperty>, AnalysisError> {
    let Some(class_metadata) = get_class_like(context.codebase, context.interner, class_id) else {
        // Error reporting for non-existent class is handled by `resolve_classnames_from_expression`.
        result.has_invalid_path = true;
        return Ok(None);
    };

    let declaring_class_id = get_declaring_class_for_property(context.codebase, context.interner, class_id, prop_name)
        .unwrap_or(class_metadata.original_name);

    let Some(declaring_class_metadata) = get_class_like(context.codebase, context.interner, &declaring_class_id) else {
        // Should not happen if declaring_class_id is valid.
        result.has_error_path = true;
        return Ok(None);
    };

    let Some(property_metadata) = declaring_class_metadata.properties.get(prop_name) else {
        result.has_invalid_path = true;
        report_non_existent_property(
            context,
            &declaring_class_metadata.original_name,
            prop_name,
            variable.span(),
            class_expr.span(),
        );

        return Ok(None);
    };

    if !property_metadata.is_static {
        let class_name_str = context.interner.lookup(&declaring_class_metadata.original_name);
        let prop_name_str = context.interner.lookup(prop_name);

        context.buffer.report(
            TypingIssueKind::InvalidStaticPropertyAccess,
            Issue::error(format!("Cannot access instance property `{}::{}` statically.", class_name_str, prop_name_str))
                .with_annotation(Annotation::primary(variable.span()).with_message("This is an instance property"))
                .with_note("Static properties are declared with the `static` keyword and accessed with `::` on a class name, not an instance.")
                .with_help(format!("To access this property, you need an instance of the class (e.g., `$instance->{}`), or declare the property as `static`.", prop_name_str)),
        );

        result.has_error_path = true;
        return Ok(None);
    }

    if !check_property_read_visibility(
        context,
        block_context,
        &declaring_class_id,
        prop_name,
        class_expr.span(),
        Some(variable.span()),
    ) {
        result.has_error_path = true;
        return Ok(None);
    }

    let mut property_type =
        property_metadata.type_metadata.as_ref().map(|metadata| metadata.type_union.clone()).unwrap_or_else(get_mixed);

    expander::expand_union(
        context.codebase,
        context.interner,
        &mut property_type,
        &TypeExpansionOptions {
            self_class: Some(&declaring_class_id),
            static_class_type: StaticClassType::Name(*class_id),
            parent_class: declaring_class_metadata.direct_parent_class.as_ref(),
            file_path: Some(&context.source.identifier),
            ..Default::default()
        },
    );

    Ok(Some(ResolvedProperty {
        property_span: property_metadata.get_name_span(),
        property_name: *prop_name,
        declaring_class_id,
        property_type,
    }))
}

fn report_non_existent_property(
    context: &mut Context,
    classname: &StringIdentifier,
    prop_name: &StringIdentifier,
    selector_span: Span,
    class_like_name_span: Span,
) {
    let class_name_str = context.interner.lookup(classname);
    let property_name_str = context.interner.lookup(prop_name);
    let class_kind_str =
        get_class_like(context.codebase, context.interner, classname).map_or("class", |m| m.kind.as_str());

    context.buffer.report(
        TypingIssueKind::NonExistentProperty,
        Issue::error(format!(
            "Static property `{property_name_str}` does not exist on {class_kind_str} `{class_name_str}`."
        ))
        .with_annotation(
            Annotation::primary(selector_span).with_message("This selector refers to a non-existent static property"),
        )
        .with_annotation(Annotation::secondary(class_like_name_span).with_message(format!(
            "The {class_kind_str} `{class_name_str}` does not have a static property named `{property_name_str}`",
        ))),
    );
}
