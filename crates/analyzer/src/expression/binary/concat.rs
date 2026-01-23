use std::rc::Rc;

use mago_atom::atom;
use mago_atom::concat_atom;
use mago_atom::f64_atom;
use mago_atom::i64_atom;

use mago_codex::ttype::TType;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::string::TString;
use mago_codex::ttype::atomic::scalar::string::TStringLiteral;
use mago_codex::ttype::get_string;
use mago_codex::ttype::get_string_with_props;
use mago_codex::ttype::union::TUnion;
use mago_reporting::Annotation;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_syntax::ast::Binary;
use mago_syntax::ast::BinaryOperator;
use mago_syntax::ast::Expression;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::artifacts::get_expression_range;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

/// Iteratively collects all operands in a string concatenation chain.
///
/// For `"a" . "b" . "c"`, this returns `["a", "b", "c"]` instead of recursing through the nested binary nodes.
/// This avoids stack overflow on deeply nested concatenations.
fn collect_concat_operands<'a, 'arena>(binary: &'a Binary<'arena>) -> Vec<&'a Expression<'arena>> {
    let mut operands = Vec::new();
    let mut stack: Vec<&Expression<'arena>> = vec![&binary.rhs, &binary.lhs];

    while let Some(expr) = stack.pop() {
        if let Expression::Binary(inner_binary) = expr
            && matches!(&inner_binary.operator, BinaryOperator::StringConcat(_))
        {
            stack.push(inner_binary.rhs);
            stack.push(inner_binary.lhs);
            continue;
        }

        operands.push(expr);
    }

    operands
}

/// Analyzes a string concatenation operation using an iterative approach to avoid stack overflow.
///
/// Instead of recursively calling analyze on each operand (which would recurse back into this
/// function for nested concatenations), we:
/// 1. Flatten the concatenation chain into a list of leaf operands
/// 2. Analyze each leaf operand (non-concat expressions don't cause deep recursion)
/// 3. Validate and compute the result type by folding left-to-right
#[inline(never)]
pub fn analyze_string_concat_operation<'ctx, 'arena>(
    binary: &Binary<'arena>,
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    let operands = collect_concat_operands(binary);

    for operand in &operands {
        operand.analyze(context, block_context, artifacts)?;
    }

    let len = operands.len();
    for (i, operand) in operands.iter().enumerate() {
        let side = if i == 0 {
            "left"
        } else if i == len - 1 {
            "right"
        } else {
            "middle"
        };

        analyze_string_concat_operand(context, artifacts, operand, side);
    }

    let result_type = fold_concat_operands(&operands, artifacts, context.settings.string_concat_combination_threshold);
    artifacts.expression_types.insert(get_expression_range(binary), Rc::new(result_type));

    Ok(())
}

