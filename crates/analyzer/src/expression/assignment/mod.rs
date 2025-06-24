use std::collections::BTreeMap;
use std::rc::Rc;

use indexmap::IndexMap;

use mago_algebra::clause::Clause;
use mago_algebra::disjoin_clauses;
use mago_codex::assertion::Assertion;
use mago_codex::data_flow::graph::GraphKind;
use mago_codex::data_flow::node::DataFlowNode;
use mago_codex::data_flow::node::DataFlowNodeId;
use mago_codex::data_flow::node::DataFlowNodeKind;
use mago_codex::misc::VariableIdentifier;
use mago_codex::ttype::TType;
use mago_codex::ttype::builder::get_type_from_string;
use mago_codex::ttype::comparator::union_comparator::can_expression_types_be_identical;
use mago_codex::ttype::get_literal_int;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::union::populate_union_type;
use mago_docblock::document::Element;
use mago_docblock::document::TagKind;
use mago_docblock::tag::parse_var_tag;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::artifacts::get_expression_range;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::find_expression_logic_issues;
use crate::formula::get_formula;
use crate::issue::TypingIssueKind;
use crate::utils::expression::array::get_array_target_type_given_index;
use crate::utils::expression::expression_has_logic;
use crate::utils::expression::get_expression_id;
use crate::utils::expression::get_root_expression_id;
use crate::utils::misc::unwrap_expression;

mod array_assignment;
mod property_assignment;
mod static_property_assignment;

impl Analyzable for Assignment {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_assignment(
            context,
            block_context,
            artifacts,
            Some(self.span()),
            &self.lhs,
            Some(&self.operator),
            Some(&self.rhs),
            None,
        )
    }
}

