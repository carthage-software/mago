use std::rc::Rc;

use mago_codex::ttype::TType;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::artifacts::AnalysisArtifacts;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::resolver::property::resolve_instance_properties;
use crate::utils::expression::get_property_access_expression_id;

#[inline]
pub fn analyze<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    property_access: &PropertyAccess<'arena>,
    assigned_value_type: &TUnion,
    assigned_value_span: Option<Span>,
) -> Result<(), AnalysisError> {
    let property_access_id = get_property_access_expression_id(
        property_access.object,
        &property_access.property,
        false,
        block_context.scope.get_class_like_name(),
        context.resolved_names,
        Some(context.codebase),
    );

    let was_inside_assignment = block_context.inside_assignment;
    block_context.inside_assignment = true;
    let resolution_result = resolve_instance_properties(
        context,
        block_context,
        artifacts,
        property_access.object,
        &property_access.property,
        property_access.arrow.span(),
        false, // `null_safe`
        true,  // `for_assignment`
    )?;
    block_context.inside_assignment = was_inside_assignment;

    let mut resolved_property_type = None;
    let mut matched_all_properties = true;
    for resolved_property in resolution_result.properties {
        let mut union_comparison_result = ComparisonResult::new();

        let type_match_found = union_comparator::is_contained_by(
            context.codebase,
            assigned_value_type,
            &resolved_property.property_type,
            true,
            assigned_value_type.ignore_falsable_issues,
            false,
            &mut union_comparison_result,
        );

        if !type_match_found {
            let property_name = resolved_property.property_name;
            let property_type_str = resolved_property.property_type.get_id();
            let assigned_type_str = assigned_value_type.get_id();

            let mut issue;

            if let Some(true) = union_comparison_result.type_coerced {
                let issue_kind;

                if union_comparison_result.type_coerced_from_nested_mixed.unwrap_or(false) {
                    issue_kind = IssueCode::MixedPropertyTypeCoercion;
                    issue = Issue::error(format!(
                        "A value with a less specific type `{assigned_type_str}` is being assigned to property `${property_name}` ({property_type_str})."
                    ))
                    .with_note("The assigned value contains a nested `mixed` type, which can hide potential bugs.");
                } else {
                    issue_kind = IssueCode::PropertyTypeCoercion;
                    issue = Issue::error(format!(
                        "A value of a less specific type `{assigned_type_str}` is being assigned to property `${property_name}` ({property_type_str})."
                    ))
                    .with_note(format!("While `{assigned_type_str}` can be assigned to `{property_type_str}`, it is a wider type which may accept values that are invalid for this property."));
                }

                if let Some(value_span) = assigned_value_span {
                    issue = issue.with_annotation(
                        Annotation::primary(value_span)
                            .with_message(format!("This value has the less specific type `{assigned_type_str}`")),
                    );
                } else {
                    issue = issue.with_annotation(
                        Annotation::primary(property_access.span())
                            .with_message("The value assigned to this property is of a less specific type"),
                    );
                }

                if let Some(property_span) = resolved_property.property_span {
                    issue = issue.with_annotation(Annotation::secondary(property_span).with_message(format!(
                        "This property `{property_name}` is declared with type `{property_type_str}`"
                    )));
                }

                context.collector.report_with_code(
                    issue_kind,
                    issue.with_help(
                        "Consider adding a type assertion to narrow the type of the value before the assignment.",
                    ),
                );
            } else {
                if let Some(value_span) = assigned_value_span {
                    issue = Issue::error(format!(
                        "Invalid type for property `{property_name}`: expected `{property_type_str}`, but got `{assigned_type_str}`."
                    ))
                    .with_annotation(
                        Annotation::primary(value_span)
                            .with_message(format!("This expression has type `{assigned_type_str}`")),
                    );
                } else {
                    issue = Issue::error(format!(
                        "Invalid assignment to property `{property_name}`: cannot assign value of type `{assigned_type_str}` to expected type `{property_type_str}`."
                    ))
                    .with_annotation(
                        Annotation::primary(property_access.span())
                            .with_message("The value assigned to this property is of an incompatible type"),
                    );
                }

                if let Some(property_span) = resolved_property.property_span {
                    issue = issue.with_annotation(Annotation::secondary(property_span).with_message(format!(
                        "This property `{property_name}` is declared with type `{property_type_str}`"
                    )));
                }

                context.collector.report_with_code(
                    IssueCode::InvalidPropertyAssignmentValue,
                    issue
                         .with_note(format!("The type `{assigned_type_str}` is not compatible with and cannot be assigned to `{property_type_str}`."))
                         .with_help("Change the assigned value to match the property's type, or update the property's type declaration."),
                );
            }
        }

        resolved_property_type = Some(add_optional_union_type(
            resolved_property.property_type,
            resolved_property_type.as_ref(),
            context.codebase,
        ));

        matched_all_properties &= type_match_found;
    }

    let mut resulting_type = if matched_all_properties && context.settings.memoize_properties {
        Some(assigned_value_type.clone())
    } else {
        resolved_property_type
    };

    if resolution_result.has_ambiguous_path
        || resolution_result.encountered_mixed
        || resolution_result.has_possibly_defined_property
    {
        resulting_type = Some(add_optional_union_type(get_mixed(), resulting_type.as_ref(), context.codebase));
    }

    if resolution_result.has_error_path || resolution_result.has_invalid_path || resolution_result.encountered_null {
        resulting_type = Some(add_optional_union_type(get_never(), resulting_type.as_ref(), context.codebase));
    }

    let resulting_type = Rc::new(resulting_type.unwrap_or_else(get_never));

    if context.settings.memoize_properties
        && let Some(property_access_id) = property_access_id
    {
        block_context.locals.insert(property_access_id, resulting_type.clone());
    }

    artifacts.set_rc_expression_type(property_access, resulting_type);

    Ok(())
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::test_analysis;

    test_analysis! {
        name = memoized_property_assignment,
        code = indoc! {r#"
            <?php

            class A {
                /** @var int<0, max> */
                private int $a = 0;

                public function work(): void {
                    $this->a++;
                    $this->a--;
                    $this->a += 5;
                    $this->a -= 2;
                    $this->a *= 2;
                    $this->a %= 2;
                    $this->a = 1;
                    $this->a = 0;
                    ++$this->a;
                    --$this->a;
                }
            }
        "#}
    }
}
