use std::borrow::Cow;
use std::collections::VecDeque;
use std::rc::Rc;

use ahash::HashMap;
use ahash::HashSet;

use mago_algebra::find_satisfying_assignments;
use mago_algebra::saturate_clauses;
use mago_codex::data_flow::node::DataFlowNode;
use mago_codex::data_flow::path::PathKind;
use mago_codex::get_method_by_id;
use mago_codex::get_method_id;
use mago_codex::metadata::CodebaseMetadata;
use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::mixed::TMixed;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::atomic::scalar::string::TStringLiteral;
use mago_codex::ttype::combine_union_types;
use mago_codex::ttype::combiner;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::atomic_comparator;
use mago_codex::ttype::comparator::union_comparator::can_expression_types_be_identical;
use mago_codex::ttype::get_arraykey;
use mago_codex::ttype::get_bool;
use mago_codex::ttype::get_false;
use mago_codex::ttype::get_mixed_any;
use mago_codex::ttype::get_true;
use mago_codex::ttype::union::TUnion;
use mago_interner::ThreadedInterner;
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
use crate::context::scope::if_scope::IfScope;
use crate::error::AnalysisError;
use crate::expression::add_decision_dataflow;
use crate::formula::get_formula;
use crate::formula::negate_or_synthesize;
use crate::issue::TypingIssueKind;
use crate::reconciler;
use crate::reconciler::ReconcilationContext;
use crate::utils::conditional;