pub fn analyze_assignment<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    assignment_span: Option<Span>,
    target_expression: &Expression,
    mut assignment_operator: Option<&AssignmentOperator>,
    source_expression: Option<&Expression>,
    source_type: Option<&TUnion>,
) -> Result<(), AnalysisError> {
    if let Some(AssignmentOperator::Assign(_)) = assignment_operator {
        assignment_operator = None;
    }

    analyze_assignement_target(target_expression, context, block_context, artifacts)?;

    let target_variable_id = get_expression_id(
        target_expression,
        block_context.scope.get_class_like_name(),
        context.resolved_names,
        context.interner,
        Some(context.codebase),
    );

    let mut existing_target_type = None;
    if let Some(target_variable_id) = &target_variable_id {
        block_context.conditionally_referenced_variable_ids.remove(target_variable_id);
        block_context.assigned_variable_ids.insert(target_variable_id.clone(), target_expression.span().start.offset);
        block_context.possibly_assigned_variable_ids.insert(target_variable_id.clone());

        existing_target_type = block_context.locals.get(target_variable_id).cloned();
    }

    if let Some(source_expression) = source_expression {
        let was_inside_general_use = block_context.inside_general_use;
        block_context.inside_general_use = true;

        match assignment_operator {
            None => {
                source_expression.analyze(context, block_context, artifacts)?;
            }
            // this rewrites $a += 4 and $a ??= 4 to $a = $a + 4 and $a = $a ?? 4 respectively
            Some(assignment_operator) => {
                let previous_expression_types = artifacts.expression_types.clone();
                block_context.inside_assignment_operation = true;

                let binary_expression = Expression::Binary(Binary {
                    lhs: Box::new(target_expression.clone()),
                    operator: match assignment_operator {
                        AssignmentOperator::Addition(span) => BinaryOperator::Addition(*span),
                        AssignmentOperator::Subtraction(span) => BinaryOperator::Subtraction(*span),
                        AssignmentOperator::Multiplication(span) => BinaryOperator::Multiplication(*span),
                        AssignmentOperator::Division(span) => BinaryOperator::Division(*span),
                        AssignmentOperator::Modulo(span) => BinaryOperator::Modulo(*span),
                        AssignmentOperator::Exponentiation(span) => BinaryOperator::Exponentiation(*span),
                        AssignmentOperator::Concat(span) => BinaryOperator::StringConcat(*span),
                        AssignmentOperator::BitwiseAnd(span) => BinaryOperator::BitwiseAnd(*span),
                        AssignmentOperator::BitwiseOr(span) => BinaryOperator::BitwiseOr(*span),
                        AssignmentOperator::BitwiseXor(span) => BinaryOperator::BitwiseXor(*span),
                        AssignmentOperator::LeftShift(span) => BinaryOperator::LeftShift(*span),
                        AssignmentOperator::RightShift(span) => BinaryOperator::RightShift(*span),
                        AssignmentOperator::Coalesce(span) => BinaryOperator::NullCoalesce(*span),
                        AssignmentOperator::Assign(_) => unreachable!(),
                    },
                    rhs: Box::new(source_expression.clone()),
                });

                binary_expression.analyze(context, block_context, artifacts)?;
                block_context.inside_assignment_operation = false;
                let assignment_type = if let Some(assignment_span) = assignment_span {
                    artifacts.get_rc_expression_type(&assignment_span).cloned()
                } else {
                    None
                };

                artifacts.expression_types = previous_expression_types;
                if let Some(expression_type) = assignment_type {
                    artifacts.expression_types.insert(get_expression_range(source_expression), expression_type);
                };
            }
        };

        if expression_has_logic(source_expression) {
            find_expression_logic_issues(source_expression, context, block_context, artifacts);
        }

        block_context.inside_general_use = was_inside_general_use;
    }

    let source_type = if let Some(source_type) = source_type {
        source_type.clone()
    } else if let Some(source_expression) = source_expression {
        if let Some(source_type) = artifacts.get_expression_type(&source_expression) {
            source_type.clone()
        } else {
            get_mixed_any()
        }
    } else {
        get_mixed_any()
    };

    if let (Some(target_variable_id), Some(existing_target_type), None) =
        (&target_variable_id, &existing_target_type, assignment_operator)
    {
        if block_context.inside_loop
            && !block_context.inside_assignment_operation
            && let Some(Expression::Clone(clone_expression)) = source_expression
            && let Expression::Variable(Variable::Direct(cloned_var)) = clone_expression.object.as_ref()
            && context.interner.lookup(&cloned_var.name) == target_variable_id
        {
            let mut origin_node_ids = vec![];

            for parent_node in &existing_target_type.parent_nodes {
                origin_node_ids.extend(artifacts.data_flow_graph.get_origin_node_ids(&parent_node.id, &[], false));
            }

            if origin_node_ids.len() > 1
                && let Some(assignment_span) = assignment_span
            {
                context.buffer.report(
                                        TypingIssueKind::CloneInsideLoop,
                                        Issue::warning(format!(
                                            "Cloning variable `{target_variable_id}` onto itself inside a loop might not have the intended effect."
                                        ))
                                        .with_annotation(
                                            Annotation::primary(assignment_span).with_message("Cloning onto self within loop")
                                        )
                                        .with_note(
                                            "This pattern overwrites the variable with a fresh clone on each loop iteration."
                                        )
                                        .with_note(
                                            "If the intent was to modify a copy of the variable defined *outside* the loop, the clone should happen *before* the loop starts."
                                        )
                                        .with_help(
                                            format!(
                                                "Consider cloning `{target_variable_id}` before the loop if you need a copy, or revise the loop logic if cloning onto itself is not the desired behavior."
                                            )
                                        ),
                                    );
            }
        }

        if block_context.inside_loop
            && !block_context.inside_assignment_operation
            && block_context.for_loop_init_bounds.0 > 0
            && target_variable_id != "$_"
        {
            let mut origin_node_ids = vec![];

            for parent_node in &existing_target_type.parent_nodes {
                origin_node_ids.extend(artifacts.data_flow_graph.get_origin_node_ids(&parent_node.id, &[], false));
            }

            if let Some(Expression::Clone(clone_expression)) = source_expression
                && let Expression::Variable(Variable::Direct(cloned_variable)) = clone_expression.object.as_ref()
                && context.interner.lookup(&cloned_variable.name) == target_variable_id
            {
                // TODO(azjezz): check psalm for this...
            }

            if let Some(assignment_span) = assignment_span {
                origin_node_ids.retain(|id| {
                    if let Some(node) = artifacts.data_flow_graph.get_node(id) {
                        match (&id, &node.kind) {
                            (
                                DataFlowNodeId::ForInit(start_offset, end_offset),
                                DataFlowNodeKind::ForLoopInit { variable: for_loop_var_id, .. },
                            ) => {
                                for_loop_var_id.0 == context.interner.intern(target_variable_id)
                                    && assignment_span.start.offset > *start_offset
                                    && assignment_span.end.offset < *end_offset
                            }
                            _ => false,
                        }
                    } else {
                        false
                    }
                });

                if !origin_node_ids.is_empty() {
                    context.buffer.report(
                    TypingIssueKind::ForLoopInvalidation,
                    Issue::warning(format!(
                        "Assignment to `{target_variable_id}` within the loop body modifies the variable originally initialized in the `for` loop header."
                    ))
                    .with_annotation(
                        Annotation::primary(assignment_span)
                            .with_message("Variable assigned here was initialized in the loop header")
                    )
                    .with_note(
                        "Modifying the loop initialization variable inside the loop can lead to unexpected behavior or infinite loops."
                    )
                    .with_help(
                        format!(
                            "Use a different variable name inside the loop if you don't intend to alter the loop's iteration variable (`{target_variable_id}`)."
                        )
                    ),
                );
                }
            }
        }
    }

    if let (Some(target_variable_id), Some(existing_target_type)) = (&target_variable_id, &existing_target_type) {
        block_context.remove_descendants(
            context.interner,
            context.codebase,
            &mut context.buffer,
            artifacts,
            target_variable_id,
            existing_target_type,
            Some(&source_type),
        );
    } else {
        let root_var_id = get_root_expression_id(target_expression, context.interner);

        if let Some(root_var_id) = root_var_id
            && let Some(existing_root_type) = block_context.locals.get(&root_var_id).cloned()
        {
            block_context.remove_variable_from_conflicting_clauses(
                context.interner,
                context.codebase,
                &mut context.buffer,
                artifacts,
                &root_var_id,
                Some(&existing_root_type),
            );
        }
    }

    let successful = assign_to_expression(
        context,
        block_context,
        artifacts,
        target_expression,
        target_variable_id,
        source_expression,
        source_type.clone(),
        false,
    )?;

    if !successful {
        if matches!(
            target_expression,
            Expression::Identifier(_) | Expression::ConstantAccess(_) | Expression::Access(Access::ClassConstant(_))
        ) {
            context.buffer.report(
                TypingIssueKind::AssignmentToConstant,
                Issue::error("Cannot assign to a constant.")
                    .with_annotation(
                        Annotation::primary(target_expression.span())
                            .with_message("Attempting assignment to constant here."),
                    )
                    .with_note("Constants cannot be reassigned after definition.")
                    .with_help("Assign the value to a variable instead, or remove the assignment."),
            );
        } else {
            context.buffer.report(
                TypingIssueKind::InvalidAssignment,
                Issue::error(
                    "Invalid target for assignment."
                )
                .with_annotation(
                    Annotation::primary(target_expression.span())
                        .with_message("This expression cannot be assigned to.")
                )
                .with_note(
                    "Assignments require a valid variable, array element, or object property on the left-hand side."
                )
                .with_help(
                    "Ensure the left side of the assignment is a valid target (e.g., `$variable`, `$array[key]`, `$object->property`)."
                ),
            );
        }
    }

    if let Some(assignment_span) = assignment_span {
        artifacts.set_expression_type(&assignment_span, source_type);
    }

    Ok(())
}

