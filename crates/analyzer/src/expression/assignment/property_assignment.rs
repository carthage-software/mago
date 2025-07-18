use std::rc::Rc;

use mago_codex::data_flow::graph::GraphKind;
use mago_codex::data_flow::node::DataFlowNode;
use mago_codex::data_flow::path::PathKind;
use mago_codex::get_class_like;
use mago_codex::ttype::TType;
use mago_codex::ttype::add_optional_union_type;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_interner::StringIdentifier;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::artifacts::AnalysisArtifacts;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::issue::TypingIssueKind;
use crate::resolver::property::get_nodes_for_property_access;
use crate::resolver::property::resolve_instance_properties;
use crate::utils::expression::get_expression_id;
use crate::utils::expression::get_property_access_expression_id;

#[inline]
pub fn analyze<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    property_access: &PropertyAccess,
    assigned_value_type: &TUnion,
    assigned_value_span: Option<Span>,
) -> Result<(), AnalysisError> {
    let property_access_id = get_property_access_expression_id(
        &property_access.object,
        &property_access.property,
        false,
        block_context.scope.get_class_like_name(),
        context.resolved_names,
        context.interner,
        Some(context.codebase),
    );

    let object_expression_id = get_expression_id(
        &property_access.object,
        None,
        context.resolved_names,
        context.interner,
        Some(context.codebase),
    );

    let was_inside_assignment = block_context.inside_assignment;
    block_context.inside_assignment = true;
    let resolution_result = resolve_instance_properties(
        context,
        block_context,
        artifacts,
        &property_access.object,
        &property_access.property,
        property_access.arrow.span(),
        false, // `null_safe`
        true,  // `for_assignment`
    )?;
    block_context.inside_assignment = was_inside_assignment;

    let mut resolved_property_type = None;
    let mut matched_all_properties = true;
    for resolved_property in resolution_result.properties {
        if let GraphKind::WholeProgram = artifacts.data_flow_graph.kind {
            add_instance_property_dataflow(
                context,
                block_context,
                artifacts,
                &object_expression_id,
                property_access.object.span(),
                property_access.property.span(),
                &resolved_property.declaring_class_id,
                &resolved_property.property_name,
                assigned_value_type,
            );
        }

        let mut union_comparison_result = ComparisonResult::new();

        let type_match_found = union_comparator::is_contained_by(
            context.codebase,
            context.interner,
            assigned_value_type,
            &resolved_property.property_type,
            true,
            assigned_value_type.ignore_falsable_issues,
            false,
            &mut union_comparison_result,
        );

        if !type_match_found {
            let property_name_str = context.interner.lookup(&resolved_property.property_name).replace('$', "");
            let property_type_str = resolved_property.property_type.get_id(Some(context.interner));
            let assigned_type_str = assigned_value_type.get_id(Some(context.interner));

            let mut issue;

            if let Some(true) = union_comparison_result.type_coerced {
                let issue_kind;

                if union_comparison_result.type_coerced_from_nested_mixed.unwrap_or(false) {
                    issue_kind = TypingIssueKind::MixedPropertyTypeCoercion;
                    issue = Issue::error(format!(
                        "A value with a less specific type `{assigned_type_str}` is being assigned to property `${property_name_str}` ({property_type_str})."
                    ))
                    .with_note("The assigned value contains a nested `mixed` type, which can hide potential bugs.");
                } else {
                    issue_kind = TypingIssueKind::PropertyTypeCoercion;
                    issue = Issue::error(format!(
                                "A value of a less specific type `{assigned_type_str}` is being assigned to property `${property_name_str}` ({property_type_str})."
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
                        "This property `{property_name_str}` is declared with type `{property_type_str}`"
                    )));
                }

                context.buffer.report(
                    issue_kind,
                    issue.with_help(
                        "Consider adding a type assertion to narrow the type of the value before the assignment.",
                    ),
                );
            } else {
                if let Some(value_span) = assigned_value_span {
                    issue = Issue::error(format!(
                        "Invalid type for property `${property_name_str}`: expected `{property_type_str}`, but got `{assigned_type_str}`."
                    ))
                    .with_annotation(
                        Annotation::primary(value_span)
                            .with_message(format!("This expression has type `{assigned_type_str}`")),
                    );
                } else {
                    issue = Issue::error(format!(
                        "Invalid assignment to property `${property_name_str}`: cannot assign value of type `{assigned_type_str}` to expected type `{property_type_str}`."
                    ))
                    .with_annotation(
                        Annotation::primary(property_access.span())
                            .with_message("The value assigned to this property is of an incompatible type"),
                    );
                }

                if let Some(property_span) = resolved_property.property_span {
                    issue = issue.with_annotation(Annotation::secondary(property_span).with_message(format!(
                        "This property `${property_name_str}` is declared with type `{property_type_str}`"
                    )));
                }

                context.buffer.report(
                    TypingIssueKind::InvalidPropertyAssignmentValue,
                    issue
                         .with_note(format!("The type `{assigned_type_str}` is not compatible with and cannot be assigned to `{property_type_str}`."))
                         .with_help("Change the assigned value to match the property's type, or update the property's type declaration."),
                );
            }
        }

        if type_match_found || union_comparison_result.type_coerced.unwrap_or(false) {
            for (name, mut bound) in union_comparison_result.type_variable_lower_bounds {
                let name_str = context.interner.lookup(&name);
                if let Some((lower_bounds, _)) = artifacts.type_variable_bounds.get_mut(name_str) {
                    bound.span = Some(property_access.span());
                    lower_bounds.push(bound);
                }
            }

            for (name, mut bound) in union_comparison_result.type_variable_upper_bounds {
                let name_str = context.interner.lookup(&name);

                if let Some((_, upper_bounds)) = artifacts.type_variable_bounds.get_mut(name_str) {
                    bound.span = Some(property_access.span());
                    upper_bounds.push(bound);
                }
            }
        }

        resolved_property_type = Some(add_optional_union_type(
            resolved_property.property_type,
            resolved_property_type.as_ref(),
            context.codebase,
            context.interner,
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
        resulting_type =
            Some(add_optional_union_type(get_mixed_any(), resulting_type.as_ref(), context.codebase, context.interner));
    }

    if resolution_result.has_error_path || resolution_result.has_invalid_path || resolution_result.encountered_null {
        resulting_type =
            Some(add_optional_union_type(get_never(), resulting_type.as_ref(), context.codebase, context.interner));
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

fn add_instance_property_dataflow(
    context: &Context<'_>,
    block_context: &mut BlockContext,
    artifacts: &mut AnalysisArtifacts,
    object_variable_id: &Option<String>,
    object_span: Span,
    property_span: Span,
    class_name: &StringIdentifier,
    property_name: &StringIdentifier,
    assigned_value_type: &TUnion,
) {
    if let Some(class_like_metadata) = get_class_like(context.codebase, context.interner, class_name) {
        if class_like_metadata.specialized_instance {
            if let Some(object_variable_id) = object_variable_id.to_owned() {
                add_instance_property_assignment_dataflow(
                    context,
                    block_context,
                    artifacts,
                    object_variable_id,
                    object_span,
                    property_span,
                    class_name,
                    property_name,
                    assigned_value_type,
                );
            }
        } else {
            add_unspecialized_property_assignment_dataflow(
                context,
                artifacts,
                property_span,
                class_name,
                property_name,
                assigned_value_type,
            );
        }
    }
}

fn add_instance_property_assignment_dataflow(
    context: &Context<'_>,
    block_context: &mut BlockContext,
    artifacts: &mut AnalysisArtifacts,
    object_variable_id: String,
    object_span: Span,
    property_span: Span,
    class_name: &StringIdentifier,
    property_name: &StringIdentifier,
    assigned_value_type: &TUnion,
) {
    let (object_variable_node, property_node) =
        get_nodes_for_property_access(context, &object_variable_id, object_span, *property_name, property_span);

    artifacts.data_flow_graph.add_node(object_variable_node.clone());
    artifacts.data_flow_graph.add_node(property_node.clone());
    artifacts.data_flow_graph.add_path(
        &property_node,
        &object_variable_node,
        PathKind::PropertyAssignment(*class_name, *property_name),
    );

    for parent_node in assigned_value_type.parent_nodes.iter() {
        artifacts.data_flow_graph.add_path(parent_node, &property_node, PathKind::Default);
    }

    let object_type = block_context.locals.get_mut(&object_variable_id);
    if let Some(object_type) = object_type {
        let mut object_type_inner = (**object_type).clone();

        if !object_type_inner.parent_nodes.iter().any(|n| n.id == object_variable_node.id) {
            object_type_inner.parent_nodes.push(object_variable_node.clone());
        }

        *object_type = Rc::new(object_type_inner);
    }
}

pub(crate) fn add_unspecialized_property_assignment_dataflow(
    context: &Context<'_>,
    artifacts: &mut AnalysisArtifacts,
    property_span: Span,
    class_name: &StringIdentifier,
    property_name: &StringIdentifier,
    assigned_value_type: &TUnion,
) {
    let localized_property_node =
        DataFlowNode::get_for_localized_property((*class_name, *property_name), property_span);

    artifacts.data_flow_graph.add_node(localized_property_node.clone());

    let property_node = DataFlowNode::get_for_property((*class_name, *property_name));

    artifacts.data_flow_graph.add_node(property_node.clone());
    artifacts.data_flow_graph.add_path(
        &localized_property_node,
        &property_node,
        PathKind::PropertyAssignment(*class_name, *property_name),
    );

    for parent_node in assigned_value_type.parent_nodes.iter() {
        artifacts.data_flow_graph.add_path(parent_node, &localized_property_node, PathKind::Default);
    }

    let declaring_property_class = context.codebase.get_declaring_class_for_property(class_name, property_name);
    if let Some(declaring_property_class) = declaring_property_class
        && declaring_property_class != class_name
    {
        let declaring_property_node = DataFlowNode::get_for_property((*class_name, *property_name));

        artifacts.data_flow_graph.add_path(
            &property_node,
            &declaring_property_node,
            PathKind::PropertyAssignment(*class_name, *property_name),
        );

        artifacts.data_flow_graph.add_node(declaring_property_node);
    }
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
