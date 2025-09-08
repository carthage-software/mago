use std::rc::Rc;

use indexmap::IndexMap;

use mago_algebra::clause::Clause;
use mago_algebra::disjoin_clauses;
use mago_codex::assertion::Assertion;
use mago_codex::identifier::function_like::FunctionLikeIdentifier;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::callable::TCallable;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::union_comparator;
use mago_codex::ttype::get_literal_int;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_never;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::artifacts::get_expression_range;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::context::block::ReferenceConstraintSource;
use crate::error::AnalysisError;
use crate::expression::find_expression_logic_issues;
use crate::formula::get_formula;
use crate::utils::docblock::check_docblock_type_incompatibility;
use crate::utils::docblock::get_type_from_var_docblock;
use crate::utils::expression::array::get_array_target_type_given_index;
use crate::utils::expression::expression_has_logic;
use crate::utils::expression::get_expression_id;
use crate::utils::expression::get_root_expression_id;
use crate::utils::misc::unwrap_expression;

mod array_assignment;
mod property_assignment;
mod static_property_assignment;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for Assignment<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        analyze_assignment(
            context,
            block_context,
            artifacts,
            Some(self.span()),
            self.lhs,
            Some(&self.operator),
            Some(self.rhs),
            None,
        )
    }
}

pub fn analyze_assignment<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    assignment_span: Option<Span>,
    target_expression: &'ast Expression<'arena>,
    mut assignment_operator: Option<&AssignmentOperator>,
    source_expression: Option<&'ast Expression<'arena>>,
    source_type: Option<&TUnion>,
) -> Result<(), AnalysisError> {
    if let Some(AssignmentOperator::Assign(_)) = assignment_operator {
        assignment_operator = None;
    }

    analyze_assignment_target(target_expression, context, block_context, artifacts)?;

    let target_variable_id = get_expression_id(
        target_expression,
        block_context.scope.get_class_like_name(),
        context.resolved_names,
        Some(context.codebase),
    );

    let mut existing_target_type = None;
    if let Some(target_variable_id) = &target_variable_id {
        block_context.conditionally_referenced_variable_ids.remove(target_variable_id);
        block_context.assigned_variable_ids.insert(target_variable_id.clone(), target_expression.span().start.offset);
        block_context.possibly_assigned_variable_ids.insert(target_variable_id.clone());

        existing_target_type = block_context.locals.get(target_variable_id).cloned();
    }

    if let (Some(source_expression), Some(target_variable_id)) = (source_expression, &target_variable_id)
        && matches!(target_expression, Expression::Variable(_))
        && is_closure_expression(source_expression)
        && let Some(preliminary_type) = get_closure_expression_type(source_expression)
    {
        block_context.locals.insert(target_variable_id.clone(), Rc::new(preliminary_type));
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
                    lhs: context.arena.alloc(target_expression.clone()),
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
                    rhs: context.arena.alloc(source_expression.clone()),
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
            get_mixed()
        }
    } else {
        get_mixed()
    };

    if let (Some(target_variable_id), None) = (&target_variable_id, assignment_operator)
        && block_context.inside_loop
        && !block_context.inside_assignment_operation
        && let Some(Expression::Clone(clone_expression)) = source_expression
        && let Expression::Variable(Variable::Direct(cloned_var)) = clone_expression.object
        && cloned_var.name == target_variable_id
        && let Some(assignment_span) = assignment_span
    {
        context.collector.report_with_code(
            IssueCode::CloneInsideLoop,
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

    if let (Some(target_variable_id), Some(existing_target_type)) = (&target_variable_id, &existing_target_type) {
        block_context.remove_descendants(context, target_variable_id, existing_target_type, Some(&source_type));
    } else {
        let root_var_id = get_root_expression_id(target_expression);

        if let Some(root_var_id) = root_var_id
            && let Some(existing_root_type) = block_context.locals.get(&root_var_id).cloned()
        {
            block_context.remove_variable_from_conflicting_clauses(context, &root_var_id, Some(&existing_root_type));
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
            context.collector.report_with_code(
                IssueCode::AssignmentToConstant,
                Issue::error("Cannot assign to a constant.")
                    .with_annotation(
                        Annotation::primary(target_expression.span())
                            .with_message("Attempting assignment to constant here."),
                    )
                    .with_note("Constants cannot be reassigned after definition.")
                    .with_help("Assign the value to a variable instead, or remove the assignment."),
            );
        } else {
            context.collector.report_with_code(
                IssueCode::InvalidAssignment,
                Issue::error("Invalid target for assignment.")
                    .with_annotation(
                        Annotation::primary(target_expression.span())
                            .with_message("This expression cannot be assigned to."),
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

pub(crate) fn assign_to_expression<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    target_expression: &'ast Expression<'arena>,
    target_expression_id: Option<String>,
    source_expression: Option<&'ast Expression<'arena>>,
    mut source_type: TUnion,
    destructuring: bool,
) -> Result<bool, AnalysisError> {
    if let Some(source_expression) = source_expression {
        source_type.by_reference = source_expression.is_reference();

        analyze_reference_assignment(context, block_context, target_expression, source_expression)?;
    }

    match target_expression {
        Expression::Variable(target_variable) if target_expression_id.is_some() => analyze_assignment_to_variable(
            context,
            block_context,
            artifacts,
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
        Expression::Access(Access::StaticProperty(property_access)) => static_property_assignment::analyze(
            context,
            block_context,
            artifacts,
            property_access,
            &source_type,
            &target_expression_id,
        )?,
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

fn analyze_reference_assignment<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    target_expression: &'ast Expression<'arena>,
    source_expression: &'ast Expression<'arena>,
) -> Result<(), AnalysisError> {
    let Expression::UnaryPrefix(UnaryPrefix {
        operator: UnaryPrefixOperator::Reference(_),
        operand: referenced_expression,
    }) = source_expression
    else {
        return Ok(());
    };

    let target_variable_id = get_expression_id(
        target_expression,
        block_context.scope.get_class_like_name(),
        context.resolved_names,
        Some(context.codebase),
    );

    let referenced_variable_id = get_expression_id(
        referenced_expression,
        block_context.scope.get_class_like_name(),
        context.resolved_names,
        Some(context.codebase),
    );

    let (Some(target_variable_id), Some(referenced_variable_id)) = (target_variable_id, referenced_variable_id) else {
        return Ok(());
    };

    if !block_context.locals.contains_key(&referenced_variable_id) {
        block_context.locals.insert(referenced_variable_id.clone(), Rc::new(get_mixed()));
    }

    if block_context.references_in_scope.contains_key(&target_variable_id) {
        block_context.decrement_reference_count(&target_variable_id);
    }

    // When assigning an existing reference as a reference it removes the
    // old reference, so it's no longer potentially from a confusing scope.
    block_context.references_possibly_from_confusing_scope.remove(&target_variable_id);
    block_context.add_conditionally_referenced_variable(&target_variable_id);
    block_context.references_in_scope.insert(target_variable_id.clone(), referenced_variable_id.clone());
    block_context.referenced_counts.entry(referenced_variable_id.clone()).and_modify(|count| *count += 1).or_insert(1);

    if referenced_variable_id.contains('[') || referenced_variable_id.contains("->") {
        block_context.references_to_external_scope.insert(target_variable_id.clone());
    }

    Ok(())
}

pub fn analyze_assignment_to_variable<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    variable_span: Span,
    source_expression: Option<&Expression<'arena>>,
    mut assigned_type: TUnion,
    variable_id: &str,
    destructuring: bool,
) {
    if let Some(constraint) = block_context.by_reference_constraints.get(variable_id) {
        if let Some(constraint_type) = constraint.constraint_type.as_ref()
            && !union_comparator::is_contained_by(
                context.codebase,
                &assigned_type,
                constraint_type,
                assigned_type.ignore_nullable_issues,
                assigned_type.ignore_falsable_issues,
                false,
                &mut ComparisonResult::default(),
            )
        {
            let assigned_type_str = assigned_type.get_id();
            let constraint_type_str = constraint_type.get_id();
            let primary_error_span = source_expression.map_or(variable_span, |expr| expr.span());

            let issue = match constraint.source {
                ReferenceConstraintSource::Parameter => {
                    Issue::error(format!(
                        "Invalid assignment to by-reference parameter `{variable_id}`.",
                    ))
                    .with_annotation(Annotation::primary(primary_error_span).with_message(format!(
                        "This value has type `{assigned_type_str}`, but the parameter expects `{constraint_type_str}`.",
                    )))
                    .with_annotation(Annotation::secondary(constraint.constraint_span).with_message(
                        "Parameter is defined with a by-reference type constraint here.",
                    ))
                    .with_note(
                        "Assigning an incompatible type to a by-reference parameter can cause unexpected `TypeError`s in the calling scope.",
                    )
                    .with_help(
                        "If the parameter should have a different type on exit, declare it using a `@param-out` docblock tag.",
                    )
                },
                ReferenceConstraintSource::Argument => {
                    Issue::error(format!(
                        "Potentially invalid assignment to referenced variable `{variable_id}`.",
                    ))
                    .with_annotation(Annotation::primary(primary_error_span).with_message(format!(
                        "This assignment to type `{assigned_type_str}` may violate a reference constraint.",
                    )))
                    .with_annotation(Annotation::secondary(constraint.constraint_span).with_message(
                        format!("Variable was passed as a by-reference argument here, constraining it to type `{constraint_type_str}`."),
                    ))
                    .with_note(
                        "An object may still hold a reference to this variable. Changing its type can lead to a `TypeError` at runtime.",
                    )
                    .with_help(
                        "Ensure this variable's type remains compatible, or refactor to avoid holding onto the external reference.",
                    )
                },
                ReferenceConstraintSource::Static => {
                    Issue::error(format!(
                        "Invalid assignment to constrained static variable `{variable_id}`.",
                    ))
                    .with_annotation(Annotation::primary(primary_error_span).with_message(format!(
                        "This value of type `{assigned_type_str}` is not compatible with the expected type `{constraint_type_str}`.",
                    )))
                    .with_annotation(Annotation::secondary(constraint.constraint_span).with_message(
                        "Static variable's type is constrained here.",
                    ))
                    .with_note(
                        "Static variables maintain their state across function calls. Violating the type constraint can cause errors in subsequent calls.",
                    )
                    .with_help(format!(
                        "Ensure the assigned value is compatible with the `{constraint_type_str}` type.",
                    ))
                },
                ReferenceConstraintSource::Global => {
                    Issue::error(format!(
                        "Invalid assignment to constrained global variable `{variable_id}`.",
                    ))
                    .with_annotation(Annotation::primary(primary_error_span).with_message(format!(
                        "This value of type `{assigned_type_str}` is not compatible with the global's expected type `{constraint_type_str}`.",
                    )))
                    .with_annotation(Annotation::secondary(constraint.constraint_span).with_message(
                        "Global variable is imported with a type constraint here.",
                    ))
                    .with_note(
                        "Global variables are shared across the application. Changing a global to an incompatible type can lead to widespread errors.",
                    )
                    .with_help(format!(
                        "Ensure the assigned value is compatible with the `{constraint_type_str}` type.",
                    ))
                },
            };

            context.collector.report_with_code(IssueCode::ReferenceConstraintViolation, issue);
        }

        assigned_type.by_reference = true;
    }

    if block_context.references_possibly_from_confusing_scope.contains(variable_id) {
        context.collector.report_with_code(
            IssueCode::ReferenceReusedFromConfusingScope,
            Issue::warning("Potential unintended modification: This variable may still hold a reference to another variable from a preceding scope.")
                .with_annotation(
                    Annotation::primary(variable_span)
                        .with_message("Assigning a new value here may unintentionally modify the variable this reference points to.")
                )
                .with_note("In PHP, a reference assigned within a block (e.g., an `if` statement or a `foreach` loop) remains active after the block has finished executing.")
                .with_note("Reusing the variable without `unset()` can lead to unexpected side effects.")
                .with_help("To safely reuse this variable, first break the reference by calling `unset()`. For example: `unset($value);`"),
        );
    }

    if variable_id.eq("$this") {
        context.collector.report_with_code(
            IssueCode::AssignmentToThis,
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

        context.collector.report_with_code(
            IssueCode::ImpossibleAssignment,
            issue
                .with_note(
                    "An expression with type `never` is guaranteed to exit, throw, or loop indefinitely."
                )
                .with_help(
                    "This assignment is unreachable because the right-hand side never completes. Remove the assignment or refactor the preceding code."
                )
        );
    }

    let mut from_docblock = false;
    if let Some((variable_type, variable_type_span)) =
        get_type_from_var_docblock(context, block_context, artifacts, Some(variable_id), !destructuring)
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

    if !from_docblock && assigned_type.is_mixed() && !variable_id.starts_with("$_") {
        let assigned_type_str = assigned_type.get_id();

        let mut issue = Issue::warning(format!(
            "Assigning `{assigned_type_str}` type to a variable may lead to unexpected behavior."
        ));

        if let Some(source_expression) = source_expression
            && let Expression::Binary(_) = source_expression
        {
            issue = issue.with_annotation(
                Annotation::secondary(source_expression.span())
                    .with_message(format!("This expression has type `{assigned_type_str}`.")),
            );
        }

        context.collector.report_with_code(
            IssueCode::MixedAssignment,
            issue.with_annotation(Annotation::primary(variable_span).with_message(format!("Assigning `{assigned_type_str}` type here.")))
                .with_note(format!("Using `{assigned_type_str}` can lead to runtime errors if the variable is used in a way that assumes a specific type."))
                .with_help("Consider using a more specific type to avoid potential issues."),
        );
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

fn analyze_destructuring<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    target_span: Span, // the span of the destructuring target ( list or array )
    source_expression: Option<&'ast Expression<'arena>>, // the expression being destructured
    array_type: TUnion, // the type of the array being destructured
    target_elements: &'ast [ArrayElement<'arena>], // the elements being destructured
) -> Result<(), AnalysisError> {
    let mut non_array = false;

    if !array_type.is_array() {
        let assigned_type_str = array_type.get_id();

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

        context.collector.report_with_code(IssueCode::InvalidDestructuringSource, issue);

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

            context.collector.report_with_code(IssueCode::MixedDestructuringShape, issue);

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

            context.collector.report_with_code(IssueCode::SkipInKeyedDestructuring, issue);

            impossible = true;
        }
    }

    for target_element in target_elements {
        match target_element {
            ArrayElement::KeyValue(key_value_element) => {
                key_value_element.key.analyze(context, block_context, artifacts)?;

                let index_type =
                    artifacts.get_expression_type(key_value_element.key).cloned().unwrap_or_else(get_mixed);

                let access_type = if impossible {
                    get_never()
                } else {
                    get_array_target_type_given_index(
                        context,
                        block_context,
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
                    key_value_element.value,
                    None,
                    Some(key_value_element.key),
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
                    value_element.value,
                    None,
                    None,
                    Some(&access_type),
                )?;
            }
            ArrayElement::Variadic(variadic_element) => {
                context.collector.report_with_code(
                    IssueCode::SpreadInDestructuring,
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
                    variadic_element.value,
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

fn analyze_assignment_target<'ctx, 'arena>(
    expression: &Expression<'arena>,
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
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
                        analyze_assignment_target(key_value_array_element.value, context, block_context, artifacts)?;
                    }
                    ArrayElement::Value(value_array_element) => {
                        analyze_assignment_target(value_array_element.value, context, block_context, artifacts)?;
                    }
                    ArrayElement::Variadic(variadic_array_element) => {
                        analyze_assignment_target(variadic_array_element.value, context, block_context, artifacts)?;
                    }
                    ArrayElement::Missing(_) => {}
                }
            }
        }
        Expression::ArrayAccess(array_access) => {
            analyze_assignment_target(array_access.array, context, block_context, artifacts)?;
            analyze_assignment_target(array_access.index, context, block_context, artifacts)?;
        }
        Expression::Access(Access::Property(property_access)) => {
            analyze_assignment_target(property_access.object, context, block_context, artifacts)?;
        }
        Expression::Access(Access::NullSafeProperty(null_safe_property_access)) => {
            analyze_assignment_target(null_safe_property_access.object, context, block_context, artifacts)?;
        }
        Expression::Access(Access::StaticProperty(static_property_access)) => {
            analyze_assignment_target(static_property_access.class, context, block_context, artifacts)?;
        }
        _ => {}
    }

    Ok(())
}

fn handle_assignment_with_boolean_logic<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    variable_expression_id: Span,
    source_expression: &Expression<'arena>,
    variable_id: &str,
) {
    let Some(right_clauses) = get_formula(
        source_expression.span(),
        source_expression.span(),
        source_expression,
        context.get_assertion_context_from_block(block_context),
        artifacts,
    ) else {
        // Complex clauses
        return;
    };

    let right_clauses =
        BlockContext::filter_clauses(context, variable_id, right_clauses.into_iter().map(Rc::new).collect(), None);

    let mut possibilities = IndexMap::default();
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

const fn is_closure_expression<'arena>(expression: &'arena Expression<'arena>) -> bool {
    if let Expression::Parenthesized(parenthesized) = expression {
        return is_closure_expression(parenthesized.expression);
    }

    matches!(expression, Expression::Closure(_))
}

fn get_closure_expression_span<'arena>(expression: &'arena Expression<'arena>) -> Option<Span> {
    if let Expression::Parenthesized(parenthesized) = expression {
        return get_closure_expression_span(parenthesized.expression);
    }

    if matches!(expression, Expression::Closure(_)) { Some(expression.span()) } else { None }
}

fn get_closure_expression_type<'arena>(expression: &'arena Expression<'arena>) -> Option<TUnion> {
    let span = get_closure_expression_span(expression)?;

    Some(TUnion::from_atomic(TAtomic::Callable(TCallable::Alias(FunctionLikeIdentifier::Closure(
        span.file_id,
        span.start,
    )))))
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::code::IssueCode;
    use crate::settings::Settings;
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
            IssueCode::DocblockTypeMismatch,
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
            IssueCode::MismatchedArrayIndex,
            IssueCode::MismatchedArrayIndex,
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
            IssueCode::UndefinedStringArrayIndex,
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
            IssueCode::UndefinedIntArrayIndex,
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
        settings = Settings {
            allow_possibly_undefined_array_keys: false,
            ..Default::default()
        },
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
            IssueCode::PossiblyUndefinedArrayIndex,
            IssueCode::PossiblyUndefinedArrayIndex,
        ]
    }

    test_analysis! {
        name = destructuring_list_from_typed_array_with_undefined_keys,
        settings = Settings {
            allow_possibly_undefined_array_keys: true,
            ..Default::default()
        },
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
    }

    test_analysis! {
        name = expression_is_too_complex,
        code = indoc! {r#"
            <?php

            function is_special_case(int $id, int $count, float $score, float $threshold, bool $is_active, bool $is_admin, string $name, string $role, string $permission, string $category): bool {
                $result = (
                    ($id > 1000 && $count < 5 || $score >= 99.5 && $threshold < $score || $name === 'azjezz' && $role !== 'guest') &&
                    ($is_active && !$is_admin || $permission === 'write' && ($category === 'critical' || $category === 'urgent')) ||
                    !($count === 0 || $id < 0) && (
                        $role === 'admin' && $is_admin ||
                        $name !== 'guest' && $permission !== 'none' ||
                        ($score - $threshold) > 5.0 && $count > 1
                    ) && (
                        $category === 'general' || $category === 'special' ||
                        ($is_active && $is_admin && $id % 2 === 0) ||
                        ($name !== 'system' && $role !== 'user' && $score < 50.0)
                    ) || (
                        $id < 0 && $count > 100 ||
                        ($score < 10.0 && $threshold > 20.0) ||
                        ($is_active && $is_admin && $name === 'root') ||
                        ($role === 'guest' && $permission === 'read' && $category === 'public')
                    )
                );

                return $result;
            }
        "#},
        issues = [
            IssueCode::ExpressionIsTooComplex,
        ]
    }
}