pub(crate) fn assign_to_expression<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    target_expression: &Expression,
    target_expression_id: Option<String>,
    source_expression: Option<&Expression>,
    source_type: TUnion,
    destructuring: bool,
) -> Result<bool, AnalysisError> {
    match target_expression {
        Expression::Variable(target_variable) if target_expression_id.is_some() => analyze_assignment_to_variable(
            context,
            block_context,
            artifacts,
            matches!(target_variable, Variable::Direct(_)),
            target_variable.span(),
            source_expression,
            source_type,
            // SAFETY: `target_expression_id` is guaranteed to be `Some` here.
            unsafe { target_expression_id.as_ref().unwrap_unchecked() },
            destructuring,
        ),
        Expression::Access(Access::Property(property_access)) => property_assignment::analyze(
            context,
            block_context,
            artifacts,
            property_access,
            &source_type,
            source_expression.map(|e| e.span()),
        )?,
        Expression::Access(Access::StaticProperty(StaticPropertyAccess { class, property, .. })) => {
            static_property_assignment::analyze(
                context,
                block_context,
                artifacts,
                (class, property),
                &source_type,
                &target_expression_id,
            )?
        }
        Expression::ArrayAccess(array_access) => {
            array_assignment::analyze(context, block_context, artifacts, array_access.into(), source_type)?;
        }
        Expression::ArrayAppend(array_append) => {
            array_assignment::analyze(context, block_context, artifacts, array_append.into(), source_type)?;
        }
        Expression::Array(array) => {
            analyze_destructuring(
                context,
                block_context,
                artifacts,
                array.span(),
                source_expression,
                source_type,
                array.elements.as_slice(),
            )?;
        }
        Expression::List(list) => {
            analyze_destructuring(
                context,
                block_context,
                artifacts,
                list.span(),
                source_expression,
                source_type,
                list.elements.as_slice(),
            )?;
        }
        _ => {
            return Ok(false);
        }
    };

    Ok(true)
}

pub fn analyze_assignment_to_variable<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    is_direct_variable: bool,
    variable_span: Span,
    source_expression: Option<&Expression>,
    mut assigned_type: TUnion,
    variable_id: &str,
    destructuring: bool,
) {
    if variable_id.eq("$this") {
        context.buffer.report(
            TypingIssueKind::AssignmentToThis,
            Issue::error("Cannot assign to `$this`.")
                .with_annotation(
                    Annotation::primary(variable_span).with_message("`$this` cannot be used as an assignment target."),
                )
                .with_note("The `$this` variable is read-only and refers to the current object instance.")
                .with_help("Use a different variable name for the assignment."),
        );
    }

    if assigned_type.is_never() {
        let mut issue =
            Issue::error("Invalid assignment: the right-hand side has type `never` and cannot produce a value.")
                .with_annotation(
                    Annotation::primary(variable_span).with_message("Cannot assign a `never` type value here"),
                );

        if let Some(source_expression) = source_expression
            && let Expression::Binary(_) = source_expression
        {
            issue = issue.with_annotation(
                Annotation::secondary(source_expression.span()).with_message("This expression has type `never`."),
            );
        }

        context.buffer.report(
            TypingIssueKind::ImpossibleAssignment,
            issue
                .with_note(
                    "An expression with type `never` is guaranteed to exit, throw, or loop indefinitely."
                )
                .with_help(
                    "This assignment is unreachable because the right-hand side never completes. Remove the assignment or refactor the preceding code."
                )
        );
    }

    if assigned_type.is_mixed() && !variable_id.starts_with("$_") {
        let mut issue = Issue::warning("Assigning `mixed` type to a variable may lead to unexpected behavior.");

        if let Some(source_expression) = source_expression
            && let Expression::Binary(_) = source_expression
        {
            issue = issue.with_annotation(
                Annotation::secondary(source_expression.span()).with_message("This expression has type `mixed`."),
            );
        }

        context.buffer.report(
            TypingIssueKind::MixedAssignment,
            issue.with_annotation(Annotation::primary(variable_span).with_message("Assigning `mixed` type here."))
                .with_note("Using `mixed` can lead to runtime errors if the variable is used in a way that assumes a specific type.")
                .with_help("Consider using a more specific type to avoid potential issues."),
        );
    }

    let mut from_docblock = false;
    if let Some((variable_type, variable_type_span)) =
        get_type_from_var_docblock(context, block_context, artifacts, Some(variable_id), variable_span, !destructuring)
    {
        check_docblock_type_incompatibility(
            context,
            Some(variable_id),
            variable_span,
            &assigned_type,
            &variable_type,
            variable_type_span,
            source_expression,
        );

        assigned_type = variable_type;
        from_docblock = true;
    }

    let has_parent_nodes = !assigned_type.parent_nodes.is_empty();

    let variable_identifier = VariableIdentifier(context.interner.intern(variable_id));
    let assignment_node = if artifacts.data_flow_graph.kind == GraphKind::FunctionBody && is_direct_variable {
        DataFlowNode::get_for_variable_source(
            variable_identifier,
            variable_span,
            false,
            has_parent_nodes,
            block_context.inside_loop
                && !block_context.inside_assignment_operation
                && block_context.for_loop_init_bounds.0 > 0,
        )
    } else {
        DataFlowNode::get_for_lvar(variable_identifier, variable_span)
    };

    artifacts.data_flow_graph.add_node(assignment_node.clone());
    assigned_type.parent_nodes = vec![assignment_node];

    if artifacts.data_flow_graph.kind == GraphKind::FunctionBody
        && !has_parent_nodes
        && !block_context.inside_assignment_operation
        && !variable_id.starts_with("$_")
    {
        let (start_offset, end_offset) = block_context.for_loop_init_bounds;
        if start_offset != 0 {
            let for_node = DataFlowNode {
                id: DataFlowNodeId::ForInit(start_offset, end_offset),
                kind: DataFlowNodeKind::ForLoopInit { variable: variable_identifier },
            };

            artifacts.data_flow_graph.add_node(for_node.clone());
            assigned_type.parent_nodes.push(for_node);
        }
    }

    if !from_docblock
        && assigned_type.is_bool()
        && let Some(source_expression) = source_expression
        && matches!(unwrap_expression(source_expression), Expression::Binary(_))
    {
        handle_assignment_with_boolean_logic(
            context,
            block_context,
            artifacts,
            variable_span,
            source_expression,
            variable_id,
        );
    }

    block_context.locals.insert(variable_id.to_owned(), Rc::new(assigned_type));
}

