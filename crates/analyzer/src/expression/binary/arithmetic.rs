use std::collections::VecDeque;
use std::rc::Rc;

use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::mixed::TMixed;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::combiner;
use mago_codex::ttype::comparator::ComparisonResult;
use mago_codex::ttype::comparator::atomic_comparator;
use mago_codex::ttype::get_mixed;
use mago_codex::ttype::union::TUnion;
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

#[inline]
pub fn analyze_arithmetic_operation<'ctx, 'arena>(
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

    let fallback = Rc::new(get_mixed());
    let left_type = artifacts.get_rc_expression_type(&binary.lhs).cloned().unwrap_or_else(|| fallback.clone());
    let right_type = artifacts.get_rc_expression_type(&binary.rhs).cloned().unwrap_or_else(|| fallback.clone());

    let mut final_result_type: Option<TUnion> = None;

    if left_type.is_null() {
        context.collector.report_with_code(
            IssueCode::NullOperand,
            Issue::error("Left operand in arithmetic operation cannot be `null`.")
                .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This is `null`."))
                .with_note("Performing arithmetic operations on `null` typically results in `0`.")
                .with_help("Ensure the left operand is a number (int/float) or a type that can be cast to a number."),
        );

        // In Psalm, null operand often leads to mixed result or halts analysis for this path.
        // Let's set result to mixed and return, similar to Psalm's behavior.
        final_result_type = Some(get_mixed());
    } else if left_type.is_nullable() && !left_type.ignore_nullable_issues {
        context.collector.report_with_code(
            IssueCode::PossiblyNullOperand,
            Issue::warning(format!(
                "Left operand in arithmetic operation might be `null` (type `{}`).",
                left_type.get_id()
            ))
            .with_annotation(Annotation::primary(binary.lhs.span()).with_message("This might be `null`."))
            .with_note("Performing arithmetic operations on `null` typically results in `0`.")
            .with_help(
                "Ensure the left operand is non-null before the operation, potentially using checks or assertions.",
            ),
        );
    }

    if right_type.is_null() {
        context.collector.report_with_code(
            IssueCode::NullOperand,
            Issue::error("Right operand in arithmetic operation cannot be `null`.")
                .with_annotation(Annotation::primary(binary.rhs.span()).with_message("This is `null`."))
                .with_note("Performing arithmetic operations on `null` typically results in `0`.")
                .with_help("Ensure the right operand is a number (int/float) or a type that can be cast to a number."),
        );

        final_result_type = Some(get_mixed());
    } else if right_type.is_nullable() && !right_type.ignore_nullable_issues {
        context.collector.report_with_code(
            IssueCode::PossiblyNullOperand,
            Issue::warning(format!(
                "Right operand in arithmetic operation might be `null` (type `{}`).",
                right_type.get_id()
            ))
            .with_annotation(Annotation::primary(binary.rhs.span()).with_message("This might be `null`"))
            .with_note("Performing arithmetic operations on `null` typically results in `0`.")
            .with_help(
                "Ensure the right operand is non-null before the operation, potentially using checks or assertions.",
            ),
        );
    }

    if is_arithmetic_compatible_generic(context, &left_type, &right_type) {
        final_result_type = Some(left_type.as_ref().clone());
    } else if is_arithmetic_compatible_generic(context, &right_type, &left_type) {
        final_result_type = Some(right_type.as_ref().clone());
    }

    if let Some(final_result_type) = final_result_type {
        assign_arithmetic_type(artifacts, final_result_type, binary);

        return Ok(());
    }

    if left_type.is_false() {
        context.collector.report_with_code(
            IssueCode::FalseOperand,
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
        context.collector.report_with_code(
            IssueCode::PossiblyFalseOperand,
            Issue::warning(format!(
                "Left operand in arithmetic operation might be `false` (type `{}`).",
                left_type.get_id()
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
        context.collector.report_with_code(
            IssueCode::FalseOperand,
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
        context.collector.report_with_code(
            IssueCode::PossiblyFalseOperand,
            Issue::warning(format!(
                "Right operand in arithmetic operation might be `false` (type `{}`).",
                right_type.get_id()
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
            if let TAtomic::GenericParameter(parameter) = atomic {
                parameter.constraint.types.into_owned()
            } else {
                vec![atomic]
            }
        })
        .collect::<VecDeque<_>>();

    let right_atomic_types = right_type
        .types
        .iter()
        .cloned()
        .flat_map(|atomic| {
            if let TAtomic::GenericParameter(parameter) = atomic {
                parameter.constraint.types.into_owned()
            } else {
                vec![atomic]
            }
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
                context.collector.report_with_code(
                    IssueCode::MixedOperand,
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

                pair_result_atomics.push(TAtomic::Mixed(TMixed::new()));
                if !right_atomic.is_mixed() {
                    has_valid_right_operand = true;
                }
            }

            if right_atomic.is_mixed() {
                context.collector.report_with_code(
                    IssueCode::MixedOperand,
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
                    pair_result_atomics.push(TAtomic::Mixed(TMixed::new()));
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
                    pair_result_atomics.extend(combiner::combine(
                        vec![left_atomic.clone(), right_atomic.clone()],
                        context.codebase,
                        false,
                    ));

                    has_valid_left_operand = true;
                    has_valid_right_operand = true;
                } else if left_atomic.is_array() {
                    invalid_right_messages.push((
                        format!("Cannot add array to non-array type {}", right_atomic.get_id()),
                        binary.rhs.span(),
                    ));

                    has_valid_left_operand = true;
                    invalid_pair = true;
                } else {
                    invalid_left_messages.push((
                        format!("Cannot add {} to non-array type array", left_atomic.get_id()),
                        binary.lhs.span(),
                    ));

                    has_valid_right_operand = true;
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
                    format!("Cannot perform arithmetic operation with non-numeric type {}", right_atomic.get_id()),
                    binary.rhs.span(),
                ));
                has_valid_left_operand = true;
                invalid_pair = true;
            } else if right_atomic.is_numeric() {
                invalid_left_messages.push((
                    format!("Cannot perform arithmetic operation with non-numeric type {}", left_atomic.get_id()),
                    binary.lhs.span(),
                ));
                has_valid_right_operand = true;
                invalid_pair = true;
            } else {
                invalid_left_messages.push((
                    format!("Cannot perform arithmetic operation on type {}", left_atomic.get_id()),
                    binary.lhs.span(),
                ));

                invalid_right_messages.push((
                    format!("Cannot perform arithmetic operation on type {}", right_atomic.get_id()),
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
        let issue_kind =
            if has_valid_left_operand { IssueCode::PossiblyInvalidOperand } else { IssueCode::InvalidOperand };

        let mut issue = if has_valid_left_operand {
            Issue::warning("Possibly invalid type for left operand.".to_string())
        } else {
            Issue::error("Invalid type for left operand.".to_string())
        };

        let mut is_first = true;
        for (msg, span) in invalid_left_messages {
            issue = issue.with_annotation(if is_first {
                Annotation::primary(span).with_message(msg)
            } else {
                Annotation::secondary(span).with_message(msg)
            });

            is_first = false;
        }

        context.collector.report_with_code(
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
        let issue_kind =
            if has_valid_right_operand { IssueCode::PossiblyInvalidOperand } else { IssueCode::InvalidOperand };

        let mut issue = if has_valid_right_operand {
            Issue::warning("Possibly invalid type for right operand.".to_string())
        } else {
            Issue::error("Invalid type for right operand.".to_string())
        };

        let mut is_first = true;
        for (msg, span) in invalid_right_messages {
            issue = issue.with_annotation(if is_first {
                Annotation::primary(span).with_message(msg)
            } else {
                Annotation::secondary(span).with_message(msg)
            });

            is_first = false;
        }

        context.collector.report_with_code(
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
        TUnion::from_vec(combiner::combine(result_atomic_types, context.codebase, false))
    } else {
        // No valid pairs found, and potentially errors issued.
        // Psalm often defaults to mixed here if operands were invalid.
        // If errors were due to null/false operands handled initially, use the type set there.
        // Otherwise, default to mixed.
        get_mixed()
    };

    assign_arithmetic_type(artifacts, final_type, binary);

    Ok(())
}

#[inline]
fn is_arithmetic_compatible_generic<'ctx, 'arena>(
    context: &Context<'ctx, 'arena>,
    union: &TUnion,
    other_union: &TUnion,
) -> bool {
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
pub fn assign_arithmetic_type<'ast, 'arena>(
    artifacts: &mut AnalysisArtifacts,
    cond_type: TUnion,
    binary: &'ast Binary<'arena>,
) {
    artifacts.set_expression_type(binary, cond_type);
}

fn determine_numeric_result<'ast, 'arena>(
    op: &'ast BinaryOperator<'arena>,
    left: &TAtomic,
    right: &TAtomic,
    in_loop: bool,
) -> Vec<TAtomic> {
    if in_loop
        && (matches!(left, TAtomic::Scalar(TScalar::Integer(_)))
            || matches!(right, TAtomic::Scalar(TScalar::Integer(_))))
    {
        return match (left, right) {
            (TAtomic::Scalar(TScalar::Integer(_)), TAtomic::Scalar(TScalar::Integer(_))) => match op {
                BinaryOperator::Division(_) => vec![TAtomic::Scalar(TScalar::int()), TAtomic::Scalar(TScalar::float())],
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
                        if right_int.is_zero() {
                            vec![TAtomic::Never]
                        } else {
                            vec![TAtomic::Scalar(TScalar::int()), TAtomic::Scalar(TScalar::float())]
                        }
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
            _ => {
                vec![TAtomic::Scalar(TScalar::int()), TAtomic::Scalar(TScalar::float())]
            }
        },
    }
}

fn calculate_int_arithmetic<'ast, 'arena>(
    op: &'ast BinaryOperator<'arena>,
    left: TInteger,
    right: TInteger,
) -> Option<TInteger> {
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