impl Analyzable for Binary {
    fn analyze<'a>(
        &self,
        context: &mut Context<'a>,
        block_context: &mut BlockContext<'a>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        match &self.operator {
            BinaryOperator::Addition(_)
            | BinaryOperator::Subtraction(_)
            | BinaryOperator::Multiplication(_)
            | BinaryOperator::Division(_)
            | BinaryOperator::Modulo(_)
            | BinaryOperator::Exponentiation(_)
            | BinaryOperator::BitwiseAnd(_)
            | BinaryOperator::BitwiseOr(_)
            | BinaryOperator::BitwiseXor(_)
            | BinaryOperator::LeftShift(_)
            | BinaryOperator::RightShift(_) => analyze_arithmetic_operation(self, context, block_context, artifacts),
            BinaryOperator::And(_) | BinaryOperator::LowAnd(_) => {
                analyze_logical_and_operation(self, context, block_context, artifacts)
            }
            BinaryOperator::Or(_) | BinaryOperator::LowOr(_) => {
                analyze_logical_or_operation(self, context, block_context, artifacts)
            }
            BinaryOperator::StringConcat(_) => analyze_string_concat_operation(self, context, block_context, artifacts),
            BinaryOperator::NullCoalesce(_) => analyze_null_coalesce_operation(self, context, block_context, artifacts),
            BinaryOperator::Elvis(_) => analyze_elvis_operation(self, context, block_context, artifacts),
            BinaryOperator::Spaceship(_) => analyze_spaceship_operation(self, context, block_context, artifacts),
            BinaryOperator::Equal(_)
            | BinaryOperator::NotEqual(_)
            | BinaryOperator::Identical(_)
            | BinaryOperator::NotIdentical(_)
            | BinaryOperator::AngledNotEqual(_)
            | BinaryOperator::LessThan(_)
            | BinaryOperator::LessThanOrEqual(_)
            | BinaryOperator::GreaterThan(_)
            | BinaryOperator::GreaterThanOrEqual(_) => {
                analyze_comparison_operation(self, context, block_context, artifacts)
            }
            BinaryOperator::LowXor(_) => analyze_logical_xor_operation(self, context, block_context, artifacts),
            BinaryOperator::Instanceof(_) => {
                self.lhs.analyze(context, block_context, artifacts)?;

                add_decision_dataflow(artifacts, &self.lhs, None, self.span(), get_bool());

                Ok(())
            }
        }
    }
}

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
fn analyze_comparison_operation<'a>(
    binary: &Binary,
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    let was_inside_general_use = block_context.inside_general_use;
    block_context.inside_general_use = true;
    binary.lhs.analyze(context, block_context, artifacts)?;
    binary.rhs.analyze(context, block_context, artifacts)?;
    block_context.inside_general_use = was_inside_general_use;

    let fallback_type = Rc::new(get_mixed_any());
    let lhs_type = artifacts.get_rc_expression_type(&binary.lhs).unwrap_or(&fallback_type);
    let rhs_type = artifacts.get_rc_expression_type(&binary.rhs).unwrap_or(&fallback_type);

    check_comparison_operand(context, &binary.lhs, lhs_type, "Left", &binary.operator)?;
    check_comparison_operand(context, &binary.rhs, rhs_type, "Right", &binary.operator)?;

    let mut reported_general_invalid_operand = false;

    if !lhs_type.is_mixed() && !rhs_type.is_mixed() {
        let lhs_is_array = lhs_type.is_array();
        let rhs_is_array = rhs_type.is_array();

        if lhs_is_array && !rhs_is_array && !rhs_type.is_null() {
            context.buffer.report(
                TypingIssueKind::InvalidOperand,
                Issue::warning(format!(
                    "Comparing an `array` with a non-array type `{}` using `{}`.",
                    rhs_type.get_id(Some(context.interner)),
                    binary.operator.as_str(context.interner)
                ))
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is an array"))
                .with_annotation(Annotation::secondary(binary.rhs.span()).with_message(format!("This has type `{}`", rhs_type.get_id(Some(context.interner)))))
                .with_note("PHP's comparison rules for arrays against other types can be non-obvious (e.g., an array is usually considered 'greater' than non-null scalars).")
                .with_help("Ensure both operands are of comparable types or explicitly cast/convert them before comparison if this behavior is not intended."),
            );
            reported_general_invalid_operand = true;
        } else if !lhs_is_array && rhs_is_array && !lhs_type.is_null() {
            context.buffer.report(
                TypingIssueKind::InvalidOperand,
                Issue::warning(format!(
                    "Comparing a non-array type `{}` with an `array` using `{}`.",
                    lhs_type.get_id(Some(context.interner)),
                    binary.operator.as_str(context.interner)
                ))
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message(format!("This has type `{}`", lhs_type.get_id(Some(context.interner)))))
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
                if is_always_less_than(lhs_type, rhs_type, context.codebase, context.interner) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(context, artifacts, binary, "always less than", "`true`");
                    }

                    get_true()
                } else if is_always_greater_than_or_equal(lhs_type, rhs_type, context.codebase, context.interner) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(context, artifacts, binary, "never less than", "`false`");
                    }

                    get_false()
                } else {
                    get_bool()
                }
            }
            BinaryOperator::LessThanOrEqual(_) => {
                if is_always_less_than_or_equal(lhs_type, rhs_type, context.codebase, context.interner) {
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
                } else if is_always_greater_than(lhs_type, rhs_type, context.codebase, context.interner) {
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
                if is_always_greater_than(lhs_type, rhs_type, context.codebase, context.interner) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(context, artifacts, binary, "always greater than", "`true`");
                    }

                    get_true()
                } else if is_always_less_than_or_equal(lhs_type, rhs_type, context.codebase, context.interner) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(context, artifacts, binary, "never greater than", "`false`");
                    }

                    get_false()
                } else {
                    get_bool()
                }
            }
            BinaryOperator::GreaterThanOrEqual(_) => {
                if is_always_greater_than_or_equal(lhs_type, rhs_type, context.codebase, context.interner) {
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
                } else if is_always_less_than(lhs_type, rhs_type, context.codebase, context.interner) {
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
                if is_always_identical_to(lhs_type, rhs_type, context.codebase, context.interner) {
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
                if is_always_identical_to(lhs_type, rhs_type, context.codebase, context.interner) {
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
                if is_always_identical_to(lhs_type, rhs_type, context.codebase, context.interner) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(context, artifacts, binary, "always identical to", "`true`");
                    }

                    get_true()
                } else if are_definitely_not_identical(lhs_type, rhs_type, context.codebase, context.interner) {
                    if !block_context.inside_loop_expressions {
                        report_redundant_comparison(context, artifacts, binary, "never identical to", "`false`");
                    }

                    get_false()
                } else {
                    get_bool()
                }
            }
            BinaryOperator::NotIdentical(_) => {
                if is_always_identical_to(lhs_type, rhs_type, context.codebase, context.interner) {
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
                } else if are_definitely_not_identical(lhs_type, rhs_type, context.codebase, context.interner) {
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

    add_decision_dataflow(artifacts, &binary.lhs, Some(&binary.rhs), binary.span(), result_type);

    Ok(())
}

/// Checks a single operand of a comparison operation for problematic types.
fn check_comparison_operand(
    context: &mut Context<'_>,
    operand: &Expression,
    operand_type: &TUnion,
    side: &'static str,
    operator: &BinaryOperator,
) -> Result<(), AnalysisError> {
    if operator.is_identity() {
        return Ok(());
    }

    let op_str = operator.as_str(context.interner);

    if operand_type.is_null() {
        context.buffer.report(
            TypingIssueKind::NullOperand,
            Issue::error(format!(
                "{side} operand in `{op_str}` comparison is `null`."
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This is `null`"))
            .with_note(format!("Comparing `null` with `{op_str}` can lead to unexpected results due to PHP's type coercion rules (e.g., `null == 0` is true)."))
            .with_help("Ensure this operand is non-null and has a comparable type. Explicitly check for `null` if it's an expected state."),
        );
    } else if operand_type.is_nullable() && !operand_type.is_mixed() {
        context.buffer.report(
            TypingIssueKind::PossiblyNullOperand,
            Issue::warning(format!(
                "{} operand in `{}` comparison might be `null` (type `{}`).",
                side, op_str, operand_type.get_id(Some(context.interner))
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This might be `null`"))
            .with_note(format!("If this operand is `null` at runtime, PHP's specific comparison rules for `null` with `{op_str}` will apply."))
            .with_help("Ensure this operand is non-null or that comparison with `null` is intended and handled safely."),
        );
    } else if operand_type.is_mixed() {
        context.buffer.report(
            TypingIssueKind::MixedOperand,
            Issue::error(format!("{side} operand in `{op_str}` comparison has `mixed` type."))
                .with_annotation(Annotation::primary(operand.span()).with_message("This has type `mixed`"))
                .with_note(format!(
                    "The result of comparing `mixed` types with `{op_str}` is unpredictable and can hide bugs."
                ))
                .with_help("Ensure this operand has a known, comparable type before using this comparison operator."),
        );
    } else if operand_type.is_false() {
        context.buffer.report(
            TypingIssueKind::FalseOperand,
            Issue::error(format!(
               "{side} operand in `{op_str}` comparison is `false`."
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This is `false`"))
            .with_note(format!("PHP compares `false` with other types according to specific rules (e.g., `false == 0` is true using `{op_str}`). This can hide bugs."))
            .with_help("Ensure this operand is not `false` or explicitly handle the `false` case if it represents a distinct state (e.g., an error from a function)."),
        );
    } else if operand_type.is_falsable() && !operand_type.ignore_falsable_issues {
        context.buffer.report(
            TypingIssueKind::PossiblyFalseOperand,
            Issue::warning(format!(
                "{} operand in `{}` comparison might be `false` (type `{}`).",
                side, op_str, operand_type.get_id(Some(context.interner))
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This might be `false`"))
            .with_note(format!("If this operand is `false` at runtime, PHP's specific comparison rules for `false` with `{op_str}` will apply."))
            .with_help("Ensure this operand is non-false or that comparison with `false` is intended and handled safely."),
        );
    }

    Ok(())
}

/// Helper to report redundant comparison issues.
fn report_redundant_comparison(
    context: &mut Context<'_>,
    artifacts: &mut AnalysisArtifacts,
    binary: &Binary,
    comparison_description: &str,
    result_value_str: &str,
) {
    context.buffer.report(
        TypingIssueKind::RedundantComparison,
        Issue::help(format!(
            "Redundant `{}` comparison: left-hand side is {} right-hand side.",
            binary.operator.as_str(context.interner),
            comparison_description
        ))
        .with_annotation(Annotation::primary(binary.lhs.span()).with_message(
            match artifacts.get_expression_type(&binary.lhs) {
                Some(t) => format!("Left operand is `{}`", t.get_id(Some(context.interner))),
                None => "Left operand type is unknown".to_string(),
            },
        ))
        .with_annotation(Annotation::secondary(binary.rhs.span()).with_message(
            match artifacts.get_expression_type(&binary.rhs) {
                Some(t) => format!("Right operand is `{}`", t.get_id(Some(context.interner))),
                None => "Right operand type is unknown".to_string(),
            },
        ))
        .with_note(format!(
            "The `{}` operator will always return {} in this case.",
            binary.operator.as_str(context.interner),
            result_value_str
        ))
        .with_help(format!(
            "Consider simplifying or removing this comparison as it always evaluates to {result_value_str}."
        )),
    );
}

#[inline]
fn analyze_string_concat_operation<'a>(
    binary: &Binary,
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    binary.lhs.as_ref().analyze(context, block_context, artifacts)?;
    binary.rhs.as_ref().analyze(context, block_context, artifacts)?;

    analyze_string_concat_operand(context, block_context, artifacts, &binary.lhs, "Left")?;
    analyze_string_concat_operand(context, block_context, artifacts, &binary.rhs, "Right")?;

    let mut result_string = TString::general();
    'precise: {
        let left_string = artifacts.get_expression_type(&binary.lhs).map(|t| get_concat_operand_string(context, t));
        let right_string = artifacts.get_expression_type(&binary.rhs).map(|t| get_concat_operand_string(context, t));

        if let Some(mut left_string) = left_string {
            let Some(right_string) = right_string else {
                left_string.literal = None;
                result_string = left_string;

                break 'precise;
            };

            left_string.is_non_empty = left_string.is_non_empty || right_string.is_non_empty;
            left_string.is_truthy = left_string.is_truthy || right_string.is_truthy;

            let Some(TStringLiteral::Value(left_literal)) = left_string.literal else {
                left_string.literal = None;
                result_string = left_string;

                break 'precise;
            };

            let Some(TStringLiteral::Value(right_literal)) = right_string.literal else {
                left_string.literal = None;
                result_string = left_string;

                break 'precise;
            };

            left_string.literal = Some(TStringLiteral::Value(format!("{left_literal}{right_literal}")));
            result_string = left_string;
        } else if let Some(mut right_string) = right_string {
            right_string.literal = None;
            result_string = right_string;
        }
    };

    let result_type = TUnion::new(vec![TAtomic::Scalar(TScalar::String(result_string))]);

    add_decision_dataflow(artifacts, &binary.lhs, Some(&binary.rhs), binary.span(), result_type);

    Ok(())
}

#[inline]
fn get_concat_operand_string(context: &mut Context<'_>, operand_type: &TUnion) -> TString {
    let mut literals = vec![];
    let mut all_literals = true;
    let mut all_unspecified_literal = true;
    let mut non_empty = false;
    let mut truthy = true;

    for operand_atomic_type in &operand_type.types {
        match operand_atomic_type {
            TAtomic::Array(_) => {
                non_empty = true;
                truthy = true;
                literals.push("Array".to_owned());

                continue;
            }
            TAtomic::Never | TAtomic::Null | TAtomic::Void => {
                continue;
            }
            TAtomic::Resource(_) => {
                non_empty = true;
                truthy = true;
                all_literals = false;
                all_unspecified_literal = false;

                continue;
            }
            _ => {}
        }

        let TAtomic::Scalar(operand_scalar) = operand_atomic_type else {
            all_literals = false;
            all_unspecified_literal = false;

            continue;
        };

        match operand_scalar {
            TScalar::Bool(boolean) => {
                all_unspecified_literal = false;

                if boolean.is_true() {
                    literals.push("1".to_owned());
                    truthy &= true;
                    non_empty = true;
                } else if !boolean.is_false() {
                    all_literals = false;
                }
            }
            TScalar::Integer(tint) => {
                all_unspecified_literal = false;

                if let Some(v) = tint.get_literal_value() {
                    non_empty = true;
                    literals.push(v.to_string());
                    truthy &= v != 0;
                } else {
                    all_literals = false;
                }
            }
            TScalar::Float(tfloat) => {
                all_unspecified_literal = false;

                if let Some(v) = tfloat.get_literal_value() {
                    non_empty = true;
                    literals.push(v.to_string());
                    truthy &= v != 0.0;
                } else {
                    all_literals = false;
                    all_unspecified_literal = false;
                }
            }
            TScalar::String(operand_string) => {
                if let Some(v) = operand_string.get_known_literal_value() {
                    literals.push(v.to_string());
                } else {
                    all_literals = false;
                }

                all_unspecified_literal = all_unspecified_literal && operand_string.is_unspecified_literal();
                non_empty = non_empty || operand_string.is_non_empty();
                truthy &= operand_string.is_truthy();
            }
            TScalar::ClassLikeString(tclass_like_string) => {
                if let Some(id) = tclass_like_string.literal_value() {
                    literals.push(context.interner.lookup(&id).to_string());
                } else {
                    all_literals = false;
                }

                non_empty = true;
                truthy &= true;
            }
            _ => {
                all_literals = false;
                all_unspecified_literal = false;
            }
        }
    }

    TString {
        literal: if all_literals {
            Some(TStringLiteral::Value(literals.join("")))
        } else if all_unspecified_literal {
            Some(TStringLiteral::Unspecified)
        } else {
            None
        },
        is_numeric: false,
        is_truthy: non_empty && truthy,
        is_non_empty: non_empty,
    }
}

#[inline]
fn analyze_string_concat_operand(
    context: &mut Context<'_>,
    block_context: &mut BlockContext<'_>,
    artifacts: &mut AnalysisArtifacts,
    operand: &Expression,
    side: &'static str,
) -> Result<(), AnalysisError> {
    let Some(operand_type) = artifacts.get_expression_type(operand) else {
        return Ok(());
    };

    if operand_type.is_null() {
        context.buffer.report(
            TypingIssueKind::NullOperand,
            Issue::error(format!(
                "Implicit conversion of `null` to empty string for {} operand in string concatenation.",
                side.to_ascii_lowercase()
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("Operand is `null` here"))
            .with_note("Using `null` in string concatenation results in an empty string `''`.")
            .with_help(
                "Explicitly cast to string `(string) $var` or handle the `null` case if concatenation is not intended.",
            ),
        );

        return Ok(());
    }

    if operand_type.is_false() {
        context.buffer.report(
            TypingIssueKind::FalseOperand,
            Issue::error(format!(
                "Implicit conversion of `false` to empty string for {} operand in string concatenation.",
                side.to_ascii_lowercase()
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("Operand is `false` here"))
            .with_note("Using `false` in string concatenation results in an empty string `''`.")
            .with_help("Explicitly cast to string `(string) $var` or handle the `false` case if concatenation is not intended."),
        );

        return Ok(());
    }

    if operand_type.is_nullable() && !operand_type.ignore_nullable_issues {
        context.buffer.report(
            TypingIssueKind::PossiblyNullOperand,
            Issue::warning(format!(
                "Possibly null {} operand used in string concatenation (type `{}`).",
                side.to_ascii_lowercase(),
                operand_type.get_id(Some(context.interner))
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This might be `null`"))
            .with_note("If this operand is `null` at runtime, it will be implicitly converted to an empty string `''`.")
            .with_help("Ensure the operand is non-null before concatenation using checks or assertions, or explicitly cast to string."),
        );
    }

    if operand_type.is_falsable() && !operand_type.ignore_falsable_issues {
        context.buffer.report(
            TypingIssueKind::PossiblyFalseOperand,
            Issue::warning(format!(
                "Possibly false {} operand used in string concatenation (type `{}`).",
                side.to_ascii_lowercase(),
                operand_type.get_id(Some(context.interner))
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This might be `false`"))
            .with_note(
                "If this operand is `false` at runtime, it will be implicitly converted to an empty string `''`.",
            )
            .with_help("Ensure the operand is non-falsy before concatenation, or explicitly cast to string."),
        );
    }

    let mut overall_type_match = true;
    let mut has_at_least_one_valid_operand_type = false;
    let mut reported_invalid_issue = false;

    for operand_atomic_type in &operand_type.types {
        if operand_atomic_type.is_any_string()
            || operand_atomic_type.is_int()
            || operand_atomic_type.is_null()
            || operand_atomic_type.is_false()
        {
            has_at_least_one_valid_operand_type = true;
            continue;
        }

        let mut current_atomic_is_valid = false;

        match operand_atomic_type {
            TAtomic::GenericParameter(parameter) => {
                if parameter.constraint.is_any_string()
                    || parameter.constraint.is_int()
                    || parameter.constraint.is_mixed()
                {
                    current_atomic_is_valid = true;
                } else {
                    if !reported_invalid_issue {
                        context.buffer.report(
                            TypingIssueKind::InvalidOperand,
                            Issue::error(format!(
                                "Invalid {} operand: template parameter `{}` constraint `{}` is not compatible with string concatenation.",
                                side.to_ascii_lowercase(),
                                context.interner.lookup(&parameter.parameter_name),
                                parameter.constraint.get_id(Some(context.interner))
                            ))
                            .with_annotation(Annotation::primary(operand.span()).with_message("Template type not guaranteed to be string/numeric"))
                            .with_help("Ensure the template parameter constraint allows string conversion or cast the value explicitly."),
                        );

                        reported_invalid_issue = true;
                    }

                    overall_type_match = false;
                }
            }

            TAtomic::Object(object) => {
                let Some(class_like_name) = object.get_name() else {
                    if !reported_invalid_issue {
                        context.buffer.report(
                            TypingIssueKind::InvalidOperand,
                            Issue::error(format!(
                                "Invalid {} operand: cannot determine if generic `object` is stringable.",
                                side.to_ascii_lowercase()
                            ))
                            .with_annotation(
                                Annotation::primary(operand.span())
                                    .with_message("Cannot verify `__toString` for generic `object`"),
                            )
                            .with_note("Only objects with a `__toString` method can be used in string concatenation.")
                            .with_help("Use a more specific object type or ensure the object implements `Stringable`."),
                        );

                        reported_invalid_issue = true;
                    }

                    overall_type_match = false;
                    continue;
                };

                let to_string_method_name = context.interner.intern("__toString");
                let method_id = get_method_id(class_like_name, &to_string_method_name);

                if let Some(method_metadata) = get_method_by_id(context.codebase, context.interner, &method_id) {
                    current_atomic_is_valid = true;

                    context.buffer.report(
                        TypingIssueKind::ImplicitToStringCast,
                        Issue::warning(format!(
                            "Implicit conversion to `string` for {} operand via `{}`.",
                            side.to_ascii_lowercase(),
                            method_id.as_string(context.interner)
                        ))
                        .with_annotation(Annotation::primary(operand.span())
                            .with_message(format!("Object implicitly converted using `{}`", method_id.as_string(context.interner)))
                        )
                        .with_note("Objects implementing `__toString` are automatically converted when used in string context.")
                        .with_help("For clarity, consider explicit casting `(string) $object` or calling the `__toString` method directly."),
                    );

                    if context.settings.analyze_effects
                        && block_context.is_mutation_free()
                        && !method_metadata.is_mutation_free
                    {
                        context.buffer.report(
                            TypingIssueKind::ImpureCallInPureContext,
                            Issue::error(format!(
                                "Impure `__toString` method called implicitly for {} operand within a mutation-free context.",
                                side.to_ascii_lowercase()
                            ))
                            .with_annotation(Annotation::primary(operand.span())
                                .with_message(format!("Implicit call to non-pure method `{}`", method_id.as_string(context.interner)))
                            )
                            .with_note("Calling methods with side effects violates the mutation-free guarantee.")
                            .with_help(format!("Ensure `{}` is mutation-free, or avoid using this object in string context within mutation-free scopes.", method_id.as_string(context.interner))),
                        );
                    }
                } else {
                    if !reported_invalid_issue {
                        context.buffer.report(
                            TypingIssueKind::InvalidOperand,
                            Issue::error(format!(
                                "Invalid {} operand: object of type `{}` cannot be converted to `string`.",
                                side.to_ascii_lowercase(),
                                operand_atomic_type.get_id(Some(context.interner))
                            ))
                            .with_annotation(Annotation::primary(operand.span())
                                .with_message(format!("Type `{}` does not have a `__toString` method", operand_atomic_type.get_id(Some(context.interner))))
                            )
                            .with_note("Only objects implementing the `Stringable` interface (or having a `__toString` method) can be used in string concatenation.")
                            .with_help("Implement `__toString` on the class or avoid using this object in string context."),
                        );

                        reported_invalid_issue = true;
                    }

                    overall_type_match = false;
                }
            }
            TAtomic::Array(_) => {
                if !reported_invalid_issue {
                    context.buffer.report(
                        TypingIssueKind::ArrayToStringConversion,
                        Issue::error(format!(
                            "Invalid {} operand: cannot use type `array` in string concatenation.",
                            side.to_ascii_lowercase()
                        ))
                        .with_annotation(Annotation::primary(operand.span()).with_message("Cannot concatenate with an `array`"))
                        .with_note("PHP raises an `E_WARNING` or `E_NOTICE` and uses the literal string 'Array' when an array is used in string context.")
                        .with_help("Do not use arrays directly in string concatenation. Use `implode()`, `json_encode()`, or loop to format its contents."),
                    );

                    reported_invalid_issue = true;
                }

                overall_type_match = false;
            }
            TAtomic::Resource(_) => {
                context.buffer.report(
                    TypingIssueKind::ImplicitResourceToStringCast,
                    Issue::warning(format!(
                        "Implicit conversion of `resource` to string for {} operand.",
                        side.to_ascii_lowercase()
                    ))
                    .with_annotation(Annotation::primary(operand.span()).with_message("Resource implicitly converted to string"))
                    .with_note("PHP converts resources to the string format 'Resource id #[id]' when used in string context.")
                    .with_help("Avoid relying on implicit resource-to-string conversion; extract necessary information from the resource first if possible."),
                );

                current_atomic_is_valid = true;
            }
            TAtomic::Mixed(_) => {
                if !reported_invalid_issue {
                    context.buffer.report(
                        TypingIssueKind::MixedOperand,
                        Issue::error(format!(
                            "Invalid {} operand: type `{}` cannot be reliably used in string concatenation.",
                            side.to_ascii_lowercase(),
                            operand_atomic_type.get_id(Some(context.interner))
                        ))
                        .with_annotation(Annotation::primary(operand.span()).with_message("Operand has `mixed` type"))
                        .with_note("Using `mixed` in string concatenation is unsafe as the actual runtime type and its string representation are unknown.")
                        .with_help("Ensure the operand has a known type (`string`, `int`, `null`, `false`, or stringable object) using type hints, assertions, or checks."),
                    );

                    reported_invalid_issue = true;
                }

                overall_type_match = false;
            }
            _ => {
                if !reported_invalid_issue {
                    context.buffer.report(
                        TypingIssueKind::InvalidOperand,
                        Issue::error(format!(
                            "Invalid type `{}` for {} operand in string concatenation.",
                             operand_atomic_type.get_id(Some(context.interner)),
                             side.to_ascii_lowercase()
                        ))
                        .with_annotation(Annotation::primary(operand.span()).with_message("Invalid type for concatenation"))
                        .with_help("Ensure the operand is a string, number, null, false, resource, or an object with `__toString`."),
                    );

                    reported_invalid_issue = true;
                }

                overall_type_match = false;
            }
        }

        has_at_least_one_valid_operand_type = has_at_least_one_valid_operand_type || current_atomic_is_valid;
    }

    if !overall_type_match && !has_at_least_one_valid_operand_type && !reported_invalid_issue {
        context.buffer.report(
            TypingIssueKind::InvalidOperand,
            Issue::error(format!(
                "Invalid type `{}` for {} operand in string concatenation.",
                operand_type.get_id(Some(context.interner)), side.to_ascii_lowercase()
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("Invalid type for concatenation"))
            .with_note("Operands in string concatenation must be strings, numbers, null, false, resources, or objects implementing `__toString`.")
            .with_help("Ensure the operand has a compatible type or cast it explicitly to `string`."),
        );
    } else if !overall_type_match && has_at_least_one_valid_operand_type && !reported_invalid_issue {
        context.buffer.report(
            TypingIssueKind::PossiblyInvalidOperand,
            Issue::warning(format!(
                "Possibly invalid type `{}` for {} operand in string concatenation.",
                operand_type.get_id(Some(context.interner)),
                side.to_ascii_lowercase()
            ))
            .with_annotation(
                Annotation::primary(operand.span()).with_message("Operand type might be invalid for concatenation"),
            )
            .with_note("Some possible types for this operand are not compatible with string concatenation.")
            .with_help(
                "Ensure the operand always has a compatible type using checks or assertions before concatenation.",
            ),
        );
    }

    Ok(())
}

/// Analyzes the logical XOR operator (`xor`).
///
/// The `xor` operator evaluates both operands and returns `true` if exactly one of them is truthy,
/// and `false` otherwise. The result type is always `bool`.
/// This function analyzes both operands, checks for problematic types in a boolean context,
/// determines if the result can be statically known, and sets up data flow.
fn analyze_logical_xor_operation<'a>(
    binary: &Binary,
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    binary.lhs.analyze(context, block_context, artifacts)?;
    binary.rhs.analyze(context, block_context, artifacts)?;

    let fallback_type = Rc::new(get_mixed_any());
    let lhs_type = artifacts.get_rc_expression_type(&binary.lhs).unwrap_or(&fallback_type);
    let rhs_type = artifacts.get_rc_expression_type(&binary.rhs).unwrap_or(&fallback_type);

    check_logical_operand(context, &binary.lhs, lhs_type, "Left", "xor")?;
    check_logical_operand(context, &binary.rhs, rhs_type, "Right", "xor")?;

    let result_type = if lhs_type.is_always_truthy() && rhs_type.is_always_truthy() {
        if !block_context.inside_loop_expressions {
            report_redundant_logical_operation(context, binary, "always true", "always true", "`false`");
        }

        get_false()
    } else if lhs_type.is_always_truthy() && rhs_type.is_always_falsy() {
        if !block_context.inside_loop_expressions {
            report_redundant_logical_operation(context, binary, "always true", "always false", "`true`");
        }

        get_true()
    } else if lhs_type.is_always_falsy() && rhs_type.is_always_truthy() {
        if !block_context.inside_loop_expressions {
            report_redundant_logical_operation(context, binary, "always false", "always true", "`true`");
        }

        get_true()
    } else if lhs_type.is_always_falsy() && rhs_type.is_always_falsy() {
        if !block_context.inside_loop_expressions {
            report_redundant_logical_operation(context, binary, "always false", "always false", "`false`");
        }

        get_false()
    } else {
        get_bool()
    };

    add_decision_dataflow(artifacts, &binary.lhs, Some(&binary.rhs), binary.span(), result_type);

    Ok(())
}

/// Checks a single operand of a logical operation (like AND, OR, XOR) for problematic types.
/// Reports errors for `mixed` and warnings for types that PHP coerces to boolean
/// (e.g., `null`, `array`, `resource`, `object`).
fn check_logical_operand(
    context: &mut Context<'_>,
    operand: &Expression,
    operand_type: &TUnion,
    side: &'static str,
    operator_name: &'static str,
) -> Result<bool, AnalysisError> {
    let mut critical_error_found = false;

    if operand_type.is_mixed() {
        context.buffer.report(
            TypingIssueKind::MixedOperand,
            Issue::error(format!("{side} operand in `{operator_name}` operation has `mixed` type."))
                .with_annotation(Annotation::primary(operand.span()).with_message("This has type `mixed`"))
                .with_note(format!(
                    "Using `mixed` in a boolean context like `{operator_name}` is unsafe as its truthiness is unknown."
                ))
                .with_help("Ensure this operand has a known type or explicitly cast to `bool`."),
        );

        critical_error_found = true;
    } else if operand_type.is_null() {
        context.buffer.report(
            TypingIssueKind::NullOperand,
            Issue::warning(format!(
                "{side} operand in `{operator_name}` operation is `null`, which coerces to `false`."
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This is `null` (coerces to `false`)"))
            .with_help("Explicitly check for `null` or cast to `bool` if this coercion is not intended."),
        );
    } else if operand_type.is_array() {
        context.buffer.report(
            TypingIssueKind::InvalidOperand,
            Issue::warning(format!("{side} operand in `{operator_name}` operation is an `array`."))
                .with_annotation(Annotation::primary(operand.span()).with_message("This is an `array`"))
                .with_note(
                    "Arrays coerce to `false` if empty, `true` if non-empty. This implicit conversion can be unclear.",
                )
                .with_help("Consider using `empty()` or `count()` for explicit checks, or cast to `bool`."),
        );
    } else if operand_type.is_objecty() {
        context.buffer.report(
            TypingIssueKind::InvalidOperand,
            Issue::warning(format!("{side} operand in `{operator_name}` operation is an `object`."))
                .with_annotation(Annotation::primary(operand.span()).with_message("This is an `object`"))
                .with_note(
                    "Objects generally coerce to `true` in boolean contexts. Ensure this is the intended behavior.",
                )
                .with_help("If specific truthiness is required, implement a method on the object or cast explicitly."),
        );
    } else if operand_type.is_resource() {
        context.buffer.report(
            TypingIssueKind::InvalidOperand,
            Issue::warning(format!("{side} operand in `{operator_name}` operation is a `resource`."))
                .with_annotation(Annotation::primary(operand.span()).with_message("This is a `resource`"))
                .with_note("Resources generally coerce to `true`. This implicit conversion can be unclear.")
                .with_help("Explicitly check the state of the resource or cast to `bool` if necessary."),
        );
    }

    Ok(critical_error_found)
}

/// Helper to report redundant logical operation issues.
fn report_redundant_logical_operation(
    context: &mut Context<'_>,
    binary: &Binary,
    lhs_description: &str,
    rhs_description: &str,
    result_value_str: &str,
) {
    context.buffer.report(
        TypingIssueKind::RedundantLogicalOperation,
        Issue::help(format!(
            "Redundant `{}` operation: left operand is {} and right operand is {}.",
            binary.operator.as_str(context.interner),
            lhs_description,
            rhs_description
        ))
        .with_annotation(
            Annotation::primary(binary.lhs.span()).with_message(format!("Left operand is {lhs_description}")),
        )
        .with_annotation(
            Annotation::secondary(binary.rhs.span()).with_message(format!("Right operand is {rhs_description}")),
        )
        .with_note(format!(
            "The `{}` operator will always return {} in this case.",
            binary.operator.as_str(context.interner),
            result_value_str
        ))
        .with_help(format!(
            "Consider simplifying or removing this logical expression as it always evaluates to {result_value_str}."
        )),
    );
}

/// Analyzes the spaceship operator (`LHS <=> RHS`).
///
/// The spaceship operator always returns an integer (`-1`, `0`, or `1`).
/// This function analyzes both operands, sets the result type to `int`,
/// reports warnings for potentially problematic comparisons (e.g., array with int),
/// and errors for invalid comparisons (e.g., involving `mixed`).
/// Data flow is established from both operands.
fn analyze_spaceship_operation<'a>(
    binary: &Binary,
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    binary.lhs.analyze(context, block_context, artifacts)?;
    binary.rhs.analyze(context, block_context, artifacts)?;

    let fallback_type = Rc::new(get_mixed_any());
    let lhs_type = artifacts.get_rc_expression_type(&binary.lhs).unwrap_or(&fallback_type);
    let rhs_type = artifacts.get_rc_expression_type(&binary.rhs).unwrap_or(&fallback_type);

    check_spaceship_operand(context, &binary.lhs, lhs_type, "Left")?;
    check_spaceship_operand(context, &binary.rhs, rhs_type, "Right")?;

    let lhs_is_array = lhs_type.is_array();
    let rhs_is_array = rhs_type.is_array();

    if lhs_is_array && !rhs_is_array && !rhs_type.is_null() {
        context.buffer.report(
            TypingIssueKind::InvalidOperand,
            Issue::error(format!(
                "Comparing an `array` with a non-array type `{}` using `<=>`.",
                rhs_type.get_id(Some(context.interner))
            ))
            .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is an array"))
            .with_annotation(Annotation::secondary(binary.rhs.span()).with_message(format!("This has type `{}`", rhs_type.get_id(Some(context.interner)))))
            .with_note("PHP compares arrays as greater than other types (except other arrays and null). This might not be the intended comparison.")
            .with_help("Ensure both operands are of comparable types or explicitly cast/convert them before comparison."),
        );
    } else if !lhs_is_array && rhs_is_array && !lhs_type.is_null() {
        context.buffer.report(
            TypingIssueKind::InvalidOperand,
            Issue::error(format!(
                "Comparing a non-array type `{}` with an `array` using `<=>`.",
                lhs_type.get_id(Some(context.interner))
            ))
            .with_annotation(Annotation::primary(binary.lhs.span()).with_message(format!("This has type `{}`", lhs_type.get_id(Some(context.interner)))))
            .with_annotation(Annotation::secondary(binary.rhs.span()).with_message("This is an array"))
            .with_note("PHP compares arrays as greater than other types (except other arrays and null). This might not be the intended comparison.")
            .with_help("Ensure both operands are of comparable types or explicitly cast/convert them before comparison."),
        );
    }

    let result_type = if !block_context.inside_loop_expressions
        && is_always_greater_than(lhs_type, rhs_type, context.codebase, context.interner)
    {
        context.buffer.report(
            TypingIssueKind::RedundantTypeComparison,
            Issue::help("Redundant spaceship comparison: left-hand side is always greater than right-hand side.")
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is always greater"))
                .with_annotation(Annotation::secondary(binary.rhs.span()).with_message("This is always less"))
                .with_note("The spaceship operator `<=>` will always return `1` in this case.")
                .with_help("Consider removing this comparison. It will always evaluate to `1`."),
        );

        TUnion::new(vec![TAtomic::Scalar(TScalar::literal_int(1))])
    } else if !block_context.inside_loop_expressions
        && is_always_identical_to(lhs_type, rhs_type, context.codebase, context.interner)
    {
        context.buffer.report(
            TypingIssueKind::RedundantTypeComparison,
            Issue::help("Redundant spaceship comparison: left-hand side is always equal to right-hand side.")
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is always equal"))
                .with_annotation(Annotation::secondary(binary.rhs.span()).with_message("This is always equal"))
                .with_note("The spaceship operator `<=>` will always return `0` in this case.")
                .with_help("Consider removing this comparison. It will always evaluate to `0`."),
        );

        TUnion::new(vec![TAtomic::Scalar(TScalar::literal_int(0))])
    } else if !block_context.inside_loop_expressions
        && is_always_less_than(lhs_type, rhs_type, context.codebase, context.interner)
    {
        context.buffer.report(
            TypingIssueKind::RedundantTypeComparison,
            Issue::help("Redundant spaceship comparison: left-hand side is always less than right-hand side.")
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is always less"))
                .with_annotation(Annotation::secondary(binary.rhs.span()).with_message("This is always greater"))
                .with_note("The spaceship operator `<=>` will always return `-1` in this case.")
                .with_help("Consider removing this comparison. It will always evaluate to `-1`."),
        );

        TUnion::new(vec![TAtomic::Scalar(TScalar::literal_int(-1))])
    } else {
        TUnion::new(vec![
            TAtomic::Scalar(TScalar::literal_int(1)),
            TAtomic::Scalar(TScalar::literal_int(0)),
            TAtomic::Scalar(TScalar::literal_int(-1)),
        ])
    };

    add_decision_dataflow(artifacts, &binary.lhs, Some(&binary.rhs), binary.span(), result_type);

    Ok(())
}

fn check_spaceship_operand(
    context: &mut Context<'_>,
    operand: &Expression,
    operand_type: &TUnion,
    side: &'static str,
) -> Result<(), AnalysisError> {
    if operand_type.is_null() {
        context.buffer.report(
             TypingIssueKind::NullOperand,
             Issue::error(format!(
                 "{side} operand in spaceship comparison (`<=>`) is `null`."
             ))
             .with_annotation(Annotation::primary(operand.span()).with_message("This is `null`"))
             .with_note("PHP compares `null` with other types according to specific rules (e.g., `null == 0` is true, `null < 1` is true).")
             .with_help("Ensure this comparison with `null` is intended, or provide a non-null operand."),
         );
    } else if operand_type.is_nullable() && !operand_type.is_mixed() {
        context.buffer.report(
            TypingIssueKind::PossiblyNullOperand,
            Issue::warning(format!(
                "{side} operand in spaceship comparison (`<=>`) might be `null` (type `{}`).",
                operand_type.get_id(Some(context.interner))
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This might be `null`"))
            .with_note("If this operand is `null` at runtime, PHP's specific comparison rules for `null` will apply.")
            .with_help("Ensure this operand is non-null or that comparison with `null` is intended."),
        );
    } else if operand_type.is_mixed() {
        context.buffer.report(
            TypingIssueKind::MixedOperand,
            Issue::error(format!("{side} operand in spaceship comparison (`<=>`) has `mixed` type."))
                .with_annotation(Annotation::primary(operand.span()).with_message("This has type `mixed`"))
                .with_note("The result of comparing `mixed` types with `<=>` is unpredictable.")
                .with_help("Ensure this operand has a known, comparable type before using the spaceship operator."),
        );
    } else if operand_type.is_false() {
        context.buffer.report(
            TypingIssueKind::FalseOperand,
            Issue::error(format!(
                "{side} operand in spaceship comparison (`<=>`) is `false`."
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This is `false`"))
            .with_note("PHP compares `false` with other types according to specific rules (e.g., `false == 0` is true, `false < 1` is true).")
            .with_help("Ensure this comparison with `false` is intended, or provide a non-false operand."),
        );
    } else if operand_type.is_falsable() && !operand_type.ignore_falsable_issues {
        context.buffer.report(
            TypingIssueKind::PossiblyFalseOperand,
            Issue::warning(format!(
                "{side} operand in spaceship comparison (`<=>`) might be `false` (type `{}`).",
                operand_type.get_id(Some(context.interner))
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This might be `false`"))
            .with_note("If this operand is `false` at runtime, PHP's specific comparison rules for `false` will apply.")
            .with_help("Ensure this operand is non-false or that comparison with `false` is intended."),
        );
    }

    Ok(())
}

/// Analyzes the null coalescing operator (`??`).
///
/// The result type is determined as follows:
/// - If the left-hand side (LHS) is definitely `null`, the result type is the type of the right-hand side (RHS).
///   A hint is issued about the LHS always being `null`.
/// - If the LHS is definitely not `null`, the result type is the type of the LHS. The RHS is still analyzed
///   for potential errors but does not contribute to the result type. A hint is issued about the RHS being redundant.
/// - If the LHS is nullable (can be `null` or other types), the result type is the union of the
///   non-null parts of the LHS and the type of the RHS.
/// - If the LHS type is unknown (`mixed`), the result type is `mixed`.
///
/// Data flow is established from the operand(s) that contribute to the result.
fn analyze_null_coalesce_operation<'a>(
    binary: &Binary,
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    let was_inside_isset = block_context.inside_isset;
    let was_inside_coalescing = block_context.inside_coalescing;
    block_context.inside_isset = true;
    block_context.inside_coalescing = true;
    binary.lhs.analyze(context, block_context, artifacts)?;
    block_context.inside_isset = was_inside_isset;
    block_context.inside_coalescing = was_inside_coalescing;

    let lhs_type_option = artifacts.get_rc_expression_type(&binary.lhs);

    let Some(lhs_type) = lhs_type_option else {
        binary.rhs.analyze(context, block_context, artifacts)?;

        artifacts.set_expression_type(binary, get_mixed_any());

        return Ok(());
    };

    let result_type: TUnion;
    let mut decision_node_parents = Vec::new();

    if lhs_type.is_null() {
        context.buffer.report(
            TypingIssueKind::RedundantNullCoalesce,
            Issue::help("Redundant null coalesce: left-hand side is always `null`.")
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is always `null`"))
                .with_annotation(
                    Annotation::secondary(binary.rhs.span())
                        .with_message("This right-hand side will always be evaluated"),
                )
                .with_note("The right-hand side of `??` will always be evaluated.")
                .with_help("Consider directly using the right-hand side expression."),
        );

        binary.rhs.analyze(context, block_context, artifacts)?;
        result_type = artifacts.get_expression_type(&binary.rhs).cloned().unwrap_or_else(get_mixed_any); // Fallback if RHS analysis fails

        if let Some(rhs_parents) = artifacts.get_expression_type(&binary.rhs).map(|t| &t.parent_nodes) {
            decision_node_parents.extend(rhs_parents.iter().cloned());
        }
    } else if !lhs_type.has_nullish() && !lhs_type.possibly_undefined && !lhs_type.possibly_undefined_from_try {
        context.buffer.report(
            TypingIssueKind::RedundantNullCoalesce,
            Issue::help(
                "Redundant null coalesce: left-hand side can never be `null` or undefined."
            )
            .with_annotation(Annotation::primary(binary.lhs.span()).with_message(format!(
                "This expression (type `{}`) is never `null` or undefined",
                lhs_type.get_id(Some(context.interner))
            )))
            .with_annotation(
                Annotation::secondary(binary.rhs.span()).with_message("This right-hand side will never be evaluated"),
            )
            .with_note(
                "The null coalesce operator `??` only evaluates the right-hand side if the left-hand side is `null` or not set.",
            )
            .with_help("Consider removing the `??` operator and the right-hand side expression."),
        );

        result_type = (**lhs_type).clone();
        binary.rhs.analyze(context, block_context, artifacts)?;

        decision_node_parents.extend(result_type.parent_nodes.iter().cloned());
    } else {
        let non_null_lhs_type = lhs_type.to_non_nullable();
        decision_node_parents.extend(lhs_type.parent_nodes.iter().cloned());

        binary.rhs.analyze(context, block_context, artifacts)?;
        let rhs_type = artifacts
            .get_expression_type(&binary.rhs)
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(get_mixed_any()));

        result_type = combine_union_types(&non_null_lhs_type, &rhs_type, context.codebase, context.interner, false);

        if let Some(rhs_parents) = artifacts.get_expression_type(&binary.rhs).map(|t| &t.parent_nodes) {
            decision_node_parents.extend(rhs_parents.iter().cloned());
        }
    }

    add_decision_dataflow(artifacts, &binary.lhs, Some(&binary.rhs), binary.span(), result_type);

    Ok(())
}

/// Analyzes the Elvis operator (`?:`).
///
/// The result type is determined based on the "falsiness" of the left-hand side (LHS):
/// - If LHS is always falsy (e.g., `false`, `null`, `0`, `""`), the result is the type of the RHS.
///   A hint is issued about the LHS always being falsy.
/// - If LHS is never falsy (e.g., `true`, non-empty string, non-zero number, object),
///   the result is the type of the LHS. The RHS is still analyzed for side effects.
///   A hint is issued about the RHS being redundant.
/// - If LHS can be falsy (e.g., `bool`, `int`, `string`), the result is a union of the
///   "truthy" parts of the LHS type and the RHS type.
/// - If LHS is `mixed`, the result is a union of `mixed` and the RHS type.
///
/// Data flow is established from the operand(s) that contribute to the result.
fn analyze_elvis_operation<'a>(
    binary: &Binary,
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    binary.lhs.analyze(context, block_context, artifacts)?;
    let lhs_type_option = artifacts.get_rc_expression_type(&binary.lhs).cloned();

    let Some(lhs_type) = lhs_type_option else {
        binary.rhs.analyze(context, block_context, artifacts)?;
        artifacts.set_expression_type(binary, get_mixed_any());
        return Ok(());
    };

    let result_type: TUnion;
    let mut decision_node_parents = Vec::new();

    if lhs_type.is_always_falsy() {
        context.buffer.report(
            TypingIssueKind::RedundantElvis,
            Issue::help("Redundant Elvis operator: left-hand side is always falsy.")
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is always falsy"))
                .with_annotation(
                    Annotation::secondary(binary.rhs.span())
                        .with_message("This right-hand side will always be evaluated"),
                )
                .with_note("The Elvis operator `?:` evaluates the right-hand side if the left-hand side is falsy.")
                .with_help("Consider directly using the right-hand side expression."),
        );

        binary.rhs.analyze(context, block_context, artifacts)?;
        result_type = artifacts.get_expression_type(&binary.rhs).cloned().unwrap_or_else(get_mixed_any);

        if let Some(rhs_parents) = artifacts.get_expression_type(&binary.rhs).map(|t| &t.parent_nodes) {
            decision_node_parents.extend(rhs_parents.iter().cloned());
        }
    } else if lhs_type.is_always_truthy() {
        context.buffer.report(
            TypingIssueKind::RedundantElvis,
            Issue::help("Redundant Elvis operator: left-hand side is always truthy.")
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message(format!(
                    "This expression (type `{}`) is always truthy",
                    lhs_type.get_id(Some(context.interner))
                )))
                .with_annotation(
                    Annotation::secondary(binary.rhs.span())
                        .with_message("This right-hand side will never be evaluated"),
                )
                .with_note("The Elvis operator `?:` only evaluates the right-hand side if the left-hand side is falsy.")
                .with_help("Consider removing the `?:` operator and the right-hand side expression."),
        );

        result_type = (*lhs_type).clone();
        binary.rhs.analyze(context, block_context, artifacts)?;

        decision_node_parents.extend(lhs_type.parent_nodes.iter().cloned());
    } else {
        binary.rhs.analyze(context, block_context, artifacts)?;
        let rhs_type = artifacts.get_expression_type(&binary.rhs).cloned().unwrap_or_else(get_mixed_any);

        let truthy_lhs_type = lhs_type.to_truthy();

        result_type = combine_union_types(&truthy_lhs_type, &rhs_type, context.codebase, context.interner, false);

        decision_node_parents.extend(lhs_type.parent_nodes.iter().cloned());
        if let Some(rhs_parents) = artifacts.get_expression_type(&binary.rhs).map(|t| &t.parent_nodes) {
            decision_node_parents.extend(rhs_parents.iter().cloned());
        }
    }

    add_decision_dataflow(artifacts, &binary.lhs, Some(&binary.rhs), binary.span(), result_type);

    Ok(())
}

#[inline]
fn analyze_arithmetic_operation<'a>(
    binary: &Binary,
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    let was_inside_general_use = block_context.inside_general_use;
    block_context.inside_general_use = true;
    binary.lhs.analyze(context, block_context, artifacts)?;
    binary.rhs.analyze(context, block_context, artifacts)?;
    block_context.inside_general_use = was_inside_general_use;

    let fallback = Rc::new(get_mixed_any());
    let left_type = artifacts.get_rc_expression_type(&binary.lhs).cloned().unwrap_or_else(|| fallback.clone());
    let right_type = artifacts.get_rc_expression_type(&binary.rhs).cloned().unwrap_or_else(|| fallback.clone());

    let mut final_result_type: Option<TUnion> = None;

    if left_type.is_null() {
        context.buffer.report(
            TypingIssueKind::NullOperand,
            Issue::error("Left operand in arithmetic operation cannot be `null`.")
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is `null`."))
                .with_note("Performing arithmetic operations on `null` typically results in `0`.")
                .with_help("Ensure the left operand is a number (int/float) or a type that can be cast to a number."),
        );

        // In Psalm, null operand often leads to mixed result or halts analysis for this path.
        // Let's set result to mixed and return, similar to Psalm's behavior.
        final_result_type = Some(get_mixed_any());
    } else if left_type.is_nullable() && !left_type.ignore_nullable_issues {
        context.buffer.report(
            TypingIssueKind::PossiblyNullOperand,
            Issue::warning(format!(
                "Left operand in arithmetic operation might be `null` (type `{}`).",
                left_type.get_id(Some(context.interner))
            ))
            .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This might be `null`."))
            .with_note("Performing arithmetic operations on `null` typically results in `0`.")
            .with_help(
                "Ensure the left operand is non-null before the operation, potentially using checks or assertions.",
            ),
        );
    }

    if right_type.is_null() {
        context.buffer.report(
            TypingIssueKind::NullOperand,
            Issue::error("Right operand in arithmetic operation cannot be `null`.")
                .with_annotation(Annotation::primary(binary.rhs.span()).with_message("This is `null`."))
                .with_note("Performing arithmetic operations on `null` typically results in `0`.")
                .with_help("Ensure the right operand is a number (int/float) or a type that can be cast to a number."),
        );

        final_result_type = Some(get_mixed_any());
    } else if right_type.is_nullable() && !right_type.ignore_nullable_issues {
        context.buffer.report(
            TypingIssueKind::PossiblyNullOperand,
            Issue::warning(format!(
                "Right operand in arithmetic operation might be `null` (type `{}`).",
                right_type.get_id(Some(context.interner))
            ))
            .with_annotation(Annotation::primary(binary.rhs.span()).with_message("This might be `null`"))
            .with_note("Performing arithmetic operations on `null` typically results in `0`.")
            .with_help(
                "Ensure the right operand is non-null before the operation, potentially using checks or assertions.",
            ),
        );
    }

    if is_arithmetic_compatiable_generic(context, &left_type, &right_type) {
        final_result_type = Some(left_type.as_ref().clone());
    } else if is_arithmetic_compatiable_generic(context, &right_type, &left_type) {
        final_result_type = Some(right_type.as_ref().clone());
    }

    if let Some(final_result_type) = final_result_type {
        assign_arithmetic_type(artifacts, final_result_type, binary);

        return Ok(());
    }

    if left_type.is_false() {
        context.buffer.report(
            TypingIssueKind::FalseOperand,
            Issue::warning(
                "Left operand in arithmetic operation is `false`.",
            )
            .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is `false`"))
            .with_note("Performing arithmetic operations on `false` typically results in `0`.")
            .with_help(
                "Ensure the left operand is a number (int/float). Using `false` directly in arithmetic is discouraged.",
            ),
        );
        // We'll treat it as 0 in the loop below, but the warning is issued.
        // If *only* false, Psalm might bail; let's continue for now
    } else if left_type.is_falsable() && !left_type.ignore_falsable_issues {
        context.buffer.report(
            TypingIssueKind::PossiblyFalseOperand,
            Issue::warning(format!(
                "Left operand in arithmetic operation might be `false` (type `{}`).",
                left_type.get_id(Some(context.interner))
            ))
            .with_annotation(
                Annotation::primary(binary.lhs.span())
                    .with_message("This might be `false`.")
            )
            .with_note(
                "Performing arithmetic operations on `false` typically results in `0`."
            )
            .with_help(
                "Ensure the left operand is non-falsy before the operation, or explicitly cast if coercion is intended."
            ),
        );
    }

    if right_type.is_false() {
        context.buffer.report(
            TypingIssueKind::FalseOperand,
            Issue::warning(
                "Right operand in arithmetic operation is `false`."
            )
            .with_annotation(
                Annotation::primary(binary.rhs.span())
                    .with_message("This is `false`.")
            )
            .with_note(
                "Performing arithmetic operations on `false` typically results in `0` after a warning/notice."
            )
            .with_help(
                "Ensure the right operand is a number (int/float). Using `false` directly in arithmetic is discouraged."
            ),
        );
    } else if right_type.is_falsable() && !right_type.ignore_falsable_issues {
        context.buffer.report(
            TypingIssueKind::PossiblyFalseOperand,
            Issue::warning(format!(
                "Right operand in arithmetic operation might be `false` (type `{}`).",
                right_type.get_id(Some(context.interner))
            ))
            .with_annotation(
                Annotation::primary(binary.rhs.span())
                    .with_message("This might be `false`.")
            )
            .with_note(
                "Performing arithmetic operations on `false` typically results in `0`."
            )
            .with_help(
                "Ensure the right operand is non-falsy before the operation, or explicitly cast if coercion is intended."
            ),
        );
    }

    let mut result_atomic_types: Vec<TAtomic> = Vec::new();
    let mut invalid_left_messages: Vec<(String, Span)> = Vec::new();
    let mut invalid_right_messages: Vec<(String, Span)> = Vec::new();
    let mut has_valid_left_operand = false;
    let mut has_valid_right_operand = false;

    let left_atomic_types = left_type
        .types
        .iter()
        .cloned()
        .flat_map(|atomic| {
            if let TAtomic::GenericParameter(parameter) = atomic { parameter.constraint.types } else { vec![atomic] }
        })
        .collect::<VecDeque<_>>();

    let right_atomic_types = right_type
        .types
        .iter()
        .cloned()
        .flat_map(|atomic| {
            if let TAtomic::GenericParameter(parameter) = atomic { parameter.constraint.types } else { vec![atomic] }
        })
        .collect::<Vec<_>>();

    for mut left_atomic in left_atomic_types {
        left_atomic = match left_atomic {
            TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_false() => TAtomic::Scalar(TScalar::literal_int(0)),
            TAtomic::Null => continue,
            atomic => atomic,
        };

        for right_atomic in &right_atomic_types {
            let right_atomic = match right_atomic {
                TAtomic::Scalar(TScalar::Bool(bool)) if bool.is_false() => TAtomic::Scalar(TScalar::literal_int(0)),
                TAtomic::Null => continue,
                atomic => atomic.clone(),
            };

            let mut pair_result_atomics: Vec<TAtomic> = Vec::new();
            let mut invalid_pair = false;

            if left_atomic.is_mixed() {
                context.buffer.report(
                    TypingIssueKind::MixedOperand,
                    Issue::error(
                        "Left operand in binary operation has type `mixed`."
                    )
                    .with_annotation(
                        Annotation::primary(binary.lhs.span())
                            .with_message("Operand is `mixed`.")
                    )
                    .with_note(
                        "Performing operations on `mixed` is unsafe as the actual runtime type is unknown."
                    )
                    .with_help(
                        "Ensure the left operand has a known type (e.g., `int`, `float`, `string`) using type hints, assertions, or checks."
                    ),
                );

                pair_result_atomics.push(TAtomic::Mixed(TMixed::vanilla()));
                if !right_atomic.is_mixed() {
                    has_valid_right_operand = true;
                }
            }

            if right_atomic.is_mixed() {
                context.buffer.report(
                    TypingIssueKind::MixedOperand,
                    Issue::error(
                        "Right operand in binary operation has type `mixed`."
                    )
                    .with_annotation(
                        Annotation::primary(binary.rhs.span())
                            .with_message("Operand is `mixed`.")
                    )
                    .with_note(
                        "Performing operations on `mixed` is unsafe as the actual runtime type is unknown."
                    )
                    .with_help(
                        "Ensure the right operand has a known type (e.g., `int`, `float`, `string`) using type hints, assertions, or checks."
                    ),
                );

                if !pair_result_atomics.iter().any(|a| a.is_mixed()) {
                    pair_result_atomics.push(TAtomic::Mixed(TMixed::vanilla()));
                }
                if !left_atomic.is_mixed() {
                    has_valid_left_operand = true;
                }
            }

            if left_atomic.is_mixed() || right_atomic.is_mixed() {
                result_atomic_types.extend(pair_result_atomics);
                continue;
            }

            if matches!(binary.operator, BinaryOperator::Addition(_))
                && (left_atomic.is_array() || right_atomic.is_array())
            {
                if left_atomic.is_array() && right_atomic.is_array() {
                    // TODO(azjezz): Implement array combination logic similar to Psalm
                    // This involves merging keys for KeyedArray, combining types for Array
                    // For now, let's assume a generic array result. Refine this.

                    let array_key_type = match (left_atomic.get_array_key_type(), right_atomic.get_array_key_type()) {
                        (Some(left_key), Some(right_key)) => {
                            combine_union_types(&left_key, &right_key, context.codebase, context.interner, false)
                        }
                        _ => get_arraykey(),
                    };

                    let array_value_type =
                        match (left_atomic.get_array_value_type(), right_atomic.get_array_value_type()) {
                            (Some(left_value), Some(right_value)) => combine_union_types(
                                &left_value,
                                &right_value,
                                context.codebase,
                                context.interner,
                                false,
                            ),
                            _ => get_mixed_any(),
                        };

                    let mut keyed_array = TKeyedArray::new();
                    keyed_array.parameters = Some((Box::new(array_key_type), Box::new(array_value_type)));
                    keyed_array.non_empty = left_atomic.is_non_empty_array() || right_atomic.is_non_empty_array();

                    pair_result_atomics.push(TAtomic::Array(TArray::Keyed(keyed_array)));

                    has_valid_left_operand = true;
                    has_valid_right_operand = true;
                } else {
                    if left_atomic.is_array() {
                        invalid_right_messages.push((
                            format!(
                                "Cannot add array to non-array type {}",
                                right_atomic.get_id(Some(context.interner))
                            ),
                            binary.rhs.span(),
                        ));
                        has_valid_left_operand = true;
                    } else {
                        invalid_left_messages.push((
                            format!(
                                "Cannot add {} to non-array type array",
                                left_atomic.get_id(Some(context.interner))
                            ),
                            binary.lhs.span(),
                        ));
                        has_valid_right_operand = true;
                    }
                    invalid_pair = true;
                }
            } else if left_atomic.is_numeric() && right_atomic.is_numeric() {
                let numeric_results =
                    determine_numeric_result(&binary.operator, &left_atomic, &right_atomic, block_context.inside_loop);

                if numeric_results.iter().any(|a| matches!(a, TAtomic::Never)) {
                    invalid_pair = true;
                    if matches!(binary.operator, BinaryOperator::Division(_) | BinaryOperator::Modulo(_)) {
                        let right_is_zero = matches!(right_atomic.get_literal_int_value(), Some(0));

                        if right_is_zero {
                            invalid_right_messages.push(("Division or modulo by zero".to_string(), binary.rhs.span()));
                            pair_result_atomics.push(TAtomic::Never);
                        } else {
                            pair_result_atomics.extend(numeric_results);
                        }
                    } else {
                        pair_result_atomics.extend(numeric_results);
                    }
                } else {
                    pair_result_atomics.extend(numeric_results);
                    has_valid_left_operand = true;
                    has_valid_right_operand = true;
                }
            } else if left_atomic.is_numeric() {
                invalid_right_messages.push((
                    format!(
                        "Cannot perform arithmetic operation with non-numeric type {}",
                        right_atomic.get_id(Some(context.interner))
                    ),
                    binary.rhs.span(),
                ));
                has_valid_left_operand = true;
                invalid_pair = true;
            } else if right_atomic.is_numeric() {
                invalid_left_messages.push((
                    format!(
                        "Cannot perform arithmetic operation with non-numeric type {}",
                        left_atomic.get_id(Some(context.interner))
                    ),
                    binary.lhs.span(),
                ));
                has_valid_right_operand = true;
                invalid_pair = true;
            } else {
                invalid_left_messages.push((
                    format!(
                        "Cannot perform arithmetic operation on type {}",
                        left_atomic.get_id(Some(context.interner))
                    ),
                    binary.lhs.span(),
                ));

                invalid_right_messages.push((
                    format!(
                        "Cannot perform arithmetic operation on type {}",
                        right_atomic.get_id(Some(context.interner))
                    ),
                    binary.rhs.span(),
                ));

                invalid_pair = true;
            }

            if !invalid_pair {
                result_atomic_types.extend(pair_result_atomics);
            }
        }
    }

    if !invalid_left_messages.is_empty() {
        let issue_kind = if has_valid_left_operand {
            TypingIssueKind::PossiblyInvalidOperand
        } else {
            TypingIssueKind::InvalidOperand
        };

        let mut issue = if has_valid_left_operand {
            Issue::warning("Possibly invalid type for left operand.".to_string())
        } else {
            Issue::error("Invalid type for left operand.".to_string())
        };

        for (msg, span) in invalid_left_messages {
            issue = issue.with_annotation(Annotation::secondary(span).with_message(msg));
        }

        context.buffer.report(
            issue_kind,
                issue
                    .with_note(
                        "The type(s) of the left operand are not compatible with this binary operation."
                    )
                    .with_help(
                        "Ensure the left operand has a type suitable for this operation (e.g., number for arithmetic, string for concatenation)."
                    )

        );
    }

    if !invalid_right_messages.is_empty() {
        let issue_kind = if has_valid_right_operand {
            TypingIssueKind::PossiblyInvalidOperand
        } else {
            TypingIssueKind::InvalidOperand
        };

        let mut issue = if has_valid_right_operand {
            Issue::warning("Possibly invalid type for right operand.".to_string())
        } else {
            Issue::error("Invalid type for right operand.".to_string())
        };

        for (msg, span) in invalid_right_messages {
            issue = issue.with_annotation(Annotation::secondary(span).with_message(msg));
        }

        context.buffer.report(
            issue_kind,

                issue
                    .with_note(
                        "The type(s) of the right operand are not compatible with this binary operation."
                    )
                    .with_help(
                        "Ensure the right operand has a type suitable for this operation (e.g., number for arithmetic, string for concatenation)."
                    )
        );
    }

    let final_type = if !result_atomic_types.is_empty() {
        let mut combined =
            TUnion::new(combiner::combine(result_atomic_types, context.codebase, context.interner, false));
        combined.parent_nodes = Vec::new();
        combined
    } else {
        // No valid pairs found, and potentially errors issued.
        // Psalm often defaults to mixed here if operands were invalid.
        // If errors were due to null/false operands handled initially, use the type set there.
        // Otherwise, default to mixed.
        get_mixed_any()
    };

    assign_arithmetic_type(artifacts, final_type, binary);

    Ok(())
}

#[inline]
fn is_arithmetic_compatiable_generic<'a>(context: &Context<'a>, union: &TUnion, other_union: &TUnion) -> bool {
    if !union.is_single() {
        return false;
    }

    let TAtomic::GenericParameter(generic_parameter) = union.get_single() else {
        return false;
    };

    for constraint_atomic in generic_parameter.constraint.types.iter() {
        for other_atomic in other_union.types.iter() {
            if !atomic_comparator::is_contained_by(
                context.codebase,
                context.interner,
                other_atomic,
                constraint_atomic,
                false,
                &mut ComparisonResult::new(),
            ) {
                return false;
            }
        }
    }

    true
}

#[inline]
fn analyze_logical_and_operation<'a>(
    binary: &Binary,
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    let mut left_block_context = block_context.clone();
    let pre_referenced_var_ids = left_block_context.conditionally_referenced_variable_ids.clone();
    let pre_assigned_var_ids = left_block_context.assigned_variable_ids.clone();
    left_block_context.conditionally_referenced_variable_ids.clear();
    left_block_context.assigned_variable_ids.clear();
    left_block_context.reconciled_expression_clauses = Vec::new();

    let left_was_inside_general_use = left_block_context.inside_general_use;
    left_block_context.inside_general_use = true;
    binary.lhs.analyze(context, &mut left_block_context, artifacts)?;
    left_block_context.inside_general_use = left_was_inside_general_use;

    let lhs_type = match artifacts.get_rc_expression_type(&binary.lhs).cloned() {
        Some(lhs_type) => {
            check_logical_operand(context, &binary.lhs, &lhs_type, "Left", "&&")?;

            lhs_type
        }
        None => Rc::new(get_mixed_any()),
    };

    let left_clauses = get_formula(
        binary.lhs.span(),
        binary.lhs.span(),
        &binary.lhs,
        context.get_assertion_context_from_block(block_context),
        artifacts,
    );

    for (var_id, var_type) in &left_block_context.locals {
        if left_block_context.assigned_variable_ids.contains_key(var_id) {
            block_context.locals.insert(var_id.clone(), var_type.clone());
        }
    }

    let mut left_referenced_var_ids = left_block_context.conditionally_referenced_variable_ids.clone();
    let mut context_clauses = left_block_context.clauses.iter().map(|v| (&**v)).collect::<Vec<_>>();
    block_context.conditionally_referenced_variable_ids.extend(pre_referenced_var_ids);
    block_context.assigned_variable_ids.extend(pre_assigned_var_ids);
    context_clauses.extend(left_clauses.iter());
    if !left_block_context.reconciled_expression_clauses.is_empty() {
        let left_reconciled_clauses_hashed =
            left_block_context.reconciled_expression_clauses.iter().map(|v| &**v).collect::<HashSet<_>>();

        context_clauses.retain(|c| !left_reconciled_clauses_hashed.contains(c));
        if context_clauses.len() == 1 {
            let first = &context_clauses[0];
            if first.wedge && first.possibilities.is_empty() {
                context_clauses = Vec::new();
            }
        }
    }

    let simplified_clauses = saturate_clauses(context_clauses);
    let (left_assertions, active_left_assertions) = find_satisfying_assignments(
        simplified_clauses.as_slice(),
        Some(binary.lhs.span()),
        &mut left_referenced_var_ids,
    );

    let mut changed_var_ids = HashSet::default();
    let mut right_block_context;
    if !left_assertions.is_empty() {
        right_block_context = block_context.clone();

        let mut reconcilation_context =
            ReconcilationContext::new(context.interner, context.codebase, &mut context.buffer, artifacts);

        reconciler::reconcile_keyed_types(
            &mut reconcilation_context,
            &left_assertions,
            active_left_assertions,
            &mut right_block_context,
            &mut changed_var_ids,
            &left_referenced_var_ids,
            &binary.rhs.span(),
            true,
            !block_context.inside_negation,
        );
    } else {
        right_block_context = left_block_context.clone()
    }

    let partitioned_clauses = BlockContext::remove_reconciled_clause_refs(
        &{
            let mut c = left_block_context.clauses.clone();
            c.extend(left_clauses.into_iter().map(Rc::new));
            c
        },
        &changed_var_ids,
    );
    right_block_context.clauses = partitioned_clauses.0;

    let result_type: TUnion;
    if lhs_type.is_always_falsy() {
        report_redundant_logical_operation(context, binary, "always falsy", "not evaluated", "`false`");

        result_type = get_false();
        let mut dead_rhs_context = right_block_context.clone();
        dead_rhs_context.has_returned = true;
        binary.rhs.analyze(context, &mut dead_rhs_context, artifacts)?;
    } else {
        binary.rhs.analyze(context, &mut right_block_context, artifacts)?;
        let rhs_type = match artifacts.get_rc_expression_type(&binary.rhs).cloned() {
            Some(rhs_type) => {
                check_logical_operand(context, &binary.rhs, &rhs_type, "Right", "&&")?;
                rhs_type
            }
            None => Rc::new(get_mixed_any()),
        };

        let left_is_truthy = lhs_type.is_always_truthy();
        if left_is_truthy {
            report_redundant_logical_operation(
                context,
                binary,
                "always truthy",
                "evaluated",
                "the boolean value of the right-hand side",
            );
        }

        if rhs_type.is_always_falsy() {
            report_redundant_logical_operation(context, binary, "evaluated", "always falsy", "`false`");

            result_type = get_false();
        } else if rhs_type.is_always_truthy() {
            report_redundant_logical_operation(
                context,
                binary,
                "evaluated",
                "always truthy",
                "the boolean value of the left-hand side",
            );

            if left_is_truthy {
                result_type = get_true();
            } else {
                result_type = get_bool();
            }
        } else {
            result_type = get_bool();
        }
    }

    let mut final_type_with_flow = result_type;
    let mut decision_node_parents = Vec::new();
    decision_node_parents.extend(lhs_type.parent_nodes.iter().cloned());
    if !lhs_type.is_always_falsy()
        && let Some(rhs_parents) = artifacts.get_expression_type(&binary.rhs).map(|t| &t.parent_nodes)
    {
        decision_node_parents.extend(rhs_parents.iter().cloned());
    }

    if !decision_node_parents.is_empty() {
        let decision_node = DataFlowNode::get_for_composition(binary.span());
        artifacts.data_flow_graph.add_node(decision_node.clone());
        for parent_node in decision_node_parents {
            if artifacts.data_flow_graph.get_node(&parent_node.id).is_some() {
                artifacts.data_flow_graph.add_path(&parent_node, &decision_node, PathKind::Default);
            }
        }

        final_type_with_flow.parent_nodes = vec![decision_node];
    }

    artifacts.set_expression_type(binary, final_type_with_flow);

    block_context.conditionally_referenced_variable_ids = left_block_context.conditionally_referenced_variable_ids;
    block_context
        .conditionally_referenced_variable_ids
        .extend(right_block_context.conditionally_referenced_variable_ids);

    if block_context.inside_conditional {
        block_context.assigned_variable_ids = left_block_context.assigned_variable_ids;
        block_context.assigned_variable_ids.extend(right_block_context.assigned_variable_ids);
    }

    if let Some(if_body_context) = &block_context.if_body_context {
        let mut if_body_context_inner = if_body_context.borrow_mut();

        if !block_context.inside_negation {
            block_context.locals = right_block_context.locals;

            if_body_context_inner.locals.extend(block_context.locals.clone());
            if_body_context_inner
                .conditionally_referenced_variable_ids
                .extend(block_context.conditionally_referenced_variable_ids.clone());
            if_body_context_inner.assigned_variable_ids.extend(block_context.assigned_variable_ids.clone());
            if_body_context_inner.reconciled_expression_clauses.extend(partitioned_clauses.1);
        } else {
            block_context.locals = left_block_context.locals;
        }
    } else {
        block_context.locals = left_block_context.locals;
    }

    Ok(())
}

#[inline]
fn analyze_logical_or_operation<'a>(
    binary: &Binary,
    context: &mut Context<'a>,
    block_context: &mut BlockContext<'a>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    let mut left_block_context;
    let mut left_referenced_var_ids;
    let left_assigned_var_ids;

    if !is_logical_or_operation(&binary.lhs, 3) {
        let mut if_scope = IfScope::default();

        let (if_conditional_scope, applied_block_context) =
            conditional::analyze(context, block_context.clone(), artifacts, &mut if_scope, &binary.lhs, false)?;
        *block_context = applied_block_context;

        left_block_context = if_conditional_scope.if_body_context;
        left_referenced_var_ids = if_conditional_scope.conditionally_referenced_variable_ids;
    } else {
        let pre_referenced_var_ids = block_context.conditionally_referenced_variable_ids.clone();
        block_context.conditionally_referenced_variable_ids = HashSet::default();

        let pre_assigned_var_ids = block_context.assigned_variable_ids.clone();

        left_block_context = block_context.clone();
        left_block_context.assigned_variable_ids = HashMap::default();

        let tmp_if_body_block_context = left_block_context.if_body_context;
        left_block_context.if_body_context = None;

        binary.lhs.analyze(context, &mut left_block_context, artifacts)?;

        left_block_context.if_body_context = tmp_if_body_block_context;

        for var_id in &left_block_context.parent_conflicting_clause_variables {
            block_context.remove_variable_from_conflicting_clauses(
                context.interner,
                context.codebase,
                &mut context.buffer,
                artifacts,
                var_id,
                None,
            );
        }

        let cloned_vars = block_context.locals.clone();
        for (var_id, left_type) in &left_block_context.locals {
            if let Some(context_type) = cloned_vars.get(var_id) {
                block_context.locals.insert(
                    var_id.clone(),
                    Rc::new(combine_union_types(context_type, left_type, context.codebase, context.interner, false)),
                );
            } else if left_block_context.assigned_variable_ids.contains_key(var_id) {
                block_context.locals.insert(var_id.clone(), left_type.clone());
            }
        }

        left_referenced_var_ids = left_block_context.conditionally_referenced_variable_ids.clone();
        left_block_context.conditionally_referenced_variable_ids.extend(pre_referenced_var_ids);

        left_assigned_var_ids = left_block_context.assigned_variable_ids.clone();
        left_block_context.assigned_variable_ids.extend(pre_assigned_var_ids);

        left_referenced_var_ids.retain(|id| !left_assigned_var_ids.contains_key(id));
    }

    let lhs_type = match artifacts.get_rc_expression_type(&binary.lhs).cloned() {
        Some(lhs_type) => {
            check_logical_operand(context, &binary.lhs, &lhs_type, "Left", "||")?;

            lhs_type
        }
        None => Rc::new(get_mixed_any()),
    };

    let left_clauses = get_formula(
        binary.lhs.span(),
        binary.lhs.span(),
        &binary.lhs,
        context.get_assertion_context_from_block(block_context),
        artifacts,
    );

    let mut negated_left_clauses = negate_or_synthesize(
        left_clauses,
        &binary.lhs,
        context.get_assertion_context_from_block(block_context),
        artifacts,
    );

    if !left_block_context.reconciled_expression_clauses.is_empty() {
        let left_reconciled_clauses_hashed =
            left_block_context.reconciled_expression_clauses.iter().map(|v| &**v).collect::<HashSet<_>>();

        negated_left_clauses.retain(|c| !left_reconciled_clauses_hashed.contains(c));

        if negated_left_clauses.len() == 1 {
            let first = &negated_left_clauses[0];
            if first.wedge && first.possibilities.is_empty() {
                negated_left_clauses = Vec::new();
            }
        }
    }

    let clauses_for_right_analysis =
        saturate_clauses(block_context.clauses.iter().map(|v| &**v).chain(negated_left_clauses.iter()));

    let (negated_type_assertions, active_negated_type_assertions) = find_satisfying_assignments(
        clauses_for_right_analysis.as_slice(),
        Some(binary.lhs.span()),
        &mut left_referenced_var_ids,
    );

    let mut changed_var_ids = HashSet::default();
    let mut right_block_context = block_context.clone();

    let result_type: TUnion;

    if lhs_type.is_always_truthy() {
        report_redundant_logical_operation(context, binary, "always true", "not evaluated", "`true`");
        result_type = get_true();
        right_block_context.has_returned = true;
        binary.rhs.analyze(context, &mut right_block_context, artifacts)?;
    } else {
        if !negated_type_assertions.is_empty() {
            let mut reconcilation_context =
                ReconcilationContext::new(context.interner, context.codebase, &mut context.buffer, artifacts);

            reconciler::reconcile_keyed_types(
                &mut reconcilation_context,
                &negated_type_assertions,
                active_negated_type_assertions,
                &mut right_block_context,
                &mut changed_var_ids,
                &left_referenced_var_ids,
                &binary.lhs.span(),
                true,
                !block_context.inside_negation,
            );
        }

        right_block_context.clauses = clauses_for_right_analysis.iter().map(|v| Rc::new(v.clone())).collect();

        if !changed_var_ids.is_empty() {
            let partiioned_clauses =
                BlockContext::remove_reconciled_clause_refs(&right_block_context.clauses, &changed_var_ids);
            right_block_context.clauses = partiioned_clauses.0;
            right_block_context.reconciled_expression_clauses.extend(partiioned_clauses.1);

            let partiioned_clauses =
                BlockContext::remove_reconciled_clause_refs(&block_context.clauses, &changed_var_ids);
            block_context.clauses = partiioned_clauses.0;
            block_context.reconciled_expression_clauses.extend(partiioned_clauses.1);
        }

        let pre_referenced_var_ids = right_block_context.conditionally_referenced_variable_ids.clone();
        right_block_context.conditionally_referenced_variable_ids = HashSet::default();

        let pre_assigned_var_ids = right_block_context.assigned_variable_ids.clone();
        right_block_context.assigned_variable_ids = HashMap::default();

        let tmp_if_body_context = right_block_context.if_body_context;
        right_block_context.if_body_context = None;

        binary.rhs.analyze(context, &mut right_block_context, artifacts)?;

        right_block_context.if_body_context = tmp_if_body_context;

        let rhs_type = match artifacts.get_rc_expression_type(&binary.rhs).cloned() {
            Some(rhs_type) => {
                check_logical_operand(context, &binary.rhs, &rhs_type, "Right", "||")?;

                rhs_type
            }
            None => Rc::new(get_mixed_any()),
        };

        if lhs_type.is_always_falsy() {
            if rhs_type.is_always_falsy() {
                report_redundant_logical_operation(context, binary, "always falsy", "always falsy", "`false`");
                result_type = get_false();
            } else if rhs_type.is_always_truthy() {
                report_redundant_logical_operation(context, binary, "always falsy", "always truthy", "`true`");
                result_type = get_true();
            } else {
                report_redundant_logical_operation(
                    context,
                    binary,
                    "always false",
                    "evaluated",
                    "the boolean value of the right-hand side",
                );

                result_type = get_bool();
            }
        } else if rhs_type.is_always_falsy() {
            report_redundant_logical_operation(
                context,
                binary,
                "evaluated",
                "always falsy",
                "the boolean value of the left-hand side",
            );

            result_type = get_bool();
        } else if rhs_type.is_always_truthy() {
            report_redundant_logical_operation(context, binary, "evaluated", "always truthy", "`true`");

            result_type = get_true();
        } else {
            result_type = get_bool();
        }

        let mut right_referenced_var_ids = right_block_context.conditionally_referenced_variable_ids.clone();
        right_block_context.conditionally_referenced_variable_ids.extend(pre_referenced_var_ids);

        let right_assigned_var_ids = right_block_context.assigned_variable_ids.clone();
        right_block_context.assigned_variable_ids.extend(pre_assigned_var_ids);

        let right_clauses = get_formula(
            binary.rhs.span(),
            binary.rhs.span(),
            &binary.rhs,
            context.get_assertion_context_from_block(block_context),
            artifacts,
        );

        let mut clauses_for_right_analysis = BlockContext::remove_reconciled_clauses(
            &clauses_for_right_analysis,
            &right_assigned_var_ids.into_keys().collect::<HashSet<_>>(),
        )
        .0;

        clauses_for_right_analysis.extend(right_clauses);

        let combined_right_clauses = saturate_clauses(clauses_for_right_analysis.iter());

        let (right_type_assertions, active_right_type_assertions) = find_satisfying_assignments(
            combined_right_clauses.as_slice(),
            Some(binary.rhs.span()),
            &mut right_referenced_var_ids,
        );

        if !right_type_assertions.is_empty() {
            let mut reconcilation_context =
                ReconcilationContext::new(context.interner, context.codebase, &mut context.buffer, artifacts);

            let mut right_changed_var_ids = HashSet::default();

            reconciler::reconcile_keyed_types(
                &mut reconcilation_context,
                &right_type_assertions,
                active_right_type_assertions,
                &mut right_block_context.clone(),
                &mut right_changed_var_ids,
                &right_referenced_var_ids,
                &binary.rhs.span(),
                true,
                block_context.inside_negation,
            );
        }

        block_context
            .conditionally_referenced_variable_ids
            .extend(right_block_context.conditionally_referenced_variable_ids);
        block_context.assigned_variable_ids.extend(right_block_context.assigned_variable_ids);
    }

    if let Some(if_body_context) = &block_context.if_body_context {
        let mut if_body_context_inner = if_body_context.borrow_mut();
        let left_vars = left_block_context.locals.clone();
        let if_vars = if_body_context_inner.locals.clone();
        for (var_id, right_type) in right_block_context.locals.clone() {
            if let Some(if_type) = if_vars.get(&var_id) {
                if_body_context_inner.locals.insert(
                    var_id,
                    Rc::new(combine_union_types(&right_type, if_type, context.codebase, context.interner, false)),
                );
            } else if let Some(left_type) = left_vars.get(&var_id) {
                if_body_context_inner.locals.insert(
                    var_id,
                    Rc::new(combine_union_types(&right_type, left_type, context.codebase, context.interner, false)),
                );
            }
        }

        if_body_context_inner
            .conditionally_referenced_variable_ids
            .extend(block_context.conditionally_referenced_variable_ids.clone());
        if_body_context_inner.assigned_variable_ids.extend(block_context.assigned_variable_ids.clone());
    }

    let mut final_type_with_flow = result_type;
    let mut decision_node_parents = Vec::new();
    decision_node_parents.extend(lhs_type.parent_nodes.iter().cloned());
    if !lhs_type.is_always_truthy()
        && let Some(rhs_parents) = artifacts.get_expression_type(&binary.rhs).map(|t| &t.parent_nodes)
    {
        decision_node_parents.extend(rhs_parents.iter().cloned());
    }

    if !decision_node_parents.is_empty() {
        let decision_node = DataFlowNode::get_for_composition(binary.span());
        artifacts.data_flow_graph.add_node(decision_node.clone());
        for parent_node in decision_node_parents {
            if artifacts.data_flow_graph.get_node(&parent_node.id).is_some() {
                artifacts.data_flow_graph.add_path(&parent_node, &decision_node, PathKind::Default);
            }
        }

        final_type_with_flow.parent_nodes = vec![decision_node];
    }

    artifacts.set_expression_type(binary, final_type_with_flow);

    Ok(())
}

#[inline]
pub fn assign_arithmetic_type(artifacts: &mut AnalysisArtifacts, cond_type: TUnion, binary: &Binary) {
    let mut cond_type = cond_type;
    let decision_node = DataFlowNode::get_for_composition(binary.span());

    artifacts.data_flow_graph.add_node(decision_node.clone());

    if let Some(lhs_type) = artifacts.expression_types.get(&get_expression_range(&binary.lhs)) {
        cond_type.parent_nodes.push(decision_node.clone());

        for old_parent_node in &lhs_type.parent_nodes {
            artifacts.data_flow_graph.add_path(old_parent_node, &decision_node, PathKind::Default);
        }
    }

    if let Some(rhs_type) = artifacts.expression_types.get(&get_expression_range(&binary.rhs)) {
        cond_type.parent_nodes.push(decision_node.clone());

        for old_parent_node in &rhs_type.parent_nodes {
            artifacts.data_flow_graph.add_path(old_parent_node, &decision_node, PathKind::Default);
        }
    }

    artifacts.set_expression_type(binary, cond_type);
}

fn determine_numeric_result(op: &BinaryOperator, left: &TAtomic, right: &TAtomic, in_loop: bool) -> Vec<TAtomic> {
    if in_loop
        && (matches!(left, TAtomic::Scalar(TScalar::Integer(_)))
            || matches!(right, TAtomic::Scalar(TScalar::Integer(_))))
    {
        return match (left, right) {
            (TAtomic::Scalar(TScalar::Integer(_)), TAtomic::Scalar(TScalar::Integer(_))) => match op {
                BinaryOperator::Division(_) => vec![TAtomic::Scalar(TScalar::Number)],
                _ => vec![TAtomic::Scalar(TScalar::int())],
            },
            _ => match op {
                BinaryOperator::Modulo(_) => vec![TAtomic::Scalar(TScalar::int())],
                _ => vec![TAtomic::Scalar(TScalar::float())],
            },
        };
    }

    match (left, right) {
        (TAtomic::Scalar(TScalar::Integer(left_int)), TAtomic::Scalar(TScalar::Integer(right_int))) => {
            let result = calculate_int_arithmetic(op, *left_int, *right_int);

            match result {
                Some(integer) => {
                    vec![TAtomic::Scalar(TScalar::Integer(integer))]
                }
                None => {
                    if matches!(op, BinaryOperator::Division(_)) {
                        if right_int.is_zero() { vec![TAtomic::Never] } else { vec![TAtomic::Scalar(TScalar::Number)] }
                    } else {
                        vec![TAtomic::Scalar(TScalar::int())]
                    }
                }
            }
        }
        (TAtomic::Scalar(TScalar::Float(_)), _) | (_, TAtomic::Scalar(TScalar::Float(_))) => {
            // TODO(azjezz): handle literal floats?
            match op {
                BinaryOperator::Modulo(_) => vec![TAtomic::Scalar(TScalar::int())],
                _ => vec![TAtomic::Scalar(TScalar::float())],
            }
        }
        _ => match op {
            BinaryOperator::Modulo(_) => vec![TAtomic::Scalar(TScalar::int())],
            BinaryOperator::Division(_) => vec![TAtomic::Scalar(TScalar::Number)],
            _ => vec![TAtomic::Scalar(TScalar::Number)],
        },
    }
}

#[inline]
const fn is_logical_or_operation(expression: &Expression, max_nesting: usize) -> bool {
    if max_nesting == 0 {
        return true;
    }

    match expression {
        Expression::Parenthesized(p) => is_logical_or_operation(&p.expression, max_nesting),
        Expression::Binary(b) => match b.operator {
            BinaryOperator::Or(_) | BinaryOperator::LowOr(_) => is_logical_or_operation(&b.lhs, max_nesting - 1),
            _ => false,
        },
        _ => false,
    }
}

fn calculate_int_arithmetic(op: &BinaryOperator, left: TInteger, right: TInteger) -> Option<TInteger> {
    use TInteger::*;

    let result = match op {
        BinaryOperator::Addition(_) => left + right,
        BinaryOperator::Subtraction(_) => left - right,
        BinaryOperator::Multiplication(_) => left * right,
        BinaryOperator::Modulo(_) => left % right,
        BinaryOperator::BitwiseAnd(_) => left & right,
        BinaryOperator::BitwiseOr(_) => left | right,
        BinaryOperator::BitwiseXor(_) => left ^ right,
        BinaryOperator::LeftShift(_) => left << right,
        BinaryOperator::RightShift(_) => left >> right,
        BinaryOperator::Division(_) => match (left, right) {
            (Literal(l_val), Literal(r_val)) => {
                if r_val != 0 && l_val % r_val == 0 {
                    Literal(l_val / r_val)
                } else {
                    Unspecified
                }
            }
            _ => Unspecified,
        },
        BinaryOperator::Exponentiation(_) => match (left, right) {
            (Literal(l_val), Literal(r_val)) => {
                if r_val < 0 {
                    Unspecified
                } else {
                    match r_val.try_into() {
                        Ok(exponent_u32) => {
                            l_val.checked_pow(exponent_u32).map(TInteger::Literal).unwrap_or(Unspecified)
                        }
                        Err(_) => Unspecified,
                    }
                }
            }
            _ => Unspecified,
        },
        _ => return None,
    };

    if result.is_unspecified() { None } else { Some(result) }
}

#[inline]
fn is_always_less_than_or_equal(lhs: &TUnion, rhs: &TUnion, cb: &CodebaseMetadata, i: &ThreadedInterner) -> bool {
    if let (Some(max_lhs), Some(min_rhs)) = (lhs.get_single_maximum_int_value(), rhs.get_single_minimum_int_value()) {
        return max_lhs <= min_rhs;
    }

    is_always_less_than(lhs, rhs, cb, i) || is_always_identical_to(lhs, rhs, cb, i)
}

#[inline]
fn is_always_greater_than_or_equal(lhs: &TUnion, rhs: &TUnion, cb: &CodebaseMetadata, i: &ThreadedInterner) -> bool {
    if let (Some(min_lhs), Some(max_rhs)) = (lhs.get_single_minimum_int_value(), rhs.get_single_maximum_int_value()) {
        return min_lhs >= max_rhs;
    }

    is_always_greater_than(lhs, rhs, cb, i) || is_always_identical_to(lhs, rhs, cb, i)
}

/// Checks if the left-hand side type is always strictly less than the right-hand side type.
/// Returns `false` if uncertain.
fn is_always_less_than(lhs: &TUnion, rhs: &TUnion, _cb: &CodebaseMetadata, _i: &ThreadedInterner) -> bool {
    if lhs.is_null() && !rhs.is_null() {
        return true;
    }

    if lhs.is_false() && rhs.is_true() {
        return true;
    }

    if lhs.is_false() && !rhs.is_null() && !rhs.is_false() {
        return true;
    }

    if !lhs.is_single() || !rhs.is_single() {
        return false;
    }

    let lhs_atomic = lhs.get_single();
    let rhs_atomic = rhs.get_single();

    match (lhs_atomic, rhs_atomic) {
        (TAtomic::Scalar(TScalar::Integer(l)), TAtomic::Scalar(TScalar::Integer(r))) => match (l, r) {
            (TInteger::Literal(l_val), TInteger::Literal(r_val)) => return l_val < r_val,
            _ => return false,
        },
        (TAtomic::Scalar(TScalar::Float(l)), TAtomic::Scalar(TScalar::Float(r))) => match (l.value, r.value) {
            (Some(l_val), Some(r_val)) => return l_val < r_val,
            _ => return false,
        },
        _ => {}
    }

    false
}

/// Checks if the left-hand side type is always strictly greater than the right-hand side type.
/// Returns `false` if uncertain.
fn is_always_greater_than(lhs: &TUnion, rhs: &TUnion, _cb: &CodebaseMetadata, _i: &ThreadedInterner) -> bool {
    if !lhs.is_null() && rhs.is_null() {
        return true;
    }

    if lhs.is_true() && rhs.is_false() {
        return true;
    }

    if lhs.is_true() && !rhs.is_null() && !rhs.is_true() {
        return true;
    }

    if !lhs.is_single() || !rhs.is_single() {
        return false;
    }

    let lhs_atomic = lhs.get_single();
    let rhs_atomic = rhs.get_single();

    match (lhs_atomic, rhs_atomic) {
        (TAtomic::Scalar(TScalar::Integer(l)), TAtomic::Scalar(TScalar::Integer(r))) => match (l, r) {
            (TInteger::Literal(l_val), TInteger::Literal(r_val)) => return l_val > r_val,
            _ => return false,
        },
        (TAtomic::Scalar(TScalar::Float(l)), TAtomic::Scalar(TScalar::Float(r))) => match (l.value, r.value) {
            (Some(l_val), Some(r_val)) => return l_val > r_val,
            _ => return false,
        },
        _ => {}
    }

    false
}

pub(super) fn is_always_identical_to(
    lhs: &TUnion,
    rhs: &TUnion,
    _cb: &CodebaseMetadata,
    _i: &ThreadedInterner,
) -> bool {
    if lhs.is_null() && rhs.is_null() {
        return true;
    }

    if lhs.is_false() && rhs.is_false() {
        return true;
    }

    if lhs.is_true() && rhs.is_true() {
        return true;
    }

    if lhs.is_enum() && rhs.is_enum() {
        let left_cases = lhs.get_enum_cases();
        let right_cases = rhs.get_enum_cases();

        if left_cases.len() > 1 || right_cases.len() > 1 {
            return false;
        }

        let (left_enum, left_case) = left_cases[0];
        let (right_enum, right_case) = right_cases[0];

        return right_case.is_some() && left_case.is_some() && left_enum == right_enum && left_case == right_case;
    }

    if let (Some(l), Some(r)) = (lhs.get_single_literal_int_value(), rhs.get_single_literal_int_value()) {
        return l == r;
    }

    if let (Some(l), Some(r)) = (lhs.get_single_literal_float_value(), rhs.get_single_literal_float_value()) {
        return l == r;
    }

    if let (Some(l), Some(r)) = (lhs.get_single_literal_string_value(), rhs.get_single_literal_string_value()) {
        return l == r;
    }

    false
}

pub fn are_definitely_not_identical(lhs: &TUnion, rhs: &TUnion, cb: &CodebaseMetadata, i: &ThreadedInterner) -> bool {
    // If either type is mixed, we cannot determine non-identity.
    if lhs.has_mixed() || lhs.has_mixed_template() || rhs.has_mixed() || rhs.has_mixed_template() {
        return false;
    }

    if !can_expression_types_be_identical(cb, i, lhs, rhs, true) {
        return true;
    }

    if (lhs.is_never() && !rhs.is_never()) || (!lhs.is_never() && rhs.is_never()) {
        return true;
    }

    if (lhs.is_null() && (!rhs.is_null() && !rhs.is_nullable()))
        || (rhs.is_null() && (!lhs.is_null() && !lhs.is_nullable()))
    {
        return true;
    }

    if lhs.is_bool() {
        if !rhs.has_bool() {
            return true;
        }

        if rhs.is_true() && lhs.is_false() {
            return true;
        }

        if rhs.is_false() && lhs.is_true() {
            return true;
        }

        return !rhs.has_bool();
    } else if rhs.is_bool() && !lhs.has_bool() {
        return true;
    }

    if let Some(l) = lhs.get_single_literal_int_value()
        && let Some(r) = rhs.get_single_literal_int_value()
    {
        return l != r;
    }

    if let Some(l) = lhs.get_single_literal_float_value()
        && let Some(r) = rhs.get_single_literal_float_value()
    {
        return l != r;
    }

    if let Some(l) = lhs.get_single_literal_string_value() {
        if let Some(r) = rhs.get_single_literal_string_value() {
            return l != r;
        } else if let Some(r) = rhs.get_single_class_string_value() {
            return !l.eq_ignore_ascii_case(i.lookup(&r));
        }
    } else if let Some(r) = rhs.get_single_literal_string_value()
        && let Some(l) = lhs.get_single_class_string_value()
    {
        return !r.eq_ignore_ascii_case(i.lookup(&l));
    }

    false
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::issue::TypingIssueKind;
    use crate::test_analysis;

    test_analysis! {
        name = concat_operator_test,
        code = indoc! {r#"
            <?php

            $name = "world";

            echo "Hello " . $name;
        "#}
    }

    test_analysis! {
        name = assertions_are_applied,
        code = indoc! {r#"
            <?php

            const PHP_INT_MAX = 9223372036854775807;

            /**
             * @param int<1, max> $length
             * @return list<string>
             */
            function str_split(string $string, int $length = 1): array
            {
                return str_split($string, $length);
            }

            function intdiv(int $num1, int $num2): int
            {
                return intdiv($num1, $num2);
            }

            /**
             * @param string $character
             * @return int<0, 255>
             */
            function ord(string $character): int
            {
                return ord($character);
            }

            /**
             * @param non-empty-string $number
             * @param int<2, 36> $from_base
             */
            function from_base(string $number, int $from_base): int
            {
                $limit = intdiv(PHP_INT_MAX, $from_base);
                $result = 0;
                foreach (str_split($number, 1) as $digit) {
                    $oval = ord($digit);

                    if (/* '0' - '9' */ $oval >= 48 && $oval <= 57) {
                        $dval = $oval - 48;
                    } elseif (/* 'a' - 'z' */ $oval >= 97 && $oval <= 122) {
                        $dval = $oval - 87;
                    } elseif (/* 'A' - 'Z' */ $oval >= 65 && $oval <= 90) {
                        $dval = $oval - 55;
                    } else {
                        $dval = 99;
                    }

                    if ($from_base < $dval) {
                        exit('Invalid digit ' . $digit . ' in base ' . $from_base);
                    }

                    $oldval = $result;
                    $result = ($from_base * $result) + $dval;
                    if ($oldval > $limit || $oldval > $result) {
                        exit('Unexpected integer overflow parsing ' . $number . ' from base ' . $from_base);
                    }
                }

                return $result;
            }
        "#}
    }

    test_analysis! {
        name = array_to_string_conversion_within_concat_operand,
        code = indoc! {r#"
            <?php

            $name = ["world"];

            echo "Hello " . $name;
        "#},
        issues = [
            TypingIssueKind::ArrayToStringConversion,
        ]
    }

    test_analysis! {
        name = bitwise_or_binary_operator,
        code = indoc! {r#"
            <?php

            const JSON_BIGINT_AS_STRING = 2;

            function x(): int
            {
                $a = JSON_BIGINT_AS_STRING | 1;

                return $a;
            }
        "#},
    }

    test_analysis! {
        name = arithmetic_on_generics,
        code = indoc! {r#"
            <?php

            /**
             * @template T of int|float
             *
             * @param T $start
             * @param T $end
             * @param T|null $step
             *
             * @return non-empty-list<T>
             */
            function range(int|float $start, int|float $end, int|float|null $step = null): array
            {
                if (((float) $start) === ((float) $end)) {
                    return [$start];
                }

                if ($start < $end) {
                    if (null === $step) {
                        $step = 1;
                    }

                    if ($step < 0) {
                        exit('If $end is greater than $start, then $step must be positive or null.');
                    }

                    $result = [];
                    for ($i = $start; $i <= $end; $i += $step) {
                        $result[] = $i;
                    }

                    return $result;
                }

                if (null === $step) {
                    $step = -1;
                }

                if ($step > 0) {
                    exit('If $start is greater than $end, then $step must be negative or null.');
                }

                $result = [];
                for ($i = $start; $i >= $end; $i += $step) {
                    $result[] = $i;
                }

                return $result;
            }
        "#},
    }

    test_analysis! {
        name = codepoints,
        code = indoc! {r#"
            <?php

            // stub
            function chr(int $code): string {
                return (string) $code;
            }

            function from_code_points(int ...$code_points): string
            {
                $string = '';
                foreach ($code_points as $code) {
                    $code %= 0x200000;
                    if (0x80 > $code) {
                        $string .= chr($code);
                        continue;
                    }

                    if (0x800 > $code) {
                        $string .= chr(0xC0 | ($code >> 6)) . chr(0x80 | ($code & 0x3F));
                        continue;
                    }

                    if (0x10000 > $code) {
                        $string .= chr(0xE0 | ($code >> 12)) . chr(0x80 | (($code >> 6) & 0x3F));
                        $string .= chr(0x80 | ($code & 0x3F));
                        continue;
                    }

                    $string .= chr(0xF0 | ($code >> 18)) . chr(0x80 | (($code >> 12) & 0x3F));
                    $string .= chr(0x80 | (($code >> 6) & 0x3F)) . chr(0x80 | ($code & 0x3F));
                }

                return $string;
            }
        "#},
    }

    test_analysis! {
        name = null_coalescing_mixed,
        code = indoc! {r#"
            <?php

            function test($foo = null) {
                return $foo ?? 'bar';
            }
        "#},
    }

    test_analysis! {
        name = cant_determine_if_types_are_identical_for_mixed_template,
        code = indoc! {r#"
            <?php

            /**
             * @template T
             * @param T $x
             */
            function x(mixed $x): void {
                if (false === $x) {
                    echo 'X is false';
                } else {
                    echo 'X is not false';
                }
            }
        "#},
    }

    test_analysis! {
        name = class_string_is_never_equal_to_literal_string,
        code = indoc! {r#"
            <?php

            class B
            {
            }

            class C
            {
            }

            class D
            {
            }

            class E
            {
            }

            class F
            {
            }

            class G
            {
            }

            class H
            {
            }

            class I
            {
            }

            class J
            {
            }

            class K
            {
            }

            class L
            {
            }

            class M
            {
            }

            class N
            {
            }

            class O
            {
            }

            class P
            {
            }

            class Q
            {
            }

            class R
            {
            }

            class S
            {
            }

            class T
            {
            }

            class U
            {
            }

            class V
            {
            }

            class W
            {
            }

            class X
            {
            }

            class Y
            {
            }

            class Z
            {
            }

            $type = '';
            if ($type === '1') {
            } elseif (
                $type === B::class ||
                    $type === C::class ||
                    $type === D::class ||
                    $type === E::class ||
                    $type === F::class ||
                    $type === G::class ||
                    $type === H::class ||
                    $type === I::class ||
                    $type === J::class ||
                    $type === K::class ||
                    $type === L::class ||
                    $type === M::class ||
                    $type === N::class ||
                    $type === O::class ||
                    $type === P::class ||
                    $type === Q::class ||
                    $type === R::class ||
                    $type === S::class ||
                    $type === T::class ||
                    $type === U::class ||
                    $type === V::class ||
                    $type === W::class ||
                    $type === X::class ||
                    $type === Y::class ||
                    $type === Z::class
            ) {
            }
        "#},
        issues = [
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantLogicalOperation,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::RedundantComparison,
            TypingIssueKind::ImpossibleCondition,
            TypingIssueKind::ImpossibleCondition,
        ]
    }

    test_analysis! {
        name = int_mod,
        code = indoc! {r#"
            <?php

            const NANOSECONDS_PER_SECOND = 1_000_000_000;

            const MICROSECONDS_PER_SECOND = 1_000_000;

            const MILLISECONDS_PER_SECOND = 1000;

            const SECONDS_PER_MINUTE = 60;

            const SECONDS_PER_HOUR = 3600;

            final readonly class Duration
            {
                /**
                 * @param int $hours
                 * @param int<-59, 59> $minutes
                 * @param int<-59, 59> $seconds
                 * @param int<-999999999, 999999999> $nanoseconds
                 *
                 * @pure
                 */
                private function __construct(
                    private int $hours,
                    private int $minutes,
                    private int $seconds,
                    private int $nanoseconds,
                ) {}

                /**
                 * @pure
                 */
                public static function fromParts(int $hours, int $minutes = 0, int $seconds = 0, int $nanoseconds = 0): self
                {
                    $s =
                        (SECONDS_PER_HOUR * $hours) +
                        (SECONDS_PER_MINUTE * $minutes) +
                        $seconds +
                        ((int) ($nanoseconds / NANOSECONDS_PER_SECOND));

                    $ns = $nanoseconds % NANOSECONDS_PER_SECOND;

                    if ($s < 0 && $ns > 0) {
                        ++$s;
                        $ns -= NANOSECONDS_PER_SECOND;
                    } elseif ($s > 0 && $ns < 0) {
                        --$s;
                        $ns += NANOSECONDS_PER_SECOND;
                    }

                    $m = (int) ($s / 60);
                    $s %= 60;
                    $h = (int) ($m / 60);
                    $m %= 60;

                    return new self($h, $m, $s, $ns);
                }
            }
        "#},
    }

    test_analysis! {
        name = string_manipulation,
        code = indoc! {r#"
            <?php

            const STR_PAD_RIGHT = 0;

            const STR_PAD_LEFT = 1;

            /**
             * @pure
             */
            function abs(int|float $num): int|float
            {
                return abs($num);
            }

            /**
             * @pure
             */
            function str_pad(string $string, int $length, string $pad_string = ' ', int $pad_type = STR_PAD_RIGHT): string
            {
                return str_pad($string, $length, $pad_string, $pad_type);
            }

            /**
             * @pure
             */
            function substr(string $string, int $offset, null|int $length = null): string
            {
                return substr($string, $offset, $length);
            }

            /**
             * @pure
             */
            function rtrim(string $string, string $characters = " \n\r\t\v\0"): string
            {
                return rtrim($string, $characters);
            }

            /**
             * @param array<string>|string $separator
             * @param array<string>|null $array
             *
             * @pure
             */
            function join(array|string $separator = '', null|array $array = null): string
            {
                return join($separator, $array);
            }

            /**
             * @param int $hours
             * @param int<-59, 59> $minutes
             * @param int<-59, 59> $seconds
             * @param int<-999999999, 999999999> $nanoseconds
             * @param int<0, max> $max_decimals
             *
             * @pure
             */
            function format_duration(int $hours, int $minutes, int $seconds, int $nanoseconds, int $max_decimals = 3): string
            {
                $decimal_part = '';
                if ($max_decimals > 0) {
                    $decimal_part = (string) abs($nanoseconds);
                    $decimal_part = str_pad($decimal_part, 9, '0', STR_PAD_LEFT);
                    $decimal_part = substr($decimal_part, 0, $max_decimals);
                    $decimal_part = rtrim($decimal_part, '0');
                }

                if ($decimal_part !== '') {
                    $decimal_part = '.' . $decimal_part;
                }

                $sec_sign = $seconds < 0 || $nanoseconds < 0 ? '-' : '';
                $sec = abs($seconds);

                $containsHours = $hours !== 0;
                $containsMinutes = $minutes !== 0;
                $concatenatedSeconds = $sec_sign . ((string) $sec) . $decimal_part;
                $containsSeconds = $concatenatedSeconds !== '0';

                /** @var list<non-empty-string> $output */
                $output = [];
                if ($containsHours) {
                    $output[] = ((string) $hours) . ' hour(s)';
                }

                if ($containsMinutes || $containsHours && $containsSeconds) {
                    $output[] = ((string) $minutes) . ' minute(s)';
                }

                if ($containsSeconds) {
                    $output[] = $concatenatedSeconds . ' second(s)';
                }

                return [] === $output ? '0 second(s)' : join(', ', $output);
            }
        "#},
    }
}