fn analyze_destructuring<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    target_span: Span,                      // the span of the destructuring target ( list or array )
    source_expression: Option<&Expression>, // the expression being destructured
    array_type: TUnion,                     // the type of the array being destructured
    target_elements: &[ArrayElement],       // the elements being destructured
) -> Result<(), AnalysisError> {
    let mut non_array = false;

    if !array_type.is_array() {
        let assigned_type_str = array_type.get_id(Some(context.interner));

        let mut issue = Issue::error(format!(
            "Invalid destructuring assignment: Cannot unpack type `{assigned_type_str}` into variables.",
        ));

        if let Some(source_expression) = source_expression {
            issue = issue
                .with_annotation(
                    Annotation::primary(source_expression.span())
                        .with_message(format!("This expression has type `{assigned_type_str}`...")),
                )
                .with_annotation(
                    Annotation::secondary(target_span)
                        .with_message("...but this destructuring pattern requires an array."),
                );
        } else {
            issue = issue.with_annotation(Annotation::primary(target_span).with_message(format!(
                "Attempting to destructure a value of type `{assigned_type_str}`, which is not an array."
            )));
        }

        issue = issue
            .with_note("Array destructuring (`[...] = $value;`) requires `$value` to be an array or an object that implements `ArrayAccess`.")
            .with_note(format!(
                "Attempting to destructure a non-array type like `{assigned_type_str}` is an undefined behavior in PHP.",
            ))
            .with_help(
                "Ensure the value on the right-hand side is an array before attempting to destructure it.",
            );

        context.buffer.report(TypingIssueKind::InvalidArrayDestructuring, issue);

        non_array = true;
    }

    let mut last_index: usize = 0;
    let mut impossible = non_array;

    let has_keyed_elements = target_elements.iter().any(|e| matches!(e, ArrayElement::KeyValue(_)));
    let has_non_keyed_elements = target_elements.iter().any(|e| matches!(e, ArrayElement::Value(_)));
    let has_skipped_elements = target_elements.iter().any(|e| matches!(e, ArrayElement::Missing(_)));

    if has_keyed_elements {
        if has_non_keyed_elements {
            let first_keyed_span =
                // SAFETY: we know that there is at least one keyed element, so this is safe.
                unsafe { target_elements.iter().find(|e| matches!(e, ArrayElement::KeyValue(_))).unwrap_unchecked().span() };

            let first_non_keyed_span =
                // SAFETY: we know that there is at least one non-keyed element, so this is safe.
                unsafe { target_elements.iter().find(|e| matches!(e, ArrayElement::Value(_))).unwrap_unchecked().span() };

            let mut issue = Issue::error("Cannot mix keyed and non-keyed elements in array destructuring.")
                .with_annotation(Annotation::primary(target_span).with_message("This destructuring mixes both styles"))
                .with_note("PHP requires destructuring assignments to use either all list-style elements or all keyed elements, but not both.")
                .with_help("Separate the destructuring into two operations or choose one style.")
            ;

            if first_keyed_span.start.offset < first_non_keyed_span.start.offset {
                issue = issue
                    .with_annotation(Annotation::secondary(first_keyed_span).with_message("This is a keyed element..."))
                    .with_annotation(
                        Annotation::secondary(first_non_keyed_span)
                            .with_message("...and this is a non-keyed (list-style) element"),
                    );
            } else {
                issue = issue
                    .with_annotation(
                        Annotation::secondary(first_non_keyed_span)
                            .with_message("This is a non-keyed (list-style) element..."),
                    )
                    .with_annotation(
                        Annotation::secondary(first_keyed_span).with_message("...and this is a keyed element"),
                    );
            }

            context.buffer.report(TypingIssueKind::MixedKeyedAndNonKeyedArrayDestructuring, issue);

            impossible = true;
        }

        if has_skipped_elements {
            let first_skipped_span =
                // SAFETY: we know that there is at least one skipped element, so this is safe.
                unsafe { target_elements.iter().find(|e| matches!(e, ArrayElement::Missing(_))).unwrap_unchecked().span() };
            let first_keyed_span =
                // SAFETY: we know that there is at least one keyed element, so this is safe.
                unsafe { target_elements.iter().find(|e| matches!(e, ArrayElement::KeyValue(_))).unwrap_unchecked().span() };

            let mut issue = Issue::error("Cannot use skipped elements (`,,`) in a keyed array destructuring.")
                .with_annotation(Annotation::primary(target_span).with_message("This destructuring is invalid"))
                .with_help(
                    "To get specific keys, access them directly. Do not mix keyed access with list-style skipping.",
                );

            if first_keyed_span.start.offset < first_skipped_span.start.offset {
                issue = issue
                    .with_annotation(Annotation::secondary(first_keyed_span).with_message("This is a keyed element..."))
                    .with_annotation(
                        Annotation::secondary(first_skipped_span)
                            .with_message("...but skipping elements is only allowed in list-style destructuring"),
                    );
            } else {
                issue = issue
                    .with_annotation(Annotation::primary(target_span).with_message("This destructuring is invalid"))
                    .with_annotation(
                        Annotation::secondary(first_skipped_span).with_message("This is a skipped element..."),
                    );
            }

            context.buffer.report(TypingIssueKind::MixedKeyedAndSkippedArrayDestructuring, issue);

            impossible = true;
        }
    }

    for target_element in target_elements {
        match target_element {
            ArrayElement::KeyValue(key_value_element) => {
                key_value_element.key.analyze(context, block_context, artifacts)?;

                let index_type = artifacts
                    .get_expression_type(key_value_element.key.as_ref())
                    .cloned()
                    .unwrap_or_else(get_mixed_any);

                let access_type = if impossible {
                    get_never()
                } else {
                    get_array_target_type_given_index(
                        context,
                        block_context,
                        artifacts,
                        key_value_element.key.span(),
                        if let Some(source_expression) = source_expression {
                            source_expression.span()
                        } else {
                            target_span
                        },
                        None,
                        &array_type,
                        &index_type,
                        false,
                        &None,
                    )
                };

                analyze_assignment(
                    context,
                    block_context,
                    artifacts,
                    None,
                    &key_value_element.value,
                    None,
                    Some(&key_value_element.key),
                    Some(&access_type),
                )?;
            }
            ArrayElement::Value(value_element) => {
                let index_type = get_literal_int(last_index as i64);

                let access_type = if impossible {
                    get_never()
                } else {
                    get_array_target_type_given_index(
                        context,
                        block_context,
                        artifacts,
                        target_span,
                        if let Some(source_expression) = source_expression {
                            source_expression.span()
                        } else {
                            target_span
                        },
                        Some(value_element.value.span()),
                        &array_type,
                        &index_type,
                        false,
                        &None,
                    )
                };

                analyze_assignment(
                    context,
                    block_context,
                    artifacts,
                    None,
                    &value_element.value,
                    None,
                    None,
                    Some(&access_type),
                )?;
            }
            ArrayElement::Variadic(variadic_element) => {
                context.buffer.report(
                    TypingIssueKind::InvalidVariadicInDestructuring,
                    Issue::error("Variadic unpacking (`...`) is not permitted in a destructuring assignment.")
                        .with_annotation(Annotation::primary(variadic_element.span()).with_message("This syntax is not allowed here"))
                        .with_note("The `...` operator can be used for argument unpacking in function calls or for spreading elements into a new array on the right-hand side of an expression, but not on the left-hand side of an assignment.")
                        .with_help("Remove the `...` operator. If you intend to capture remaining array elements, this must be done in a separate step."),
                );

                analyze_assignment(
                    context,
                    block_context,
                    artifacts,
                    None,
                    &variadic_element.value,
                    None,
                    None,
                    Some(&get_never()),
                )?;

                continue;
            }
            ArrayElement::Missing(_) => {}
        }

        last_index += 1;
    }

    Ok(())
}

