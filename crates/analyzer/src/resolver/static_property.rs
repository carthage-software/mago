use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;
use mago_codex::get_class_like;
use mago_codex::get_declaring_class_for_property;
use mago_codex::ttype::expander::StaticClassType;
use mago_codex::ttype::expander::TypeExpansionOptions;
use mago_codex::ttype::expander::{self};
use mago_codex::ttype::get_mixed;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::resolver::class_name::resolve_classnames_from_expression;
use crate::resolver::property::PropertyResolutionResult;
use crate::resolver::property::ResolvedProperty;
use crate::visibility::check_property_read_visibility;

/// Resolves all possible static properties from a class expression and a member selector.
pub fn resolve_static_properties<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    class_expression: &'ast Expression<'arena>,
    property_variable: &'ast Variable<'arena>,
) -> Result<PropertyResolutionResult, AnalysisError> {
    let mut result = PropertyResolutionResult::default();

    let classnames = resolve_classnames_from_expression(context, block_context, artifacts, class_expression, false)?;

    let mut property_names = vec![];

    'resolve_names: {
        let variable_type = match property_variable {
            Variable::Direct(direct_variable) => {
                property_names.push(atom(direct_variable.name));

                break 'resolve_names;
            }
            Variable::Indirect(indirect_variable) => {
                let was_inside_general_use = block_context.inside_general_use;
                block_context.inside_general_use = true;
                indirect_variable.expression.analyze(context, block_context, artifacts)?;
                block_context.inside_general_use = was_inside_general_use;

                artifacts.get_rc_expression_type(indirect_variable.expression)
            }
            Variable::Nested(nested_variable) => {
                let was_inside_general_use = block_context.inside_general_use;
                block_context.inside_general_use = true;
                nested_variable.variable.analyze(context, block_context, artifacts)?;
                block_context.inside_general_use = was_inside_general_use;

                artifacts.get_rc_expression_type(nested_variable.variable)
            }
        };

        let Some(variable_type) = variable_type else {
            result.has_invalid_path = true;
            break 'resolve_names;
        };

        for variable_atomic_type in variable_type.types.as_ref() {
            let Some(property_name) = variable_atomic_type.get_literal_string_value() else {
                if variable_atomic_type.is_any_string() {
                    result.has_ambiguous_path = true;
                } else {
                    result.has_invalid_path = true;
                }

                continue;
            };

            property_names.push(concat_atom!("$", property_name));
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

        let Some(fqcn) = resolved_classname.fqcn else {
            result.has_ambiguous_path = true;
            continue;
        };

        for property_name in &property_names {
            if let Some(resolved_property) = find_static_property_in_class(
                context,
                block_context,
                &fqcn,
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
fn find_static_property_in_class<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &BlockContext<'ctx>,
    class_id: &Atom,
    property_name: &Atom,
    variable: &'ast Variable<'arena>,
    class_expr: &'ast Expression<'arena>,
    result: &mut PropertyResolutionResult,
) -> Result<Option<ResolvedProperty>, AnalysisError> {
    let Some(class_metadata) = get_class_like(context.codebase, class_id) else {
        // Error reporting for non-existent class is handled by `resolve_classnames_from_expression`.
        result.has_invalid_path = true;
        return Ok(None);
    };

    let declaring_class_id = get_declaring_class_for_property(context.codebase, class_id, property_name)
        .unwrap_or(class_metadata.original_name);

    let Some(declaring_class_metadata) = get_class_like(context.codebase, &declaring_class_id) else {
        // Should not happen if declaring_class_id is valid.
        result.has_error_path = true;
        return Ok(None);
    };

    let Some(property_metadata) = declaring_class_metadata.properties.get(property_name) else {
        result.has_invalid_path = true;
        report_non_existent_property(
            context,
            &declaring_class_metadata.original_name,
            property_name,
            variable.span(),
            class_expr.span(),
        );

        return Ok(None);
    };

    if !property_metadata.flags.is_static() {
        let classname = declaring_class_metadata.original_name;

        context.collector.report_with_code(
            IssueCode::InvalidStaticPropertyAccess,
            Issue::error(format!("Cannot access instance property `{classname}::{property_name}` statically."))
                .with_annotation(Annotation::primary(variable.span()).with_message("This is an instance property"))
                .with_note("Static properties are declared with the `static` keyword and accessed with `::` on a class name, not an instance.")
                .with_help(format!("To access this property, you need an instance of the class (e.g., `$instance->{property_name}`), or declare the property as `static`.")),
        );

        result.has_error_path = true;
        return Ok(None);
    }

    if !check_property_read_visibility(
        context,
        block_context,
        &declaring_class_id,
        property_name,
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
        &mut property_type,
        &TypeExpansionOptions {
            self_class: Some(declaring_class_id),
            static_class_type: StaticClassType::Name(*class_id),
            parent_class: declaring_class_metadata.direct_parent_class,
            ..Default::default()
        },
    );

    Ok(Some(ResolvedProperty {
        property_span: property_metadata.name_span.or(property_metadata.span),
        property_name: *property_name,
        declaring_class_id,
        property_type,
    }))
}

fn report_non_existent_property<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    classname: &Atom,
    property_name: &Atom,
    selector_span: Span,
    class_like_name_span: Span,
) {
    let class_kind_str = get_class_like(context.codebase, classname).map_or("class", |m| m.kind.as_str());

    context.collector.report_with_code(
        IssueCode::NonExistentProperty,
        Issue::error(format!("Static property `{property_name}` does not exist on {class_kind_str} `{classname}`."))
            .with_annotation(
                Annotation::primary(selector_span)
                    .with_message("This selector refers to a non-existent static property"),
            )
            .with_annotation(Annotation::secondary(class_like_name_span).with_message(format!(
                "The {class_kind_str} `{classname}` does not have a static property named `{property_name}`",
            ))),
    );
}
