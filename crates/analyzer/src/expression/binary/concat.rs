use std::rc::Rc;

use mago_atom::atom;
use mago_atom::concat_atom;
use mago_atom::f64_atom;
use mago_atom::i64_atom;
use mago_codex::get_method_identifier;
use mago_codex::method_identifier_exists;
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
use mago_syntax::ast::*;

use crate::analyzable::Analyzable;
use crate::artifacts::AnalysisArtifacts;
use crate::artifacts::get_expression_range;
use crate::code::IssueCode;
use crate::context::Context;
use crate::context::block::BlockContext;
use crate::error::AnalysisError;

#[inline]
pub fn analyze_string_concat_operation<'ctx, 'arena>(
    binary: &Binary<'arena>,
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
) -> Result<(), AnalysisError> {
    binary.lhs.analyze(context, block_context, artifacts)?;
    binary.rhs.analyze(context, block_context, artifacts)?;

    analyze_string_concat_operand(context, artifacts, binary.lhs, "Left")?;
    analyze_string_concat_operand(context, artifacts, binary.rhs, "Right")?;

    let result_type = concat_operands(binary.lhs, binary.rhs, artifacts);

    artifacts.expression_types.insert(get_expression_range(binary), Rc::new(result_type));

    Ok(())
}

#[inline]
fn analyze_string_concat_operand<'ctx, 'ast, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    artifacts: &mut AnalysisArtifacts,
    operand: &'ast Expression<'arena>,
    side: &'static str,
) -> Result<(), AnalysisError> {
    let Some(operand_type) = artifacts.get_expression_type(operand) else {
        return Ok(());
    };

    if operand_type.is_null() {
        context.collector.report_with_code(
            IssueCode::NullOperand,
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
        context.collector.report_with_code(
            IssueCode::FalseOperand,
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
        context.collector.report_with_code(
            IssueCode::PossiblyNullOperand,
            Issue::warning(format!(
                "Possibly null {} operand used in string concatenation (type `{}`).",
                side.to_ascii_lowercase(),
                operand_type.get_id()
            ))
            .with_annotation(Annotation::primary(operand.span()).with_message("This might be `null`"))
            .with_note("If this operand is `null` at runtime, it will be implicitly converted to an empty string `''`.")
            .with_help("Ensure the operand is non-null before concatenation using checks or assertions, or explicitly cast to string."),
        );
    }

    if operand_type.is_falsable() && !operand_type.ignore_falsable_issues {
        context.collector.report_with_code(
            IssueCode::PossiblyFalseOperand,
            Issue::warning(format!(
                "Possibly false {} operand used in string concatenation (type `{}`).",
                side.to_ascii_lowercase(),
                operand_type.get_id()
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

    for operand_atomic_type in operand_type.types.as_ref() {
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
                        context.collector.report_with_code(
                            IssueCode::InvalidOperand,
                            Issue::error(format!(
                                "Invalid {} operand: template parameter `{}` constraint `{}` is not compatible with string concatenation.",
                                side.to_ascii_lowercase(),
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

                let method_identifier = get_method_identifier(class_like_name, "__toString");

                if method_identifier_exists(context.codebase, &method_identifier) {
                    current_atomic_is_valid = true;

                    context.collector.report_with_code(
                        IssueCode::ImplicitToStringCast,
                        Issue::warning(format!(
                            "Implicit conversion to `string` for {} operand via `{}`.",
                            side.to_ascii_lowercase(),
                            method_identifier.as_string()
                        ))
                        .with_annotation(Annotation::primary(operand.span())
                            .with_message(format!("Object implicitly converted using `{}`", method_identifier.as_string()))
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
                                side.to_ascii_lowercase(),
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
                    context.collector.report_with_code(
                        IssueCode::ArrayToStringConversion,
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
                context.collector.report_with_code(
                    IssueCode::ImplicitResourceToStringCast,
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
                    context.collector.report_with_code(
                        IssueCode::MixedOperand,
                        Issue::error(format!(
                            "Invalid {} operand: type `{}` cannot be reliably used in string concatenation.",
                            side.to_ascii_lowercase(),
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
        context.collector.report_with_code(
            IssueCode::InvalidOperand,
            Issue::error(format!(
                "Invalid type `{}` for {} operand in string concatenation.",
                operand_type.get_id(), side.to_ascii_lowercase()
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

fn concat_operands<'ast, 'arena>(
    left: &'ast Expression<'arena>,
    right: &'ast Expression<'arena>,
    artifacts: &mut AnalysisArtifacts,
) -> TUnion {
    let left_type = artifacts.get_expression_type(left);
    let right_type = artifacts.get_expression_type(right);

    let (left_strings, right_strings) = match (left_type, right_type) {
        (Some(left_type), Some(right_type)) => (get_operand_strings(left_type), get_operand_strings(right_type)),
        (Some(left_type), None) => (get_operand_strings(left_type), vec![TString::general()]),
        (None, Some(right_type)) => (vec![TString::general()], get_operand_strings(right_type)),
        (None, None) => {
            return get_string();
        }
    };

    // Determine if we can take the fast path. The fast path is possible only if there are
    // no specific literal values that need to be concatenated at runtime.
    let has_literals =
        left_strings.iter().any(|s| s.literal.is_some()) || right_strings.iter().any(|s| s.literal.is_some());

    if !has_literals {
        let is_non_empty = left_strings.iter().any(|s| s.is_non_empty) || right_strings.iter().any(|s| s.is_non_empty);
        let is_truthy = left_strings.iter().all(|s| s.is_truthy) || right_strings.iter().all(|s| s.is_truthy);
        let is_lowercase = left_strings.iter().all(|s| s.is_lowercase) && right_strings.iter().all(|s| s.is_lowercase);

        return get_string_with_props(false, is_truthy, is_non_empty, is_lowercase);
    }

    let mut result_strings = vec![];
    for left_string in left_strings {
        for right_string in &right_strings {
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

    if result_strings.is_empty() {
        return get_string();
    }

    if result_strings.iter().all(|s| !s.is_literal_origin()) {
        let is_non_empty = result_strings.iter().any(|s| s.is_non_empty);
        let is_truthy = result_strings.iter().all(|s| s.is_truthy);
        let is_lowercase = result_strings.iter().all(|s| s.is_lowercase);

        return get_string_with_props(false, is_truthy, is_non_empty, is_lowercase);
    }

    TUnion::new(result_strings.into_iter().map(|string| TAtomic::Scalar(TScalar::String(string))).collect())
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