fn analyze_assignement_target<'a>(
    expression: &Expression,
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    match expression {
        Expression::Variable(Variable::Nested(nested)) => {
            nested.variable.analyze(context, block_context, artifacts)?;
        }
        Expression::Variable(Variable::Indirect(indirect)) => {
            indirect.expression.analyze(context, block_context, artifacts)?;
        }
        Expression::List(List { elements, .. }) | Expression::Array(Array { elements, .. }) => {
            for element in elements.iter() {
                match element {
                    ArrayElement::KeyValue(key_value_array_element) => {
                        analyze_assignement_target(&key_value_array_element.value, context, block_context, artifacts)?;
                    }
                    ArrayElement::Value(value_array_element) => {
                        analyze_assignement_target(&value_array_element.value, context, block_context, artifacts)?;
                    }
                    ArrayElement::Variadic(variadic_array_element) => {
                        analyze_assignement_target(&variadic_array_element.value, context, block_context, artifacts)?;
                    }
                    ArrayElement::Missing(_) => {}
                }
            }
        }
        Expression::ArrayAccess(array_access) => {
            analyze_assignement_target(&array_access.array, context, block_context, artifacts)?;
            analyze_assignement_target(&array_access.index, context, block_context, artifacts)?;
        }
        Expression::Access(Access::Property(property_access)) => {
            analyze_assignement_target(&property_access.object, context, block_context, artifacts)?;
        }
        Expression::Access(Access::NullSafeProperty(null_safe_property_access)) => {
            analyze_assignement_target(&null_safe_property_access.object, context, block_context, artifacts)?;
        }
        Expression::Access(Access::StaticProperty(static_property_access)) => {
            analyze_assignement_target(&static_property_access.class, context, block_context, artifacts)?;
        }
        _ => {}
    }

    Ok(())
}

fn handle_assignment_with_boolean_logic(
    context: &mut Context<'_>,
    block_context: &mut BlockContext,
    artifacts: &mut AnalysisArtifacts,
    variable_expression_id: Span,
    source_expression: &Expression,
    variable_id: &str,
) {
    let right_clauses = get_formula(
        source_expression.span(),
        source_expression.span(),
        source_expression,
        context.get_assertion_context_from_block(block_context),
        artifacts,
    );

    let right_clauses = BlockContext::filter_clauses(
        context.interner,
        context.codebase,
        &mut context.buffer,
        artifacts,
        variable_id,
        right_clauses.into_iter().map(Rc::new).collect(),
        None,
    );

    let mut possibilities = BTreeMap::new();
    possibilities.insert(variable_id.to_owned(), IndexMap::from([(Assertion::Falsy.to_hash(), Assertion::Falsy)]));

    block_context.clauses.extend(
        disjoin_clauses(
            vec![Clause::new(possibilities, variable_expression_id, variable_expression_id, None, None, None)],
            right_clauses.into_iter().map(|v| (*v).clone()).collect(),
            source_expression.span(),
        )
        .into_iter()
        .map(Rc::new),
    );
}

