use std::rc::Rc;

use mago_codex::ttype::TType;
use mago_codex::ttype::get_bool;
use mago_codex::ttype::get_false;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_true;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::artifacts::get_expression_range;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;
use crate::expression::binary::utils::are_definitely_not_identical;
use crate::expression::binary::utils::is_always_greater_than;
use crate::expression::binary::utils::is_always_greater_than_or_equal;
use crate::expression::binary::utils::is_always_identical_to;
use crate::expression::binary::utils::is_always_less_than;
use crate::expression::binary::utils::is_always_less_than_or_equal;

/// Analyzes standard comparison operations (e.g., `==`, `===`, `<`, `<=`, `>`, `>=`).
///
/// All these operations result in a boolean. This function:
/// 1. Analyzes both left and right operands.
/// 2. Calls `check_comparison_operand` to validate each operand's type for comparison.
/// 3. Sets the result type of the binary expression to `bool`.
/// 4. Reports warnings for potentially problematic comparisons (e.g., array with int).
/// 5. Reports errors for invalid comparisons (e.g., involving `mixed`).
/// 6. Reports hints for redundant comparisons where the outcome is statically known.
/// 7. Establishes data flow from operands to the expression node.
pub fn analyze_comparison_operation<'ctx, 'arena>(
    binary: &Binary<'arena>,
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    let was_inside_general_use = block_context.inside_general_use;
    block_context.inside_general_use = true;
    binary.lhs.analyze(context, block_context, artifacts)?;
    binary.rhs.analyze(context, block_context, artifacts)?;
    block_context.inside_general_use = was_inside_general_use;

    let fallback_type = Rc::new(get_mixed());
    let lhs_type = artifacts.get_rc_expression_type(&binary.lhs).unwrap_or(&fallback_type);
    let rhs_type = artifacts.get_rc_expression_type(&binary.rhs).unwrap_or(&fallback_type);

    check_comparison_operand(context, binary.lhs, lhs_type, "Left", &binary.operator)?;
    check_comparison_operand(context, binary.rhs, rhs_type, "Right", &binary.operator)?;

    let mut reported_general_invalid_operand = false;

    if !lhs_type.is_mixed() && !rhs_type.is_mixed() {
        let lhs_is_array = lhs_type.is_array();
        let rhs_is_array = rhs_type.is_array();

        if lhs_is_array && !rhs_is_array && !rhs_type.is_null() {
            context.collector.report_with_code(
                IssueCode::InvalidOperand,
                Issue::warning(format!(
                    "Comparing an `array` with a non-array type `{}` using `{}`.",
                    rhs_type.get_id(),
                    binary.operator.as_str()
                ))
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is an array"))
                .with_annotation(Annotation::secondary(binary.rhs.span()).with_message(format!("This has type `{}`", rhs_type.get_id())))
                .with_note("PHP's comparison rules for arrays against other types can be non-obvious (e.g., an array is usually considered 'greater' than non-null scalars).")
                .with_help("Ensure both operands are of comparable types or explicitly cast/convert them before comparison if this behavior is not intended."),
            );
            reported_general_invalid_operand = true;
        } else if !lhs_is_array && rhs_is_array && !lhs_type.is_null() {
            context.collector.report_with_code(
                IssueCode::InvalidOperand,
                Issue::warning(format!(
                    "Comparing a non-array type `{}` with an `array` using `{}`.",
                    lhs_type.get_id(),
                    binary.operator.as_str()
                ))
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message(format!("This has type `{}`", lhs_type.get_id())))
                .with_annotation(Annotation::secondary(binary.rhs.span()).with_message("This is an array"))
                .with_note("PHP's comparison rules for arrays against other types can be non-obvious.")
                .with_help("Ensure both operands are of comparable types or explicitly cast/convert them before comparison if this behavior is not intended."),
            );
            reported_general_invalid_operand = true;
        }
    }

    let result_type = if !reported_general_invalid_operand {
        match binary.operator {
            BinaryOperator::LessThan(_) => {
                if is_always_less_than(lhs_type, rhs_type) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(context, artifacts, binary, "always less than", "`true`");
                    }

                    get_true()
                } else if is_always_greater_than_or_equal(lhs_type, rhs_type) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(context, artifacts, binary, "never less than", "`false`");
                    }

                    get_false()
                } else {
                    get_bool()
                }
            }
            BinaryOperator::LessThanOrEqual(_) => {
                if is_always_less_than_or_equal(lhs_type, rhs_type) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(
                            context,
                            artifacts,
                            binary,
                            "always less than or equal to",
                            "`true`",
                        );
                    }

                    get_true()
                } else if is_always_greater_than(lhs_type, rhs_type) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(
                            context,
                            artifacts,
                            binary,
                            "never less than or equal to",
                            "`false`",
                        );
                    }

                    get_false()
                } else {
                    get_bool()
                }
            }
            BinaryOperator::GreaterThan(_) => {
                if is_always_greater_than(lhs_type, rhs_type) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(context, artifacts, binary, "always greater than", "`true`");
                    }

                    get_true()
                } else if is_always_less_than_or_equal(lhs_type, rhs_type) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(context, artifacts, binary, "never greater than", "`false`");
                    }

                    get_false()
                } else {
                    get_bool()
                }
            }
            BinaryOperator::GreaterThanOrEqual(_) => {
                if is_always_greater_than_or_equal(lhs_type, rhs_type) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(
                            context,
                            artifacts,
                            binary,
                            "always greater than or equal to",
                            "`true`",
                        );
                    }

                    get_true()
                } else if is_always_less_than(lhs_type, rhs_type) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(
                            context,
                            artifacts,
                            binary,
                            "never greater than or equal to",
                            "`false`",
                        );
                    }

                    get_false()
                } else {
                    get_bool()
                }
            }
            BinaryOperator::Equal(_) | BinaryOperator::AngledNotEqual(_) => {
                if is_always_identical_to(lhs_type, rhs_type) {
                    if !block_context.inside_loop_expressions {
                        let (message_verb, result_value_str) = if matches!(binary.operator, BinaryOperator::Equal(_)) {
                            ("always equal to", "`true`")
                        } else {
                            ("never equal to (always not equal)", "`false`")
                        };

                        report_redundant_comparison(context, artifacts, binary, message_verb, result_value_str);
                    }

                    if matches!(binary.operator, BinaryOperator::Equal(_)) { get_true() } else { get_false() }
                } else {
                    get_bool()
                }
            }
            BinaryOperator::NotEqual(_) => {
                if is_always_identical_to(lhs_type, rhs_type) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(
                            context,
                            artifacts,
                            binary,
                            "never equal to (always false for !=)",
                            "`false`",
                        );
                    }

                    get_false()
                } else {
                    get_bool()
                }
            }
            BinaryOperator::Identical(_) => {
                if is_always_identical_to(lhs_type, rhs_type) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(context, artifacts, binary, "always identical to", "`true`");
                    }

                    get_true()
                } else if are_definitely_not_identical(context.codebase, lhs_type, rhs_type, false) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(context, artifacts, binary, "never identical to", "`false`");
                    }

                    get_false()
                } else {
                    get_bool()
                }
            }
            BinaryOperator::NotIdentical(_) => {
                if is_always_identical_to(lhs_type, rhs_type) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(
                            context,
                            artifacts,
                            binary,
                            "never identical to (always false for !==)",
                            "`false`",
                        );
                    }

                    get_false()
                } else if are_definitely_not_identical(context.codebase, lhs_type, rhs_type, false) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(context, artifacts, binary, "always not identical to", "`true`");
                    }
                    get_true()
                } else {
                    get_bool()
                }
            }
            _ => get_bool(),
        }
    } else {
        get_bool()
    };

    artifacts.expression_types.insert(get_expression_range(binary), Rc::new(result_type));

    Ok(())
}

