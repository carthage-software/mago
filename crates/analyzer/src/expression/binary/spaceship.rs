use std::rc::Rc;

use mago_codex::ttype::TType;
use mago_codex::ttype::get_minus_one_int;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::get_one_int;
use mago_codex::ttype::get_signum_result;
use mago_codex::ttype::get_zero_int;
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
use crate::expression::binary::utils::is_always_greater_than;
use crate::expression::binary::utils::is_always_identical_to;
use crate::expression::binary::utils::is_always_less_than;

/// Analyzes the spaceship operator (`LHS <=> RHS`).
///
/// The spaceship operator always returns an integer (`-1`, `0`, or `1`).
/// This function analyzes both operands, sets the result type to `int`,
/// reports warnings for potentially problematic comparisons (e.g., array with int),
/// and errors for invalid comparisons (e.g., involving `mixed`).
/// Data flow is established from both operands.
pub fn analyze_spaceship_operation<'ctx, 'arena>(
    binary: &Binary<'arena>,
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    binary.lhs.analyze(context, block_context, artifacts)?;
    binary.rhs.analyze(context, block_context, artifacts)?;

    let fallback_type = Rc::new(get_mixed());
    let lhs_type = artifacts.get_rc_expression_type(&binary.lhs).unwrap_or(&fallback_type);
    let rhs_type = artifacts.get_rc_expression_type(&binary.rhs).unwrap_or(&fallback_type);

    check_spaceship_operand(context, binary.lhs, lhs_type, "Left")?;
    check_spaceship_operand(context, binary.rhs, rhs_type, "Right")?;

    let lhs_is_array = lhs_type.is_array();
    let rhs_is_array = rhs_type.is_array();

    if lhs_is_array && !rhs_is_array && !rhs_type.is_null() {
        context.collector.report_with_code(
            IssueCode::InvalidOperand,
            Issue::error(format!(
                "Comparing an `array` with a non-array type `{}` using `<=>`.",
                rhs_type.get_id()
            ))
            .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is an array"))
            .with_annotation(Annotation::secondary(binary.rhs.span()).with_message(format!("This has type `{}`", rhs_type.get_id())))
            .with_note("PHP compares arrays as greater than other types (except other arrays and null). This might not be the intended comparison.")
            .with_help("Ensure both operands are of comparable types or explicitly cast/convert them before comparison."),
        );
    } else if !lhs_is_array && rhs_is_array && !lhs_type.is_null() {
        context.collector.report_with_code(
            IssueCode::InvalidOperand,
            Issue::error(format!(
                "Comparing a non-array type `{}` with an `array` using `<=>`.",
                lhs_type.get_id()
            ))
            .with_annotation(Annotation::primary(binary.lhs.span()).with_message(format!("This has type `{}`", lhs_type.get_id())))
            .with_annotation(Annotation::secondary(binary.rhs.span()).with_message("This is an array"))
            .with_note("PHP compares arrays as greater than other types (except other arrays and null). This might not be the intended comparison.")
            .with_help("Ensure both operands are of comparable types or explicitly cast/convert them before comparison."),
        );
    }

    let result_type = if !block_context.inside_loop_expressions && is_always_greater_than(lhs_type, rhs_type) {
        context.collector.report_with_code(
            IssueCode::RedundantTypeComparison,
            Issue::help("Redundant spaceship comparison: left-hand side is always greater than right-hand side.")
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is always greater"))
                .with_annotation(Annotation::secondary(binary.rhs.span()).with_message("This is always less"))
                .with_note("The spaceship operator `<=>` will always return `1` in this case.")
                .with_help("Consider removing this comparison. It will always evaluate to `1`."),
        );

        get_one_int()
    } else if !block_context.inside_loop_expressions && is_always_identical_to(lhs_type, rhs_type) {
        context.collector.report_with_code(
            IssueCode::RedundantTypeComparison,
            Issue::help("Redundant spaceship comparison: left-hand side is always equal to right-hand side.")
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is always equal"))
                .with_annotation(Annotation::secondary(binary.rhs.span()).with_message("This is always equal"))
                .with_note("The spaceship operator `<=>` will always return `0` in this case.")
                .with_help("Consider removing this comparison. It will always evaluate to `0`."),
        );

        get_zero_int()
    } else if !block_context.inside_loop_expressions && is_always_less_than(lhs_type, rhs_type) {
        context.collector.report_with_code(
            IssueCode::RedundantTypeComparison,
            Issue::help("Redundant spaceship comparison: left-hand side is always less than right-hand side.")
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is always less"))
                .with_annotation(Annotation::secondary(binary.rhs.span()).with_message("This is always greater"))
                .with_note("The spaceship operator `<=>` will always return `-1` in this case.")
                .with_help("Consider removing this comparison. It will always evaluate to `-1`."),
        );

        get_minus_one_int()
    } else {
        get_signum_result()
    };

    artifacts.expression_types.insert(get_expression_range(binary), Rc::new(result_type));

    Ok(())
}

fn check_spaceship_operand<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    operand: &'ast Expression<'arena>,
    operand_type: &TUnion,
    side: &'static str,
) -> Result<(), AnalysisError> {
    if operand_type.is_null() {
        context.collector.report_with_code(
             IssueCode::NullOperand,
             Issue::error(format!(
                 "{side} operand in spaceship comparison (`<=>`) is `null`."
             ))
             .with_annotation(Annotation::primary(operand.span()).with_message("This is `null`"))
             .with_note("PHP compares `null` with other types according to specific rules (e.g., `null == 0` is true, `null < 1` is true).")
             .with_help("Ensure this comparison with `null` is intended, or provide a non-null operand."),
         );
    } else if operand_type.is_nullable() && !operand_type.is_mixed() {
        context.collector.report_with_code(
            IssueCode::PossiblyNullOperand,
            Issue::warning(format!(
                "{side} operand in spaceship comparison (`<=>`) might be `null` (type `{}`).",
                operand_type.get_id()
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This might be `null`"))
            .with_note("If this operand is `null` at runtime, PHP's specific comparison rules for `null` will apply.")
            .with_help("Ensure this operand is non-null or that comparison with `null` is intended."),
        );
    } else if operand_type.is_mixed() {
        context.collector.report_with_code(
            IssueCode::MixedOperand,
            Issue::error(format!("{side} operand in spaceship comparison (`<=>`) has `mixed` type."))
                .with_annotation(Annotation::primary(operand.span()).with_message("This has type `mixed`"))
                .with_note("The result of comparing `mixed` types with `<=>` is unpredictable.")
                .with_help("Ensure this operand has a known, comparable type before using the spaceship operator."),
        );
    } else if operand_type.is_false() {
        context.collector.report_with_code(
            IssueCode::FalseOperand,
            Issue::error(format!(
                "{side} operand in spaceship comparison (`<=>`) is `false`."
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This is `false`"))
            .with_note("PHP compares `false` with other types according to specific rules (e.g., `false == 0` is true, `false < 1` is true).")
            .with_help("Ensure this comparison with `false` is intended, or provide a non-false operand."),
        );
    } else if operand_type.is_falsable() && !operand_type.ignore_falsable_issues {
        context.collector.report_with_code(
            IssueCode::PossiblyFalseOperand,
            Issue::warning(format!(
                "{side} operand in spaceship comparison (`<=>`) might be `false` (type `{}`).",
                operand_type.get_id()
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This might be `false`"))
            .with_note("If this operand is `false` at runtime, PHP's specific comparison rules for `false` will apply.")
            .with_help("Ensure this operand is non-false or that comparison with `false` is intended."),
        );
    }

    Ok(())
}