/// Finds the last applicable `@var` tag for a given variable and parses its type string.
///
/// This function retrieves the docblock associated with the current statement from the
/// context. It then iterates through all `@var`, `@psalm-var`, and `@phpstan-var` tags
/// to find the last one that applies to the specified `variable_id`. If a matching
/// tag is found, it attempts to parse the type string into a `TUnion`.
///
/// If parsing fails, a detailed error is reported to the user.
///
/// # Arguments
///
/// * `context`: The main analysis context, providing access to the docblock parser and error buffer.
/// * `variable_id`: The name of the variable (e.g., "$foo") for which to find a type hint.
/// * `variable_span`: The span of the variable's usage, used for error reporting context.
///
/// # Returns
///
/// An `Option<TUnion>` containing the parsed type if a valid, matching `@var` tag
/// was found and successfully parsed. Returns `None` otherwise.
pub(crate) fn get_type_from_var_docblock<'a>(
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
    value_expression_variable_id: Option<&str>,
    value_expression_span: Span,
    mut allow_unnamed: bool,
) -> Option<(TUnion, Span)> {
    allow_unnamed = allow_unnamed && !block_context.inside_return && !block_context.inside_loop_expressions;

    context
        // Get the parsed docblock of the current statement
        .get_parsed_docblock()
        // Extract the elements from the docblock
        .map(|document| document.elements)
        // If no docblock is present, use an empty vector
        .unwrap_or_default()
        // Iterate over the elements
        .into_iter()
        // Filter out non-tag elements
        .filter_map(|element| match element {
            Element::Tag(tag) => Some(tag),
            _ => None,
        })
        // Parse `@var` tags
        .filter_map(|tag| {
            if !matches!(tag.kind, TagKind::Var | TagKind::PsalmVar | TagKind::PhpstanVar) {
                return None;
            }

            let tag_content = context.interner.lookup(&tag.description);

            parse_var_tag(tag_content, tag.description_span)
        })
        // Filter out tags that do not match the variable name
        .filter_map(|var_tag| match var_tag.variable_name {
            None if allow_unnamed => Some(var_tag.type_string),
            Some(name_in_tag) if Some(name_in_tag.as_str()) == value_expression_variable_id => {
                Some(var_tag.type_string)
            }
            _ => None,
        })
        // Get the last matching type string, as it's the most specific/recent declaration.
        .next_back()
        // Convert the type string to a TUnion
        .and_then(|type_string| {
            get_type_from_string(
                &type_string.value,
                type_string.span,
                &context.scope,
                &context.type_resolution_context,
                block_context.scope.get_class_like_name(),
                context.interner,
            )
            .map(|variable_type| (variable_type, type_string.span))
            .map_err(|type_error| {
                let error_span = type_error.span();

                if let Some(value_expression_variable_id) = value_expression_variable_id {
                    context.buffer.report(
                        TypingIssueKind::InvalidDocblock,
                        Issue::error(format!(
                            "Invalid type in `@var` tag for variable `{value_expression_variable_id}`."
                        ))
                        .with_annotation(Annotation::primary(error_span).with_message(type_error.to_string()))
                        .with_annotation(
                            Annotation::secondary(value_expression_span)
                                .with_message("This variable's type is defined by the docblock"),
                        )
                        .with_note(type_error.note())
                        .with_help(type_error.help()),
                    )
                } else {
                    context.buffer.report(
                        TypingIssueKind::InvalidDocblock,
                        Issue::error("Invalid type in `@var` tag for expression.".to_string())
                            .with_annotation(Annotation::primary(error_span).with_message(type_error.to_string()))
                            .with_annotation(
                                Annotation::secondary(value_expression_span)
                                    .with_message("This expression's type is defined by the docblock"),
                            )
                            .with_note(type_error.note())
                            .with_help(type_error.help()),
                    )
                }
            })
            .ok()
        })
        .map(|(mut variable_type, variable_type_span)| {
            populate_union_type(
                &mut variable_type,
                &context.codebase.symbols,
                context.interner,
                block_context.scope.get_reference_source(&context.source.identifier).as_ref(),
                &mut artifacts.symbol_references,
                true,
            );

            (variable_type, variable_type_span)
        })
}