/// Checks a single operand of a comparison operation for problematic types.
fn check_comparison_operand<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    operand: &'ast Expression<'arena>,
    operand_type: &TUnion,
    side: &'static str,
    operator: &'ast BinaryOperator<'arena>,
) -> Result<(), AnalysisError> {
    if operator.is_identity() {
        return Ok(());
    }

    let op_str = operator.as_str();

    if operand_type.is_null() {
        context.collector.report_with_code(
            IssueCode::NullOperand,
            Issue::error(format!(
                "{side} operand in `{op_str}` comparison is `null`."
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This is `null`"))
            .with_note(format!("Comparing `null` with `{op_str}` can lead to unexpected results due to PHP's type coercion rules (e.g., `null == 0` is true)."))
            .with_help("Ensure this operand is non-null and has a comparable type. Explicitly check for `null` if it's an expected state."),
        );
    } else if operand_type.is_nullable() && !operand_type.is_mixed() {
        context.collector.report_with_code(
            IssueCode::PossiblyNullOperand,
            Issue::warning(format!(
                "{} operand in `{}` comparison might be `null` (type `{}`).",
                side, op_str, operand_type.get_id()
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This might be `null`"))
            .with_note(format!("If this operand is `null` at runtime, PHP's specific comparison rules for `null` with `{op_str}` will apply."))
            .with_help("Ensure this operand is non-null or that comparison with `null` is intended and handled safely."),
        );
    } else if operand_type.is_mixed() {
        context.collector.report_with_code(
            IssueCode::MixedOperand,
            Issue::error(format!("{side} operand in `{op_str}` comparison has `mixed` type."))
                .with_annotation(Annotation::primary(operand.span()).with_message("This has type `mixed`"))
                .with_note(format!(
                    "The result of comparing `mixed` types with `{op_str}` is unpredictable and can hide bugs."
                ))
                .with_help("Ensure this operand has a known, comparable type before using this comparison operator."),
        );
    } else if operand_type.is_false() {
        context.collector.report_with_code(
            IssueCode::FalseOperand,
            Issue::error(format!(
               "{side} operand in `{op_str}` comparison is `false`."
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This is `false`"))
            .with_note(format!("PHP compares `false` with other types according to specific rules (e.g., `false == 0` is true using `{op_str}`). This can hide bugs."))
            .with_help("Ensure this operand is not `false` or explicitly handle the `false` case if it represents a distinct state (e.g., an error from a function)."),
        );
    } else if operand_type.is_falsable() && !operand_type.ignore_falsable_issues {
        context.collector.report_with_code(
            IssueCode::PossiblyFalseOperand,
            Issue::warning(format!(
                "{} operand in `{}` comparison might be `false` (type `{}`).",
                side, op_str, operand_type.get_id()
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This might be `false`"))
            .with_note(format!("If this operand is `false` at runtime, PHP's specific comparison rules for `false` with `{op_str}` will apply."))
            .with_help("Ensure this operand is non-false or that comparison with `false` is intended and handled safely."),
        );
    }

    Ok(())
}

/// Helper to report redundant comparison issues.
fn report_redundant_comparison<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    artifacts: &mut AnalysisArtifacts,
    binary: &'ast Binary<'arena>,
    comparison_description: &str,
    result_value_str: &str,
) {
    let operator_span = binary.operator.span();
    if operator_span.is_zero() {
        // this is a synthetic node, do not report it.
        return;
    }

    context.collector.report_with_code(
        IssueCode::RedundantComparison,
        Issue::help(format!(
            "Redundant `{}` comparison: left-hand side is {} right-hand side.",
            binary.operator.as_str(),
            comparison_description
        ))
        .with_annotation(Annotation::primary(binary.lhs.span()).with_message(
            match artifacts.get_expression_type(&binary.lhs) {
                Some(t) => format!("Left operand is `{}`", t.get_id()),
                None => "Left operand type is unknown".to_string(),
            },
        ))
        .with_annotation(Annotation::secondary(binary.rhs.span()).with_message(
            match artifacts.get_expression_type(&binary.rhs) {
                Some(t) => format!("Right operand is `{}`", t.get_id()),
                None => "Right operand type is unknown".to_string(),
            },
        ))
        .with_note(format!(
            "The `{}` operator will always return {} in this case.",
            binary.operator.as_str(),
            result_value_str
        ))
        .with_help(format!(
            "Consider simplifying or removing this comparison as it always evaluates to {result_value_str}."
        )),
    );
}