#[inline]
fn analyze_string_concat_operand<'arena>(
    context: &mut Context<'_, 'arena>,
    artifacts: &mut AnalysisArtifacts,
    operand: &Expression<'arena>,
    side: &'static str,
) {
    let Some(operand_type) = artifacts.get_expression_type(operand) else {
        return;
    };

    if operand_type.is_null() {
        context.collector.report_with_code(
            IssueCode::NullOperand,
            Issue::error(format!(
                "Implicit conversion of `null` to empty string for {side} operand in string concatenation."
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("Operand is `null` here"))
            .with_note("Using `null` in string concatenation results in an empty string `''`.")
            .with_help(
                "Explicitly cast to string `(string) $var` or handle the `null` case if concatenation is not intended.",
            ),
        );

        return;
    }

    if operand_type.is_false() {
        context.collector.report_with_code(
            IssueCode::FalseOperand,
            Issue::error(format!(
                "Implicit conversion of `false` to empty string for {side} operand in string concatenation."
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("Operand is `false` here"))
            .with_note("Using `false` in string concatenation results in an empty string `''`.")
            .with_help("Explicitly cast to string `(string) $var` or handle the `false` case if concatenation is not intended."),
        );

        return;
    }

    if operand_type.is_nullable() && !operand_type.ignore_nullable_issues() {
        context.collector.report_with_code(
            IssueCode::PossiblyNullOperand,
            Issue::warning(format!(
                "Possibly null {} operand used in string concatenation (type `{}`).",
                side,
                operand_type.get_id()
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This might be `null`"))
            .with_note("If this operand is `null` at runtime, it will be implicitly converted to an empty string `''`.")
            .with_help("Ensure the operand is non-null before concatenation using checks or assertions, or explicitly cast to string."),
        );
    }

    if operand_type.is_falsable() && !operand_type.ignore_falsable_issues() {
        context.collector.report_with_code(
            IssueCode::PossiblyFalseOperand,
            Issue::warning(format!(
                "Possibly false {} operand used in string concatenation (type `{}`).",
                side,
                operand_type.get_id()
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This might be `false`"))
            .with_note(
                "If this operand is `false` at runtime, it will be implicitly converted to an empty string `''`.",
            )
            .with_help("Ensure the operand is non-falsy before concatenation, or explicitly cast to string."),
        );
    }

    let has_array = operand_type.types.iter().any(|t| matches!(t, TAtomic::Array(_)));
    let has_non_array = operand_type.types.iter().any(|t| !matches!(t, TAtomic::Array(_)));
    let is_mixed_union = has_array && has_non_array;

    let mut overall_type_match = true;
    let mut has_at_least_one_valid_operand_type = false;
    let mut reported_invalid_issue = false;

    for operand_atomic_type in operand_type.types.as_ref() {
        if operand_atomic_type.is_any_string()
            || operand_atomic_type.is_int()
            || operand_atomic_type.is_float()
            || operand_atomic_type.is_array_key()
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
                    || parameter.constraint.is_float()
                    || parameter.constraint.is_array_key()
                    || parameter.constraint.is_mixed()
                {
                    current_atomic_is_valid = true;
                } else {
                    if !reported_invalid_issue {
                        context.collector.report_with_code(
                            IssueCode::InvalidOperand,
                            Issue::error(format!(
                                "Invalid {} operand: template parameter `{}` constraint `{}` is not compatible with string concatenation.",
                                side,
                                parameter.parameter_name,
                                parameter.constraint.get_id()
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
                        context.collector.report_with_code(
                            IssueCode::InvalidOperand,
                            Issue::error(format!(
                                "Invalid {side} operand: cannot determine if generic `object` is stringable."
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

                if context.codebase.method_exists(class_like_name, "__toString") {
                    current_atomic_is_valid = true;

                    context.collector.report_with_code(
                        IssueCode::ImplicitToStringCast,
                        Issue::warning(format!(
                            "Implicit conversion to `string` for {side} operand via `{class_like_name}::__toString()`."
                        ))
                        .with_annotation(Annotation::primary(operand.span())
                            .with_message(format!("Object implicitly converted using `{class_like_name}::__toString()`"))
                        )
                        .with_note("Objects implementing `__toString` are automatically converted when used in string context.")
                        .with_help("For clarity, consider explicit casting `(string) $object` or calling the `__toString` method directly."),
                    );
                } else {
                    if !reported_invalid_issue {
                        context.collector.report_with_code(
                            IssueCode::InvalidOperand,
                            Issue::error(format!(
                                "Invalid {} operand: object of type `{}` cannot be converted to `string`.",
                                side,
                                operand_atomic_type.get_id()
                            ))
                            .with_annotation(Annotation::primary(operand.span())
                                .with_message(format!("Type `{}` does not have a `__toString` method", operand_atomic_type.get_id()))
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
                    if is_mixed_union {
                        context.collector.report_with_code(
                            IssueCode::ArrayToStringConversion,
                            Issue::warning(format!(
                                "Potential array in {side} operand of string concatenation."
                            ))
                            .with_annotation(
                                Annotation::primary(operand.span())
                                    .with_message("This expression may be an array."),
                            )
                            .with_note(
                                "Using an array in string concatenation produces the literal 'Array' and triggers a PHP warning.",
                            )
                            .with_help(
                                "Add a type check (e.g., `is_string()`) before concatenation to ensure the value is not an array.",
                            ),
                        );
                    } else {
                        context.collector.report_with_code(
                            IssueCode::ArrayToStringConversion,
                            Issue::error(format!(
                                "Invalid {side} operand: cannot use type `array` in string concatenation."
                            ))
                            .with_annotation(
                                Annotation::primary(operand.span())
                                    .with_message("Cannot concatenate with an `array`"),
                            )
                            .with_note(
                                "PHP raises an `E_WARNING` or `E_NOTICE` and uses the literal string 'Array' when an array is used in string context.",
                            )
                            .with_help(
                                "Do not use arrays directly in string concatenation. Use `implode()`, `json_encode()`, or loop to format its contents.",
                            ),
                        );
                    }

                    reported_invalid_issue = true;
                }

                overall_type_match = false;
            }
            TAtomic::Resource(_) => {
                context.collector.report_with_code(
                    IssueCode::ImplicitResourceToStringCast,
                    Issue::warning(format!(
                        "Implicit conversion of `resource` to string for {side} operand."
                    ))
                    .with_annotation(Annotation::primary(operand.span()).with_message("Resource implicitly converted to string"))
                    .with_note("PHP converts resources to the string format 'Resource id #[id]' when used in string context.")
                    .with_help("Avoid relying on implicit resource-to-string conversion; extract necessary information from the resource first if possible."),
                );

                current_atomic_is_valid = true;
            }
            TAtomic::Mixed(_) => {
                if !reported_invalid_issue {
                    context.collector.report_with_code(
                        IssueCode::MixedOperand,
                        Issue::error(format!(
                            "Invalid {} operand: type `{}` cannot be reliably used in string concatenation.",
                            side,
                            operand_atomic_type.get_id()
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
                    context.collector.report_with_code(
                        IssueCode::InvalidOperand,
                        Issue::error(format!(
                            "Invalid type `{}` for {} operand in string concatenation.",
                             operand_atomic_type.get_id(),
                             side
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
        context.collector.report_with_code(
            IssueCode::InvalidOperand,
            Issue::error(format!(
                "Invalid type `{}` for {} operand in string concatenation.",
                operand_type.get_id(), side
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("Invalid type for concatenation"))
            .with_note("Operands in string concatenation must be strings, numbers, null, false, resources, or objects implementing `__toString`.")
            .with_help("Ensure the operand has a compatible type or cast it explicitly to `string`."),
        );
    } else if !overall_type_match && has_at_least_one_valid_operand_type && !reported_invalid_issue {
        context.collector.report_with_code(
            IssueCode::PossiblyInvalidOperand,
            Issue::warning(format!(
                "Possibly invalid type `{}` for {} operand in string concatenation.",
                operand_type.get_id(),
                side
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
}

/// Folds multiple operands into a single result type by concatenating left-to-right.
///
/// This is used by the iterative concat analysis to compute the final type from
/// a flattened list of operands.
fn fold_concat_operands(operands: &[&Expression<'_>], artifacts: &AnalysisArtifacts, threshold: u16) -> TUnion {
    if operands.is_empty() {
        return get_string();
    }

    // Get strings for the first operand
    let first_type = artifacts.get_expression_type(operands[0]);
    let mut current_strings = match first_type {
        Some(t) => get_operand_strings(t),
        None => vec![TString::general()],
    };

    // Fold each subsequent operand
    for operand in &operands[1..] {
        let operand_type = artifacts.get_expression_type(*operand);
        let operand_strings = match operand_type {
            Some(t) => get_operand_strings(t),
            None => vec![TString::general()],
        };

        current_strings = concat_string_lists(current_strings, &operand_strings, threshold);
    }

    if current_strings.is_empty() {
        return get_string();
    }

    if current_strings.iter().all(|s| !s.is_literal_origin()) {
        let is_non_empty = current_strings.iter().any(|s| s.is_non_empty);
        let is_truthy = current_strings.iter().all(|s| s.is_truthy);
        let is_lowercase = current_strings.iter().all(|s| s.is_lowercase);

        return get_string_with_props(false, is_truthy, is_non_empty, is_lowercase);
    }

    TUnion::new(current_strings.into_iter().map(|string| TAtomic::Scalar(TScalar::String(string))).collect())
}

/// Concatenates two lists of string types, producing all possible combinations.
fn concat_string_lists(left_strings: Vec<TString>, right_strings: &[TString], threshold: u16) -> Vec<TString> {
    let has_literals =
        left_strings.iter().any(|s| s.literal.is_some()) || right_strings.iter().any(|s| s.literal.is_some());

    if !has_literals || left_strings.len().saturating_mul(right_strings.len()) > threshold as usize {
        let is_non_empty = left_strings.iter().any(|s| s.is_non_empty) || right_strings.iter().any(|s| s.is_non_empty);
        let is_truthy = left_strings.iter().all(|s| s.is_truthy) || right_strings.iter().all(|s| s.is_truthy);
        let is_lowercase = left_strings.iter().all(|s| s.is_lowercase) && right_strings.iter().all(|s| s.is_lowercase);

        return vec![TString::general_with_props(false, is_truthy, is_non_empty, is_lowercase)];
    }

    let mut result_strings = Vec::new();
    for left_string in left_strings {
        for right_string in right_strings {
            let mut resulting_string = TString::general();
            resulting_string.is_non_empty = left_string.is_non_empty || right_string.is_non_empty;
            resulting_string.is_truthy = left_string.is_truthy || right_string.is_truthy;
            resulting_string.is_lowercase = left_string.is_lowercase && right_string.is_lowercase;
            resulting_string.literal = match (&left_string.literal, &right_string.literal) {
                (Some(TStringLiteral::Value(left_literal)), Some(TStringLiteral::Value(right_literal))) => {
                    result_strings.push(TString::known_literal(concat_atom!(left_literal, right_literal)));

                    continue;
                }
                (Some(_), Some(_)) => Some(TStringLiteral::Unspecified),
                _ => None,
            };

            result_strings.push(resulting_string);
        }
    }

    result_strings
}

#[inline]
fn get_operand_strings(operand_type: &TUnion) -> Vec<TString> {
    let mut operand_strings = vec![];

    for operand_atomic_type in operand_type.types.as_ref() {
        match operand_atomic_type {
            TAtomic::Array(_) => {
                operand_strings.push(TString::known_literal(atom("Array")));

                continue;
            }
            TAtomic::Never | TAtomic::Null | TAtomic::Void => {
                operand_strings.push(TString::known_literal(atom("")));

                continue;
            }
            TAtomic::Resource(_) => {
                operand_strings.push(TString::general_with_props(false, true, true, false));

                continue;
            }
            _ => {}
        }

        let TAtomic::Scalar(operand_scalar) = operand_atomic_type else {
            operand_strings.push(TString::general_with_props(false, false, false, false));

            continue;
        };

        match operand_scalar {
            TScalar::Bool(boolean) => {
                if boolean.is_true() {
                    operand_strings.push(TString::known_literal(atom("1")));
                } else if boolean.is_false() {
                    operand_strings.push(TString::known_literal(atom("")));
                } else {
                    operand_strings.push(TString::known_literal(atom("1")));
                    operand_strings.push(TString::known_literal(atom("")));
                }
            }
            TScalar::Integer(tint) => {
                if let Some(v) = tint.get_literal_value() {
                    operand_strings.push(TString::known_literal(i64_atom(v)));
                } else {
                    operand_strings.push(TString::general_with_props(true, false, false, true));
                }
            }
            TScalar::Float(tfloat) => {
                if let Some(v) = tfloat.get_literal_value() {
                    operand_strings.push(TString::known_literal(f64_atom(v)));
                } else {
                    operand_strings.push(TString::general_with_props(true, false, false, true));
                }
            }
            TScalar::String(operand_string) => {
                operand_strings.push(operand_string.clone());
            }
            TScalar::ClassLikeString(tclass_like_string) => {
                if let Some(id) = tclass_like_string.literal_value() {
                    operand_strings.push(TString::known_literal(id));
                } else {
                    operand_strings.push(TString::general_with_props(false, true, true, false));
                }
            }
            _ => {
                operand_strings.push(TString::general());
            }
        }
    }

    operand_strings
}