pub(crate) fn check_docblock_type_incompatibility<'a>(
    context: &mut Context<'a>,
    value_expression_variable_id: Option<&str>,
    value_expression_span: Span,
    inferred_type: &TUnion,
    docblock_type: &TUnion,
    dockblock_type_span: Span,
    source_expression: Option<&Expression>,
) {
    if !can_expression_types_be_identical(context.codebase, context.interner, inferred_type, docblock_type, false) {
        // Get clean string representations of the types for the error message.
        let docblock_type_str = docblock_type.get_id(Some(context.interner));
        let inferred_type_str = inferred_type.get_id(Some(context.interner));

        let mut issue = if let Some(value_expression_variable_id) = value_expression_variable_id {
            Issue::error(format!("Docblock type mismatch for variable `{value_expression_variable_id}`."))
                .with_annotation(
                    Annotation::primary(dockblock_type_span)
                        .with_message(format!("This docblock asserts the type should be `{docblock_type_str}`...")),
                )
        } else {
            Issue::error("Docblock type mismatch for expression.".to_string()).with_annotation(
                Annotation::primary(dockblock_type_span)
                    .with_message(format!("This docblock asserts the type should be `{docblock_type_str}`...")),
            )
        };

        if let Some(value_expression_variable_id) = value_expression_variable_id {
            if let Some(source_expression) = source_expression {
                issue = issue.with_annotation(Annotation::secondary(source_expression.span()).with_message(format!(
                    "...but this expression provides an incompatible type `{inferred_type_str}`."
                )));
            }

            issue = issue.with_annotation(
                Annotation::secondary(value_expression_span)
                    .with_message(format!("The assignment to `{value_expression_variable_id}` here is invalid.")),
            ) .with_note(
                "The type of the assigned value and the `@var` docblock type have no overlap, making this assignment impossible."
            )
            .with_help(format!(
                "Change the assigned value to match `{docblock_type_str}`, or update the `@var` tag to a compatible type."
            ));
        } else {
            issue = issue.with_annotation(
                Annotation::secondary(value_expression_span)
                    .with_message(format!("...but this expression provides an incompatible type `{inferred_type_str}`.")),
            )
            .with_note(
                "The type resolved from the docblock and the type of the expression have no overlap, making the docblock type invalid.",
            )
            .with_help(format!(
                "Change the expression to match `{docblock_type_str}`, or update the `@var` tag to a compatible type."
            ));
        }

        context.buffer.report(TypingIssueKind::DocblockTypeMismatch, issue);
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::issue::TypingIssueKind;
    use crate::test_analysis;

    test_analysis! {
        name = test_var_docblock,
        code = indoc! {r#"
            <?php

            namespace Example;

            /**
             * @template T
             */
            class Suspension
            {
                /**
                 * @template S
                 *
                 * @return Suspension<S>
                 */
                public static function create(): Suspension
                {
                    return new self();
                }

                /**
                 * @param T $_value
                 */
                public function resume(mixed $_value): void
                {
                    exit(0);
                }

                /**
                 * @return T
                 *
                 * @psalm-suppress InvalidReturnType
                 */
                public function suspend(): mixed
                {
                    exit(0);
                }
            }

            /** @var Suspension<string> */
            $suspension = Suspension::create();
            $suspension->resume('Hello, World!');
            $value = $suspension->suspend();

            echo $value;
        "#}
    }

    test_analysis! {
        name = test_var_docblock_override_narrow,
        code = indoc! {r#"
            <?php

            namespace Example;

            /**
             * @return scalar
             */
            function get_scalar() {
                return 'Hello, World!';
            }

            /** @var string */
            $scalar = get_scalar();
        "#},
    }

    test_analysis! {
        name = test_var_docblock_override_widen,
        code = indoc! {r#"
            <?php

            /**
             * @return list<int>
             */
            function get_list(): array {
                return [1, 2, 3];
            }

            /** @var list<int|string> */
            $scalar = get_list();
        "#},
    }

    test_analysis! {
        name = test_var_docblock_overridei,
        code = indoc! {r#"
            <?php

            /**
             * @return list<int>
             */
            function get_list(): array {
                return [1, 2, 3];
            }

            /** @var bool */
            $scalar = get_list();
        "#},
        issues = [
            TypingIssueKind::DocblockTypeMismatch,
        ]
    }

    test_analysis! {
        name = list_assignment,
        code = indoc! {r#"
            <?php

            /**
             * @return array{a: int, b: int}
             */
            function get_a_and_b(): array {
                return ['a' => 1, 'b' => 2];
            }

            /**
             * @return array{1, 2}
             */
            function get_tuple(): array {
                return [1, 2];
            }

            function list_assignment(): void {
                list($_a, $_b) = get_tuple();
                list('a' => $_a, 'b' => $_b) = get_a_and_b();
            }

            function array_assignment(): void {
                [$_a, $_b] = get_tuple();
                ['a' => $_a, 'b' => $_b] = get_a_and_b();
            }
        "#}
    }

    test_analysis! {
        name = destructuring_shape,
        code = indoc! {r#"
            <?php

            /**
             * @return array{name: string, age: int, hobbies: list<string>}
             */
            function get_shape(): array
            {
                return [
                    'name' => 'John Doe',
                    'age' => 30,
                    'hobbies' => ['reading', 'gaming', 'hiking'],
                ];
            }

            /**
             * @param string $_string
             */
            function i_take_string(string $_string): void {}

            /**
             * @param int $_int
             */
            function i_take_int(int $_int): void {}

            /**
             * @param list<string> $_list
             */
            function i_take_list_of_strings(array $_list): void {}

            ['name' => $name, 'age' => $age, 'hobbies' => $hobbies] = get_shape();

            i_take_string($name); // OK
            i_take_int($age); // OK
            i_take_list_of_strings($hobbies); // OK
        "#},
    }

    test_analysis! {
        name = destructuring_keyed_shape_to_variables,
        code = indoc! {r#"
            <?php
            /** @return array{name: string, age: int, hobbies: list<string>} */
            function get_user_shape(): array {
                return ['name' => 'John', 'age' => 30, 'hobbies' => ['coding']];
            }
            /** @param string $_s */
            function i_take_string(string $_s): void {}
            /** @param int $_i */
            function i_take_int(int $_i): void {}
            /** @param list<string> $_l */
            function i_take_list_of_strings(array $_l): void {}

            ['name' => $name, 'age' => $age, 'hobbies' => $hobbies] = get_user_shape();

            i_take_string($name);
            i_take_int($age);
            i_take_list_of_strings($hobbies);
        "#},
        issues = []
    }

    test_analysis! {
        name = destructuring_list_to_variables,
        code = indoc! {r#"
            <?php
            /** @return list<string> */
            function get_simple_list(): array { return ['a', 'b', 'c']; }
            /** @param string $_s */
            function i_take_string(string $_s): void {}

            [$a, $b, $c] = get_simple_list();
            i_take_string($a);
            i_take_string($b);
            i_take_string($c);
        "#},
        issues = []
    }

    test_analysis! {
        name = destructuring_list_with_skipped_elements,
        code = indoc! {r#"
            <?php
            /** @param 'one' $_s */
            function i_take_one(string $_s): void {}
            /** @param 'three' $_s */
            function i_take_three(string $_s): void {}

            [$first, , $third] = ['one', 'two', 'three'];
            i_take_one($first);

            i_take_three($third);
        "#},
        issues = []
    }

    test_analysis! {
        name = destructuring_list_with_trailing_comma_skip,
        code = indoc! {r#"
            <?php
            /** @param 10 $_i */
            function i_take_ten(int $_i): void {}
            [$x, , ] = [10, 20, 30];
            i_take_ten($x);
        "#},
        issues = []
    }

    test_analysis! {
        name = destructuring_nested_list_within_keyed,
        code = indoc! {r#"
            <?php
            /** @return array{name: string, data: list<int>} */
            function get_shape_with_list(): array { return ['name' => 'test', 'data' => [10, 20]]; }
            /** @param string $_s */
            function i_take_string(string $_s): void {}
            /** @param int $_i */
            function i_take_int(int $_i): void {}

            ['name' => $name, 'data' => [$val1, $val2]] = get_shape_with_list();
            i_take_string($name);
            i_take_int($val1);
            i_take_int($val2);
        "#},
        issues = []
    }

    test_analysis! {
        name = destructuring_nested_keyed_within_list,
        code = indoc! {r#"
            <?php
            /** @return list<array{id: int}> */
            function get_list_of_shapes(): array { return [['id' => 1], ['id' => 2]]; }
            /** @param int $_i */
            function i_take_int(int $_i): void {}

            [['id' => $firstId], ['id' => $secondId]] = get_list_of_shapes();
            i_take_int($firstId);
            i_take_int($secondId);
        "#},
        issues = []
    }

    test_analysis! {
        name = destructuring_empty_array_results_in_null,
        code = indoc! {r#"
            <?php
            /** @param null $_n */
            function i_take_null($_n): void {}
            [$d, $e] = [];
            i_take_null($d);
            i_take_null($e);
        "#},
        issues = [
            TypingIssueKind::MismatchedArrayIndex,
            TypingIssueKind::MismatchedArrayIndex,
        ]
    }

    test_analysis! {
        name = destructuring_missing_keyed_element_results_in_null,
        code = indoc! {r#"
            <?php
            /** @param null $_n */
            function i_take_null($_n): void {}
            /** @return array{name: string} */
            function get_partial_shape(): array { return ['name' => 'test']; }

            ['name' => $name, 'age' => $age] = get_partial_shape();
            i_take_null($age);
        "#},
        issues = [
            TypingIssueKind::UndefinedStringArrayIndex,
        ]
    }

    test_analysis! {
        name = destructuring_list_with_fewer_elements_results_in_null,
        code = indoc! {r#"
            <?php
            /** @param null $_n */
            function i_take_null($_n): void {}
            /** @param int $_i */
            function i_take_int(int $_i): void {}

            [$a, $b] = [1];
            i_take_int($a);
            i_take_null($b);
        "#},
        issues = [
            TypingIssueKind::UndefinedIntArrayIndex,
        ]
    }

    test_analysis! {
        name = destructuring_list_syntax_basic,
        code = indoc! {r#"
            <?php
            /** @param string $_s */
            function i_take_string(string $_s): void {}
            list($a, $b) = ['A', 'B'];
            i_take_string($a);
            i_take_string($b);
        "#},
        issues = []
    }

    test_analysis! {
        name = destructuring_list_syntax_with_skipped_elements,
        code = indoc! {r#"
            <?php
            /** @param string $_s */
            function i_take_string(string $_s): void {}
            list($a, , $c) = ['A', 'B', 'C'];
            i_take_string($a);
            i_take_string($c);
        "#},
        issues = []
    }

    test_analysis! {
        name = destructuring_list_syntax_with_keyed_source,
        code = indoc! {r#"
            <?php
            /** @param string $_s */
            function i_take_string(string $_s): void {}
            $source = [0 => 'a', 1 => 'b', 'key' => 'c'];
            list($a, $b) = $source;
            i_take_string($a);
            i_take_string($b);
        "#},
        issues = []
    }

    test_analysis! {
        name = destructuring_keyed_with_integer_keys,
        code = indoc! {r#"
            <?php
            /** @param string $_s */
            function i_take_string(string $_s): void {}
            $source = [1 => 'one', 2 => 'two'];
            [1 => $val1, 2 => $val2] = $source;
            i_take_string($val1);
            i_take_string($val2);
        "#},
        issues = []
    }

    test_analysis! {
        name = destructuring_empty_target_is_valid,
        code = indoc! {r#"
            <?php
            [] = [1, 2, 3]; // This is valid syntax, should produce no errors.
        "#},
        issues = []
    }

    test_analysis! {
        name = destructuring_keyed_from_list_source,
        code = indoc! {r#"
            <?php
            /** @return list<string> */
            function get_list(): array { return ["a", "b"]; }
            /** @param string $_s */
            function i_take_string(string $_s): void {}
            [0 => $valA, 1 => $valB] = get_list();
            i_take_string($valA);
            i_take_string($valB);
        "#},
        issues = []
    }

    test_analysis! {
        name = destructuring_list_from_typed_array,
        code = indoc! {r#"
            <?php
            /** @param array<int, float> $source */
            function test_typed_source(array $source): void {
                [$a, $b] = $source;
                /** @param float $_f */
                function i_take_float(float $_f): void {}
                i_take_float($a);
                i_take_float($b);
            }
        "#},
        issues = [
            TypingIssueKind::PossiblyUndefinedArrayIndex,
            TypingIssueKind::PossiblyUndefinedArrayIndex,
        ]
    }
}
