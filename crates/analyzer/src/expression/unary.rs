use std::collections::BTreeMap;
use std::ops::Add;
use std::ops::Sub;
use std::rc::Rc;

use mago_atom::atom;
use mago_atom::empty_atom;
use mago_atom::f64_atom;
use mago_atom::i64_atom;
use mago_codex::get_class_like;
use mago_codex::get_declaring_method_identifier;
use mago_codex::get_method;
use mago_codex::identifier::method::MethodIdentifier;
use mago_codex::ttype::atomic::TAtomic;
use mago_codex::ttype::atomic::array::TArray;
use mago_codex::ttype::atomic::array::keyed::TKeyedArray;
use mago_codex::ttype::atomic::array::list::TList;
use mago_codex::ttype::atomic::mixed::TMixed;
use mago_codex::ttype::atomic::object::TObject;
use mago_codex::ttype::atomic::scalar::TScalar;
use mago_codex::ttype::atomic::scalar::int::TInteger;
use mago_codex::ttype::atomic::scalar::string::TStringLiteral;
use mago_codex::ttype::combiner::combine;
use mago_codex::ttype::union::TUnion;
use mago_codex::ttype::*;
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
use crate::expression::assignment::assign_to_expression;
use crate::expression::call::method_call::analyze_implicit_method_call;
use crate::utils::expression::get_expression_id;
use crate::utils::php_emulation::str_increment;
use crate::utils::php_emulation::str_is_numeric;

impl<'ast, 'arena> Analyzable<'ast, 'arena> for UnaryPrefix<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let is_negation = matches!(self.operator, UnaryPrefixOperator::Not(_));
        let is_variable_reference = matches!(self.operator, UnaryPrefixOperator::Reference(_))
            && matches!(self.operand, Expression::Variable(Variable::Direct(_)));

        let was_in_negation = block_context.inside_negation;
        let was_in_variable_reference = block_context.inside_variable_reference;
        let was_in_general_use = block_context.inside_general_use;
        block_context.inside_general_use = true;
        block_context.inside_variable_reference = is_variable_reference;
        block_context.inside_negation = if is_negation { !was_in_negation } else { was_in_negation };

        self.operand.analyze(context, block_context, artifacts)?;

        block_context.inside_negation = was_in_negation;
        block_context.inside_general_use = was_in_general_use;
        block_context.inside_variable_reference = was_in_variable_reference;

        let operand_type = artifacts.get_rc_expression_type(&self.operand).cloned();
        match self.operator {
            // operators that always retain the type of the operand
            UnaryPrefixOperator::Reference(_) => {
                let mut referenced_type = operand_type.map(|t| t.as_ref().clone()).unwrap_or_else(get_mixed);
                referenced_type.by_reference = true;

                artifacts.set_rc_expression_type(self, Rc::new(referenced_type));
            }
            UnaryPrefixOperator::ErrorControl(_) | UnaryPrefixOperator::Plus(_) => {
                if let Some(operand_type) = operand_type {
                    artifacts.set_rc_expression_type(self, operand_type);
                } else {
                    artifacts.set_expression_type(self, get_mixed());
                }
            }
            UnaryPrefixOperator::BitwiseNot(_) => {
                if let Some(operand_type) = operand_type {
                    artifacts.set_rc_expression_type(self, operand_type);
                } else {
                    artifacts.set_expression_type(self, get_mixed());
                }
            }
            UnaryPrefixOperator::Not(_) => {
                let resulting_type = match operand_type {
                    Some(t) if t.is_always_truthy() => get_false(),
                    Some(t) if t.is_always_falsy() => get_true(),
                    _ => get_bool(),
                };

                artifacts.set_expression_type(self, resulting_type);
            }
            UnaryPrefixOperator::Negation(_) => {
                let mut resulting_types = vec![];
                for operand_part in operand_type.as_ref().map(|o| o.types.as_ref()).unwrap_or_default() {
                    match operand_part {
                        TAtomic::Null | TAtomic::Void => {
                            // -null results in int(0).
                            resulting_types.push(TAtomic::Scalar(TScalar::literal_int(0)));
                        }
                        TAtomic::Scalar(scalar) => match scalar {
                            TScalar::Bool(boolean) => match boolean.value {
                                None => {
                                    resulting_types.push(TAtomic::Scalar(TScalar::literal_int(0)));
                                    resulting_types.push(TAtomic::Scalar(TScalar::literal_int(-1)));
                                }
                                Some(true) => {
                                    resulting_types.push(TAtomic::Scalar(TScalar::literal_int(-1)));
                                }
                                Some(false) => {
                                    resulting_types.push(TAtomic::Scalar(TScalar::literal_int(0)));
                                }
                            },
                            TScalar::Integer(integer) => {
                                resulting_types.push(TAtomic::Scalar(TScalar::Integer(integer.negated())));
                            }
                            TScalar::Float(float) => match float.value {
                                Some(value) => {
                                    resulting_types.push(TAtomic::Scalar(TScalar::literal_float(-value.0)));
                                }
                                None => {
                                    resulting_types.push(TAtomic::Scalar(TScalar::float()));
                                }
                            },
                            _ => {
                                resulting_types.push(TAtomic::Scalar(TScalar::int()));
                                resulting_types.push(TAtomic::Scalar(TScalar::float()));
                            }
                        },
                        TAtomic::GenericParameter(parameter) => {
                            if parameter.constraint.is_int_or_float() {
                                resulting_types.push(TAtomic::GenericParameter(parameter.clone()));
                            }
                        }
                        _ => {
                            // TODO(azjezz): we should handle more types here.
                        }
                    }
                }

                if resulting_types.is_empty() {
                    resulting_types.push(TAtomic::Scalar(TScalar::int()));
                    resulting_types.push(TAtomic::Scalar(TScalar::float()));
                }

                let resulting_type = TUnion::from_vec(combine(resulting_types, context.codebase, false));

                artifacts.set_expression_type(self, resulting_type);
            }
            UnaryPrefixOperator::PreIncrement(_) => {
                let resulting_type = increment_operand(context, block_context, artifacts, self.operand, self.span())?;
                artifacts.set_expression_type(self, resulting_type);
            }
            UnaryPrefixOperator::PreDecrement(_) => {
                let resulting_type = decrement_operand(context, block_context, artifacts, self.operand, self.span())?;
                artifacts.set_expression_type(self, resulting_type);
            }
            UnaryPrefixOperator::IntCast(_, _) | UnaryPrefixOperator::IntegerCast(_, _) => {
                let resulting_type = match operand_type {
                    Some(t) => {
                        // we intentionally do not report redundant casts here, as
                        // what we think is an integer, might be a float at runtime.
                        //
                        // Example:
                        //
                        // ```
                        // function factorial(int $number): int {
                        //     if ($number <= 1) {
                        //         return 1;
                        //     }
                        //
                        //     return $number * factorial($number - 1);
                        // }
                        // ```
                        //
                        // While this function looks like it always returns an integer,
                        // it could result in a float ( via overflow ) at runtime.
                        //
                        // While currently we do not report overflows, we should allow the
                        // user to explicitly cast the result to an integer.
                        //
                        // ---
                        //
                        // if t.is_int() {
                        //     report_redundant_type_cast(&self.operator, self, &t, context);
                        // }
                        cast_type_to_int(&t, context)
                    }
                    None => get_int(),
                };

                artifacts.set_expression_type(self, resulting_type);
            }
            UnaryPrefixOperator::ArrayCast(_, _) => {
                let resulting_type = match operand_type {
                    Some(t) => {
                        if t.is_array() {
                            report_redundant_type_cast(&self.operator, self, &t, context);
                        }

                        cast_type_to_array(&t, context, self)
                    }
                    None => wrap_atomic(TAtomic::Array(TArray::Keyed(TKeyedArray::new()))),
                };

                artifacts.set_expression_type(self, resulting_type);
            }
            UnaryPrefixOperator::BoolCast(_, _) | UnaryPrefixOperator::BooleanCast(_, _) => {
                let resulting_type = match operand_type {
                    Some(t) => {
                        if t.is_bool() {
                            report_redundant_type_cast(&self.operator, self, &t, context);
                        }

                        cast_type_to_bool(&t, context, self)
                    }
                    None => get_bool(),
                };

                artifacts.set_expression_type(self, resulting_type);
            }
            UnaryPrefixOperator::DoubleCast(_, _)
            | UnaryPrefixOperator::RealCast(_, _)
            | UnaryPrefixOperator::FloatCast(_, _) => {
                let resulting_type = match operand_type {
                    Some(t) => {
                        if t.is_float() {
                            report_redundant_type_cast(&self.operator, self, &t, context);
                        }

                        cast_type_to_float(&t, context, self)
                    }
                    None => get_float(),
                };

                artifacts.set_expression_type(self, resulting_type);
            }
            UnaryPrefixOperator::ObjectCast(_, _) => {
                let resulting_type = match operand_type {
                    Some(t) => {
                        if t.is_objecty() {
                            report_redundant_type_cast(&self.operator, self, &t, context);
                        }

                        cast_type_to_object(&t, context, self)
                    }
                    None => get_object(),
                };

                artifacts.set_expression_type(self, resulting_type);
            }
            UnaryPrefixOperator::BinaryCast(_, _) | UnaryPrefixOperator::StringCast(_, _) => {
                let resulting_type = match operand_type {
                    Some(t) => {
                        if t.is_any_string() {
                            report_redundant_type_cast(&self.operator, self, &t, context);
                        }

                        cast_type_to_string(&t, context, block_context, artifacts, self.span())?
                    }
                    None => get_string(),
                };

                artifacts.set_expression_type(self, resulting_type);
            }
            UnaryPrefixOperator::UnsetCast(_, _) => {
                // unsupported, but we can ignore it.
            }
            UnaryPrefixOperator::VoidCast(_, _) => {
                artifacts.set_expression_type(self, get_void());
            }
        }

        Ok(())
    }
}

impl<'ast, 'arena> Analyzable<'ast, 'arena> for UnaryPostfix<'arena> {
    fn analyze<'ctx>(
        &'ast self,
        context: &mut Context<'ctx, 'arena>,
        block_context: &mut BlockContext<'ctx>,
        artifacts: &mut AnalysisArtifacts,
    ) -> Result<(), AnalysisError> {
        let was_in_general_use = block_context.inside_general_use;
        block_context.inside_general_use = true;
        self.operand.analyze(context, block_context, artifacts)?;
        block_context.inside_general_use = was_in_general_use;

        match self.operator {
            UnaryPostfixOperator::PostIncrement(span) => {
                increment_operand(context, block_context, artifacts, self.operand, span)?;
            }
            UnaryPostfixOperator::PostDecrement(span) => {
                decrement_operand(context, block_context, artifacts, self.operand, span)?;
            }
        };

        if let Some(operand_type) = artifacts.get_rc_expression_type(&self.operand).cloned() {
            artifacts.set_rc_expression_type(self, operand_type);
        }

        Ok(())
    }
}

/// Increments the given operand and returns its type after incrementing.
///
/// If the operand is a variable-like entity, the function attempts to assign the incremented value back to it.
///
/// # Arguments
///
/// * `context` - The analysis context.
/// * `block_context` - Mutable context for the current code block.
/// * `artifacts` - Mutable store for analysis results.
/// * `operand` - The expression AST node representing the operand to be incremented.
/// * `operation_span` - The span of the entire increment operation (e.g., `++$x` or `$x++`).
///
/// # Returns
///
/// An `TUnion` representing the type of the operand *after* the increment operation.
///
/// Returns `mixed|any` if the operand's type cannot be determined or if a fatal error occurs.
fn increment_operand<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    operand: &Expression<'arena>,
    operation_span: Span,
) -> Result<TUnion, AnalysisError> {
    let Some(operand_type) = artifacts.get_expression_type(operand) else {
        return Ok(get_mixed());
    };

    let mut possibilities = vec![];
    for operand_atomic_type in operand_type.types.as_ref() {
        match operand_atomic_type {
            TAtomic::Scalar(scalar) => match scalar {
                TScalar::Integer(int_scalar) => {
                    let resulting_integer = int_scalar.add(TInteger::literal(1));

                    if block_context.inside_loop {
                        possibilities.push(TAtomic::Scalar(TScalar::Integer(match resulting_integer {
                            TInteger::Literal(value) => TInteger::From(value),
                            integer => integer,
                        })));
                    } else {
                        possibilities.push(TAtomic::Scalar(TScalar::Integer(resulting_integer)));
                    }
                }
                TScalar::Float(float_scalar) => {
                    if block_context.inside_loop {
                        // Do not set literal value in loop context.
                        possibilities.push(TAtomic::Scalar(TScalar::float()));
                    } else if let Some(value) = float_scalar.value {
                        possibilities.push(TAtomic::Scalar(TScalar::literal_float(value.0 + 1.0)));
                    } else {
                        possibilities.push(TAtomic::Scalar(TScalar::float()));
                    }
                }
                TScalar::Numeric => {
                    possibilities.push(TAtomic::Scalar(TScalar::numeric()));
                }
                TScalar::String(string_scalar) => {
                    if block_context.inside_loop {
                        possibilities.push(TAtomic::Scalar(TScalar::String(string_scalar.without_literal())));
                    } else if let Some(TStringLiteral::Value(string_val)) = &string_scalar.literal {
                        if string_val.is_empty() {
                            possibilities.push(TAtomic::Scalar(TScalar::literal_string(atom("1"))));
                        } else if str_is_numeric(string_val) {
                            let mut negative = false;
                            let value = if let Some(value) = string_val.strip_prefix("+") {
                                value
                            } else if let Some(value) = string_val.strip_prefix("-") {
                                negative = true;
                                value
                            } else {
                                string_val
                            };

                            let value = value.trim_start_matches("0");
                            if value.is_empty() {
                                possibilities.push(TAtomic::Scalar(TScalar::literal_int(1)));
                            } else if let Ok(value) = value.parse::<i64>() {
                                possibilities.push(TAtomic::Scalar(TScalar::literal_int(if negative {
                                    value.wrapping_sub(1)
                                } else {
                                    value.wrapping_add(1)
                                })));
                            } else if let Ok(value) = value.parse::<f64>() {
                                possibilities.push(TAtomic::Scalar(TScalar::literal_float(if negative {
                                    value - 1.0
                                } else {
                                    value + 1.0
                                })));
                            } else {
                                possibilities.push(TAtomic::Scalar(TScalar::int()));
                                possibilities.push(TAtomic::Scalar(TScalar::float()));
                            }
                        } else if let Some(incremented) = str_increment(string_val) {
                            possibilities.push(TAtomic::Scalar(TScalar::literal_string(atom(&incremented))));
                        } else {
                            possibilities
                                .push(TAtomic::Scalar(TScalar::String(string_scalar.with_unspecified_literal())));
                        }
                    } else {
                        // Non-literal string: result is string, but value unknown.
                        possibilities.push(TAtomic::Scalar(TScalar::String(string_scalar.clone())));
                    }
                }
                TScalar::Bool(boolean_scalar) => {
                    // PHP: ++true remains true, ++false remains false. The type remains bool.
                    possibilities.push(TAtomic::Scalar(TScalar::Bool(*boolean_scalar)));
                }
                TScalar::ClassLikeString(_) => {
                    // Incrementing a class name string is highly unusual.
                    context.collector.report_with_code(
                        IssueCode::InvalidOperand,
                        Issue::warning(
                            "Incrementing a class-string is unusual and likely a bug."
                        )
                        .with_annotation(Annotation::primary(operand.span()).with_message("Class-string incremented"))
                        .with_note("PHP will treat the class name as a regular string for increment, which might not be the intended behavior.")
                        .with_help("Verify if this operation is intended. If string manipulation is needed, ensure it's on a regular string variable."),
                    );

                    // Result is a generic string as the incremented class name is unknown.
                    possibilities.push(TAtomic::Scalar(TScalar::string()));
                }
                TScalar::Generic | TScalar::ArrayKey => {
                    context.collector.report_with_code(
                        IssueCode::InvalidOperand,
                        Issue::warning(format!(
                            "Incrementing a generic scalar type (`{}`). This may not yield the expected result.",
                            scalar.get_id()
                        ))
                        .with_annotation(Annotation::primary(operand.span()).with_message(format!("Type is `{}`", scalar.get_id())))
                        .with_help("Ensure the generic type resolves to a numeric type or string suitable for increment, or provide a more specific type."),
                    );

                    possibilities.push(TAtomic::Scalar(scalar.clone()));
                }
            },
            TAtomic::Null | TAtomic::Void => {
                // ++null results in int(1).
                possibilities.push(TAtomic::Scalar(TScalar::literal_int(1)));
            }
            TAtomic::Callable(callable) => {
                if callable.get_signature().is_none_or(|signature| signature.is_closure()) {
                    context.collector.report_with_code(
                        IssueCode::InvalidOperand,
                        Issue::error("Cannot increment a closure.")
                            .with_annotation(Annotation::primary(operand.span()).with_message("This is a closure"))
                            .with_note("PHP throws a TypeError when attempting to increment a closure object."),
                    );

                    possibilities.push(TAtomic::Never);
                } else {
                    context.collector.report_with_code(
                            IssueCode::InvalidOperand,
                            Issue::error(format!(
                                "Cannot reliably increment callable of type `{}`.",
                                callable.get_id()
                            ))
                            .with_annotation(Annotation::primary(operand.span()).with_message("Invalid callable type for increment"))
                            .with_note("Incrementing array callables or invocable objects without specific overload behavior leads to errors."),
                        );

                    possibilities.push(TAtomic::Mixed(TMixed::new()));
                }
            }
            _ => {
                let type_name = operand_atomic_type.get_id();
                context.collector.report_with_code(
                        IssueCode::InvalidOperand,
                        Issue::error(format!(
                            "Cannot increment value of type `{type_name}`."
                        ))
                        .with_annotation(Annotation::primary(operand.span()).with_message(format!("Type `{type_name}` cannot be incremented")))
                        .with_note(match operand_atomic_type {
                            TAtomic::Array(_) => "Incrementing an array results in a `TypeError` exception.",
                            TAtomic::Object(_) => "Incrementing an object without operator overloading support results in a `TypeError` exception.",
                            TAtomic::Resource(_) => "Incrementing a resource results in a `TypeError` exception.",
                            TAtomic::Never => "An expression of type `never` does not produce a value to decrement.",
                            _ => "This type is not suitable for decrement operations."
                        })
                        .with_help("Ensure the operand is a number or a string suitable for incrementing."),
                    );

                possibilities.push(TAtomic::Mixed(TMixed::new()));
            }
        }
    }

    let resulting_type_union = if possibilities.is_empty() {
        get_mixed()
    } else {
        TUnion::from_vec(combine(possibilities, context.codebase, false))
    };

    let operand_id = get_expression_id(
        operand,
        block_context.scope.get_class_like_name(),
        context.resolved_names,
        Some(context.codebase),
    );

    let successful = assign_to_expression(
        context,
        block_context,
        artifacts,
        operand,
        operand_id,
        None,
        resulting_type_union.clone(),
        false,
    )?;

    if !successful {
        context.collector.report_with_code(
            IssueCode::InvalidOperand,
            Issue::error("Failed to assign incremented value to operand.")
                .with_annotation(Annotation::primary(operation_span).with_message("Failed to assign incremented value"))
                .with_note("The operand's type may not support assignment after incrementing.")
                .with_help("Ensure the operand is a variable-like entity that can be assigned a new value."),
        );
    }

    Ok(resulting_type_union)
}

/// Decrements the given operand and returns its type after decrementing.
///
/// If the operand is a variable-like entity (e.g., a direct variable), the function
/// attempts to assign the decremented value back to it.
///
/// This function reports issues for types that are problematic or behave unexpectedly
/// when decremented.
///
/// # Arguments
///
/// * `context` - The analysis context.
/// * `block_context` - Mutable context for the current code block.
/// * `artifacts` - Mutable store for analysis results.
/// * `operand` - The expression AST node representing the operand to be decremented.
/// * `operation_span` - The span of the entire decrement operation (e.g., `--$x` or `$x--`).
///
/// # Returns
///
/// An `TUnion` representing the type of the operand *after* the decrement operation.
fn decrement_operand<'ctx, 'arena>(
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    operand: &Expression<'arena>,
    operation_span: Span,
) -> Result<TUnion, AnalysisError> {
    // Changed return to Result for consistency
    let Some(operand_type) = artifacts.get_expression_type(operand) else {
        return Ok(get_mixed());
    };

    let mut possibilities = vec![];

    for operand_atomic_type in operand_type.types.as_ref() {
        match operand_atomic_type {
            TAtomic::Scalar(scalar) => {
                match scalar {
                    TScalar::Integer(int_scalar) => {
                        if block_context.inside_loop {
                            // Do not set literal value in loop context.
                            // TODO(azjez): we should set the type to a range here.
                            possibilities.push(TAtomic::Scalar(TScalar::int()));
                        } else {
                            possibilities.push(TAtomic::Scalar(TScalar::Integer(int_scalar.sub(TInteger::literal(1)))));
                        }
                    }
                    TScalar::Float(float_scalar) => {
                        if let Some(value) = float_scalar.value
                            && !block_context.inside_loop
                        {
                            possibilities.push(TAtomic::Scalar(TScalar::literal_float(value.0 - 1.0)));
                        } else {
                            possibilities.push(TAtomic::Scalar(TScalar::float()));
                        }
                    }
                    TScalar::Numeric => {
                        possibilities.push(TAtomic::Scalar(TScalar::numeric()));
                    }
                    TScalar::String(string_scalar) => {
                        if block_context.inside_loop {
                            possibilities.push(TAtomic::Scalar(TScalar::String(string_scalar.without_literal())));
                        } else if !string_scalar.is_numeric {
                            context.collector.report_with_code(
                                IssueCode::InvalidOperand,
                                Issue::error("Cannot decrement a non-numeric string.")
                                    .with_annotation(
                                        Annotation::primary(operand.span())
                                            .with_message("Invalid string for decrement."),
                                    )
                                    .with_note("String decrement supports numeric strings only.")
                                    .with_help("Decrementing a non-numeric string has no effects on the string value."),
                            );

                            possibilities.push(TAtomic::Scalar(TScalar::String(string_scalar.clone())));
                        } else if let Some(TStringLiteral::Value(string_val)) = &string_scalar.literal {
                            if string_val.is_empty() {
                                possibilities.push(TAtomic::Scalar(TScalar::literal_int(-1)));
                            } else {
                                let mut negative = false;
                                let value = if let Some(value) = string_val.strip_prefix("+") {
                                    value
                                } else if let Some(value) = string_val.strip_prefix("-") {
                                    negative = true;
                                    value
                                } else {
                                    string_val
                                };

                                let value = value.trim_start_matches("0");
                                if value.is_empty() {
                                    possibilities.push(TAtomic::Scalar(TScalar::literal_int(-1)));
                                } else if let Ok(value) = value.parse::<i64>() {
                                    possibilities.push(TAtomic::Scalar(TScalar::literal_int(if negative {
                                        value.wrapping_add(1)
                                    } else {
                                        value.wrapping_sub(1)
                                    })));
                                } else if let Ok(value) = value.parse::<f64>() {
                                    possibilities.push(TAtomic::Scalar(TScalar::literal_float(if negative {
                                        value + 1.0
                                    } else {
                                        value - 1.0
                                    })));
                                } else {
                                    possibilities.push(TAtomic::Scalar(TScalar::int()));
                                    possibilities.push(TAtomic::Scalar(TScalar::float()));
                                }
                            }
                        } else {
                            // Non-literal string: result is string, but value unknown.
                            possibilities.push(TAtomic::Scalar(TScalar::String(string_scalar.clone())));
                        }
                    }
                    TScalar::Bool(boolean_scalar) => {
                        possibilities.push(TAtomic::Scalar(TScalar::Bool(*boolean_scalar)));
                    }
                    TScalar::ClassLikeString(_) => {
                        // Incrementing a class name string is highly unusual.
                        context.collector.report_with_code(
                            IssueCode::InvalidOperand,
                            Issue::warning(
                                "Decrementing a class-string is unusual and likely a bug."
                            )
                                .with_annotation(Annotation::primary(operand.span()).with_message("Class-string decremented"))
                                .with_note("PHP will treat the class name as a regular string for decrement, which might not be the intended behavior.")
                                .with_help("Verify if this operation is intended. If string manipulation is needed, ensure it's on a regular string variable."),
                        );

                        // Result is a generic string as the incremented class name is unknown.
                        possibilities.push(TAtomic::Scalar(TScalar::string()));
                    }
                    TScalar::Generic | TScalar::ArrayKey => {
                        context.collector.report_with_code(
                            IssueCode::InvalidOperand,
                            Issue::warning(format!(
                                "Decrementing a generic scalar type (`{}`). This may not yield the expected result.",
                                scalar.get_id()
                            ))
                                .with_annotation(Annotation::primary(operand.span()).with_message(format!("Type is `{}`", scalar.get_id())))
                                .with_help("Ensure the generic type resolves to a numeric type or string suitable for increment, or provide a more specific type."),
                        );

                        possibilities.push(TAtomic::Scalar(scalar.clone()));
                    }
                }
            }
            TAtomic::Null | TAtomic::Void => {
                // --null results in `null`
                possibilities.push(TAtomic::Null);
            }
            TAtomic::Callable(callable) => {
                if callable.get_signature().is_none_or(|signature| signature.is_closure()) {
                    context.collector.report_with_code(
                        IssueCode::InvalidOperand,
                        Issue::error("Cannot decrement a closure.")
                            .with_annotation(Annotation::primary(operand.span()).with_message("This is a closure"))
                            .with_note("PHP throws a TypeError when attempting to decrement a closure object."),
                    );

                    possibilities.push(TAtomic::Never);
                } else {
                    context.collector.report_with_code(
                        IssueCode::InvalidOperand,
                        Issue::error(format!(
                            "Cannot reliably decrement callable of type `{}`.",
                            callable.get_id()
                        ))
                            .with_annotation(Annotation::primary(operand.span()).with_message("Invalid callable type for decrement"))
                            .with_note("Decrementing array callables or invocable objects without specific overload behavior leads to errors."),
                    );

                    possibilities.push(TAtomic::Mixed(TMixed::new()));
                }
            }
            _ => {
                let type_name = operand_atomic_type.get_id();
                context.collector.report_with_code(
                        IssueCode::InvalidOperand,
                        Issue::error(format!(
                            "Cannot decrement value of type `{type_name}`."
                        ))
                            .with_annotation(Annotation::primary(operand.span()).with_message(format!("Type `{type_name}` cannot be decremented")))
                            .with_note(match operand_atomic_type {
                                TAtomic::Array(_) => "Decrementing an array results in a `TypeError` exception.",
                                TAtomic::Object(_) => "Decrementing an object without operator overloading support results in a `TypeError` exception.",
                                TAtomic::Resource(_) => "Decrementing a resource results in a `TypeError` exception.",
                                TAtomic::Never => "An expression of type `never` does not produce a value to decrement.",
                                _ => "This type is not suitable for decrement operations."
                            })
                            .with_help("Ensure the operand is a number or a string suitable for decrementing."),
                    );

                possibilities.push(TAtomic::Mixed(TMixed::new()));
            }
        }
    }

    let resulting_type_union = if possibilities.is_empty() {
        get_mixed()
    } else {
        TUnion::from_vec(combine(possibilities, context.codebase, false))
    };

    let operand_id = get_expression_id(
        operand,
        block_context.scope.get_class_like_name(),
        context.resolved_names,
        Some(context.codebase),
    );

    let successful = assign_to_expression(
        context,
        block_context,
        artifacts,
        operand,
        operand_id,
        None,
        resulting_type_union.clone(),
        false,
    )?;

    if !successful {
        context.collector.report_with_code(
            IssueCode::InvalidOperand,
            Issue::error("Failed to assign decremented value to operand.")
                .with_annotation(Annotation::primary(operation_span).with_message("Failed to assign decremented value"))
                .with_note("The operand's type may not support assignment after decrementing.")
                .with_help("Ensure the operand is a variable-like entity that can be assigned a new value."),
        );
    }

    Ok(resulting_type_union)
}

fn report_redundant_type_cast<'ctx, 'ast, 'arena>(
    cast_operator: &'ast UnaryPrefixOperator,
    expression: &'ast UnaryPrefix<'arena>,
    known_type: &TUnion,
    context: &mut Context<'ctx, 'arena>,
) {
    context.collector.report_with_code(
        IssueCode::RedundantCast,
        Issue::help(format!("Redundant cast to `{}`: the expression already has this type.", cast_operator.as_str()))
            .with_annotation(
                Annotation::primary(expression.operand.span())
                    .with_message(format!("This expression already has type `{}`.", known_type.get_id())),
            )
            .with_note("Casting a value to a type it already possesses has no effect.")
            .with_help(format!("Remove the redundant `{}` cast.", cast_operator.as_str())),
    );
}

fn cast_type_to_array<'ctx, 'ast, 'arena>(
    operand_type: &TUnion,
    context: &mut Context<'ctx, 'arena>,
    cast_expression: &'ast UnaryPrefix<'arena>,
) -> TUnion {
    if operand_type.is_never() {
        context.collector.report_with_code(
            IssueCode::InvalidTypeCast,
            Issue::error("Cannot cast type `never` to `array`.")
                .with_annotation(
                    Annotation::primary(cast_expression.span()).with_message("Invalid cast from `never` to `array`"),
                )
                .with_note("An expression of type `never` does not produce a value and thus cannot be cast.")
                .with_help("Ensure the expression being cast can complete normally."),
        );

        return get_never();
    }

    let mut resulting_array_atomics = Vec::new();
    let mut reported_object_warning = false;

    for atomic_type in operand_type.types.as_ref() {
        match atomic_type {
            TAtomic::Array(arr) => {
                // If it's already an array, it remains as is.
                resulting_array_atomics.push(TAtomic::Array(arr.clone()));
            }
            TAtomic::Null | TAtomic::Void => {
                // null or void cast to an empty array.
                context.collector.report_with_code(
                    IssueCode::InvalidTypeCast,
                    Issue::error(format!(
                        "Casting type `{}` to `array` will produce an empty array.",
                        atomic_type.get_id()
                    ))
                    .with_annotation(
                        Annotation::primary(cast_expression.span())
                            .with_message(format!("Invalid cast from `{}` to `array`", atomic_type.get_id()))
                    )
                    .with_note("Casting `null` or `void` to `array` produces an empty array. This is often a sign of an uninitialized variable or logic error.")
                    .with_help("Initialize the variable with an array or handle the `null`/`void` case explicitly before casting."),
                );

                resulting_array_atomics.push(TAtomic::Array(TArray::Keyed(TKeyedArray::new())));
            }
            TAtomic::Scalar(_) | TAtomic::Resource(_) | TAtomic::Callable(_) => {
                // Scalars (int, float, string, bool) become a list with one element at key 0.
                let mut scalar_list = TList::new(Box::new(get_never()));
                scalar_list.known_count = Some(1);
                scalar_list.non_empty = true;
                scalar_list.known_elements =
                    Some(BTreeMap::from_iter([(0, (false, wrap_atomic(atomic_type.clone())))]));

                resulting_array_atomics.push(TAtomic::Array(TArray::List(scalar_list)));
            }
            TAtomic::Object(casted_object) => {
                let is_stdclass = casted_object.get_name().is_some_and(|name| {
                    // Check if the object is stdClass
                    name.eq_ignore_ascii_case("stdClass")
                });

                // Object to array: properties become key-value pairs.
                // Keys are strings (property names), values are mixed (property values).
                // stdClass is a special case where we do not report a warning.
                if !reported_object_warning && !is_stdclass {
                    context.collector.report_with_code(
                        IssueCode::InvalidTypeCast,
                        Issue::warning(format!(
                            "Object of type `{}` cast to `array`. Property visibility (public, protected, private) affects the resulting array.",
                            atomic_type.get_id()
                        ))
                        .with_annotation(Annotation::primary(cast_expression.span()).with_message("Object cast to array"))
                        .with_note("Casting an object to an array converts its properties to key-value pairs. Private/protected properties will have mangled keys.")
                        .with_help("For reliable object-to-array conversion, consider implementing a `toArray()` method or using specific library functions that handle visibility and structure as intended."),
                    );

                    reported_object_warning = true;
                }

                // TODO(azjezz): we can do better here
                // we can lookup the object and get the properties, and return
                // a keyed array with the properties
                let mut obj_array = TKeyedArray::new();
                obj_array.parameters = Some((Box::new(get_string()), Box::new(get_mixed())));

                resulting_array_atomics.push(TAtomic::Array(TArray::Keyed(obj_array)));
            }
            TAtomic::Mixed(_) => {
                // Mixed to array: result is array<array-key, mixed>.
                if !reported_object_warning {
                    // Reuse flag to avoid spamming for mixed as well
                    context.collector.report_with_code(
                        IssueCode::InvalidTypeCast,
                        Issue::warning("Casting `mixed` to `array`.".to_string())
                            .with_annotation(Annotation::primary(cast_expression.operand.span()).with_message("This expression has type `mixed`"))
                            .with_note("The structure and element types of the resulting array cannot be determined statically when casting `mixed`.")
                            .with_help("Ensure the value is an array or use type checks before casting if a specific array structure is expected."),
                    );
                    reported_object_warning = true;
                }

                resulting_array_atomics.push(TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
                    Box::new(get_arraykey()),
                    Box::new(get_mixed()),
                ))));
            }
            _ => {
                if !reported_object_warning {
                    context.collector.report_with_code(
                        IssueCode::InvalidTypeCast,
                        Issue::error(format!(
                            "Cannot reliably cast type `{}` to `array`.",
                            atomic_type.get_id()
                        ))
                        .with_annotation(Annotation::primary(cast_expression.span())
                            .with_message(format!("Unclear cast from `{}` to `array`", atomic_type.get_id())))
                        .with_help("Ensure the expression being cast has a defined conversion to array (e.g., scalar, null, object, or already an array)."),
                    );

                    reported_object_warning = true;
                }

                // Fallback to a generic array type if cast is ambiguous
                resulting_array_atomics.push(TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(
                    Box::new(get_arraykey()),
                    Box::new(get_mixed()),
                ))));
            }
        }
    }

    // Combine all potential array types resulting from the cast.
    TUnion::from_vec(combine(resulting_array_atomics, context.codebase, false))
}

fn cast_type_to_bool<'ctx, 'ast, 'arena>(
    operand_type: &TUnion,
    context: &mut Context<'ctx, 'arena>,
    cast_expression: &'ast UnaryPrefix<'arena>,
) -> TUnion {
    if operand_type.is_never() {
        return get_never();
    }

    let mut truthy_counts = 0;
    let mut falsy_counts = 0;
    let mut has_non_literal_bool = false;

    for atomic_type in operand_type.types.as_ref() {
        if atomic_type.is_truthy() {
            truthy_counts += 1;

            continue;
        }

        if atomic_type.is_falsy() {
            falsy_counts += 1;

            continue;
        }

        if atomic_type.is_mixed() {
            context.collector.report_with_code(
                IssueCode::MixedOperand,
                Issue::warning("Casting `mixed` to `bool`.".to_string()) // Warning, as it's a valid cast but loses type info
                    .with_annotation(Annotation::primary(cast_expression.operand.span()).with_message("This expression has type `mixed`"))
                    .with_note("The truthiness of `mixed` cannot be determined statically. The result will be a general `bool`.")
                    .with_help("Consider adding type assertions or checks if a more specific boolean outcome is expected."),
            );
        }

        has_non_literal_bool = true;
    }

    if !has_non_literal_bool {
        if truthy_counts > 0 && falsy_counts == 0 {
            return get_true();
        }

        if falsy_counts > 0 && truthy_counts == 0 {
            return get_false();
        }
    }

    get_bool()
}

fn cast_type_to_float<'ctx, 'ast, 'arena>(
    operand_type: &TUnion,
    context: &mut Context<'ctx, 'arena>,
    cast_expression: &'ast UnaryPrefix<'arena>,
) -> TUnion {
    if operand_type.is_never() {
        return get_never();
    }

    let mut resulting_float_atomics = Vec::new();
    let mut reported_error_for_object = false;

    for atomic_type in operand_type.types.as_ref() {
        match atomic_type {
            TAtomic::Null | TAtomic::Void => resulting_float_atomics.push(TAtomic::Scalar(TScalar::literal_float(0.0))),
            TAtomic::Scalar(scalar) => {
                match scalar {
                    TScalar::Bool(b) => {
                        if let Some(val) = b.value {
                            resulting_float_atomics.push(TAtomic::Scalar(TScalar::literal_float(if val {
                                1.0
                            } else {
                                0.0
                            })));
                        } else {
                            resulting_float_atomics.push(TAtomic::Scalar(TScalar::literal_float(0.0)));
                            resulting_float_atomics.push(TAtomic::Scalar(TScalar::literal_float(1.0)));
                        }
                    }
                    TScalar::Integer(i) => {
                        if let Some(val) = i.get_literal_value() {
                            resulting_float_atomics.push(TAtomic::Scalar(TScalar::literal_float(val as f64)));
                        } else {
                            return get_float();
                        }
                    }
                    TScalar::Float(f) => resulting_float_atomics.push(TAtomic::Scalar(TScalar::Float(*f))),
                    TScalar::String(s) => {
                        if let Some(TStringLiteral::Value(val)) = &s.literal {
                            let mut num_str = String::new();
                            for ch in val.chars() {
                                if ch.is_ascii_digit() || ch == '.' || (num_str.is_empty() && (ch == '+' || ch == '-'))
                                {
                                    num_str.push(ch);
                                } else {
                                    break;
                                }
                            }

                            if let Ok(f_val) = num_str.parse::<f64>() {
                                resulting_float_atomics.push(TAtomic::Scalar(TScalar::literal_float(f_val)));
                            } else {
                                resulting_float_atomics.push(TAtomic::Scalar(TScalar::literal_float(0.0)));
                            }

                            if !val.is_empty() && num_str.is_empty() && val != "0" {
                                context.collector.report_with_code(
                                    IssueCode::InvalidTypeCast,
                                    Issue::warning(format!("String `{val}` implicitly cast to float `0.0`."))
                                        .with_annotation(Annotation::primary(cast_expression.operand.span()).with_message("Non-numeric string cast to float"))
                                        .with_help("Explicitly cast or ensure string is numeric if float conversion is intended."),
                                );
                            }
                        } else {
                            if !s.is_numeric {
                                context.collector.report_with_code(
                                    IssueCode::InvalidTypeCast,
                                    Issue::warning(format!("Non numeric string of type `{}` implicitly cast to `float`.", s.get_id()))
                                        .with_annotation(Annotation::primary(cast_expression.operand.span()).with_message("String cast to float"))
                                        .with_note("PHP will attempt to parse a leading numeric value; otherwise, it results in `0.0`. This can be error-prone.")
                                        .with_help("Ensure the string is numeric or use explicit parsing if a specific float value is expected."),
                                );
                            }

                            return get_float();
                        }
                    }
                    TScalar::ClassLikeString(_) => {
                        context.collector.report_with_code(
                            IssueCode::InvalidTypeCast,
                            Issue::warning("Class-like string implicitly cast to float `0.0`.".to_string())
                                .with_annotation(
                                    Annotation::primary(cast_expression.operand.span())
                                        .with_message("Class-string cast to float"),
                                )
                                .with_help("Casting class names to float is usually not intended."),
                        );

                        resulting_float_atomics.push(TAtomic::Scalar(TScalar::literal_float(0.0)));
                    }
                    _ => {
                        return get_float();
                    }
                }
            }
            TAtomic::Array(arr) => {
                if arr.is_non_empty() {
                    resulting_float_atomics.push(TAtomic::Scalar(TScalar::literal_float(1.0)));
                } else {
                    resulting_float_atomics.push(TAtomic::Scalar(TScalar::literal_float(0.0)));
                    resulting_float_atomics.push(TAtomic::Scalar(TScalar::literal_float(1.0)));
                }
            }
            TAtomic::Object(_) => {
                if !reported_error_for_object {
                    context.collector.report_with_code(
                        IssueCode::InvalidTypeCast,
                        Issue::error(format!(
                            "Object of type `{}` cannot be cast to `float`. PHP will attempt this and produce `1.0` after an error.",
                            atomic_type.get_id()
                        ))
                        .with_annotation(Annotation::primary(cast_expression.span()).with_message("Invalid cast from object to float"))
                        .with_note("This operation will raise an `E_WARNING` and result in `1.0`.")
                        .with_help("Avoid casting objects directly to float. Extract a numeric property or implement a specific conversion method."),
                    );

                    reported_error_for_object = true;
                }

                resulting_float_atomics.push(TAtomic::Scalar(TScalar::literal_float(1.0)));
            }
            TAtomic::Resource(_) => {
                context.collector.report_with_code(
                    IssueCode::InvalidTypeCast,
                    Issue::warning("Implicit conversion of `resource` to `float` (its ID).".to_string())
                        .with_annotation(Annotation::primary(cast_expression.operand.span()).with_message("Resource ID used as float"))
                        .with_note("PHP converts resources to their numeric ID when cast to float. This is rarely the intended behavior.")
                        .with_help("Avoid casting resources directly to float. Use resource-specific functions to get relevant numeric data if needed."),
                );

                return get_float();
            }
            TAtomic::Never => return get_never(),
            TAtomic::Mixed(_) => {
                context.collector.report_with_code(
                    IssueCode::InvalidTypeCast,
                    Issue::warning("Casting `mixed` to `float`.".to_string())
                        .with_annotation(Annotation::primary(cast_expression.operand.span()).with_message("This expression has type `mixed`"))
                        .with_note("The float value of `mixed` cannot be determined statically. The result will be a general `float`.")
                        .with_help("Consider adding type assertions or checks if a more specific float outcome is expected."),
                );

                return get_float();
            }
            _ => return get_float(), // Other types default to general float
        }
    }

    if resulting_float_atomics.is_empty() {
        return get_float();
    }

    TUnion::from_vec(combine(resulting_float_atomics, context.codebase, false))
}

fn cast_type_to_int(operand_type: &TUnion, context: &mut Context<'_, '_>) -> TUnion {
    let mut possibilities = vec![];
    for t in operand_type.types.as_ref() {
        let possible = match t {
            TAtomic::Null | TAtomic::Void => TAtomic::Scalar(TScalar::literal_int(0)),
            TAtomic::Array(array) => {
                if !array.is_non_empty() {
                    possibilities.push(TAtomic::Scalar(TScalar::literal_int(0)));
                }

                TAtomic::Scalar(TScalar::literal_int(1))
            }
            TAtomic::Object(_) => TAtomic::Scalar(TScalar::literal_int(1)),
            TAtomic::Callable(callable) if callable.get_signature().is_none_or(|signature| signature.is_closure()) => {
                TAtomic::Scalar(TScalar::literal_int(1))
            }
            TAtomic::Never => return get_never(),
            TAtomic::Scalar(scalar) => match scalar {
                TScalar::Numeric | TScalar::ArrayKey => {
                    return get_int();
                }
                TScalar::Bool(bool_scalar) => match bool_scalar.value {
                    Some(true) => TAtomic::Scalar(TScalar::literal_int(1)),
                    Some(false) => TAtomic::Scalar(TScalar::literal_int(0)),
                    None => {
                        possibilities.push(TAtomic::Scalar(TScalar::literal_int(0)));

                        TAtomic::Scalar(TScalar::literal_int(1))
                    }
                },
                TScalar::Integer(int_scalar) => match int_scalar.get_literal_value() {
                    Some(i) => TAtomic::Scalar(TScalar::literal_int(i)),
                    None => {
                        return get_int();
                    }
                },
                TScalar::Float(float_scalar) => match float_scalar.value {
                    Some(f) => {
                        if f.is_nan() {
                            return get_int();
                        }

                        TAtomic::Scalar(TScalar::literal_int(f.0 as i64))
                    }
                    None => {
                        return get_int();
                    }
                },
                TScalar::String(string_scalar) => match &string_scalar.literal {
                    Some(TStringLiteral::Value(string_literal)) => {
                        if let Ok(value) = string_literal.parse::<i64>() {
                            TAtomic::Scalar(TScalar::literal_int(value))
                        } else {
                            return get_int();
                        }
                    }
                    _ => {
                        return get_int();
                    }
                },
                TScalar::ClassLikeString(_) => TAtomic::Scalar(TScalar::literal_int(0)),
                TScalar::Generic => {
                    return get_int();
                }
            },
            _ => return get_int(),
        };

        possibilities.push(possible);
    }

    TUnion::from_vec(combine(possibilities, context.codebase, false))
}

fn cast_type_to_object<'ctx, 'ast, 'arena>(
    operand_type: &TUnion,
    context: &mut Context<'ctx, 'arena>,
    cast_expression: &'ast UnaryPrefix<'arena>,
) -> TUnion {
    let mut possibilities = vec![];
    for t in operand_type.types.as_ref() {
        match t {
            TAtomic::Resource(_) => {
                context.collector.report_with_code(
                    IssueCode::InvalidTypeCast,
                    Issue::error("Cannot cast type `resource` to `object`.")
                        .with_annotation(
                            Annotation::primary(cast_expression.span())
                                .with_message("Invalid cast from `resource` to `object`."),
                        )
                        .with_note(
                            "Casting a `resource` to `object` is disallowed and will throw an `Error` at runtime.",
                        )
                        .with_help("Remove the cast or ensure the expression being cast is not a `resource`."),
                );

                return get_never();
            }
            TAtomic::Never => return get_never(),

            TAtomic::Callable(callable) if callable.get_signature().is_none_or(|signature| signature.is_closure()) => {
                possibilities.push(t.clone());
            }
            TAtomic::Object(_) => {
                possibilities.push(t.clone());
            }
            _ => {}
        }
    }

    if possibilities.is_empty() {
        return get_named_object(atom("stdClass"), None);
    }

    TUnion::from_vec(combine(possibilities, context.codebase, false))
}

pub fn cast_type_to_string<'ctx, 'arena>(
    operand_type: &TUnion,
    context: &mut Context<'ctx, 'arena>,
    block_context: &mut BlockContext<'ctx>,
    artifacts: &mut AnalysisArtifacts,
    expression_span: Span,
) -> Result<TUnion, AnalysisError> {
    let mut possibilities = vec![];
    for t in operand_type.types.as_ref() {
        let possible = match t {
            TAtomic::Scalar(scalar) => match scalar {
                TScalar::Bool(boolean) => {
                    if boolean.is_true() {
                        TAtomic::Scalar(TScalar::literal_string(atom("1")))
                    } else if boolean.is_false() {
                        TAtomic::Scalar(TScalar::literal_string(empty_atom()))
                    } else {
                        possibilities.push(TAtomic::Scalar(TScalar::literal_string(empty_atom())));

                        TAtomic::Scalar(TScalar::literal_string(atom("1")))
                    }
                }
                TScalar::Integer(integer) => {
                    if let Some(value) = integer.get_literal_value() {
                        TAtomic::Scalar(TScalar::literal_string(i64_atom(value)))
                    } else {
                        TAtomic::Scalar(TScalar::numeric_string())
                    }
                }
                TScalar::Float(float) => {
                    if let Some(value) = float.get_literal_value() {
                        TAtomic::Scalar(TScalar::literal_string(f64_atom(value)))
                    } else {
                        TAtomic::Scalar(TScalar::numeric_string())
                    }
                }
                TScalar::Numeric => TAtomic::Scalar(TScalar::numeric_string()),
                TScalar::String(string) => TAtomic::Scalar(TScalar::String(string.clone())),
                TScalar::ClassLikeString(class_string) => {
                    if let Some(value) = class_string.literal_value() {
                        TAtomic::Scalar(TScalar::literal_string(value))
                    } else {
                        TAtomic::Scalar(TScalar::non_empty_string())
                    }
                }
                _ => TAtomic::Scalar(TScalar::string()),
            },
            TAtomic::Callable(callable) => {
                return if callable.get_signature().is_none_or(|signature| signature.is_closure()) {
                    context.collector.report_with_code(
                        IssueCode::InvalidTypeCast,
                        Issue::error("Cannot cast type `Closure` to `string`.")
                            .with_annotation(
                                Annotation::primary(expression_span.span())
                                    .with_message("Invalid cast from `Closure` to `string`."),
                            )
                            .with_note(
                                "Casting a `Closure` to `string` is disallowed and will throw an `Error` at runtime.",
                            )
                            .with_help("Remove the cast or ensure the expression being cast is not a `Closure`."),
                    );

                    Ok(get_never())
                } else {
                    context.collector.report_with_code(
                        IssueCode::InvalidTypeCast,
                        Issue::warning(format!(
                            "Cannot reliably cast callable of type `{}` to `string`.",
                            callable.get_id()
                        ))
                            .with_annotation(
                                Annotation::primary(expression_span.span())
                                    .with_message("Invalid cast from callable to string"),
                            )
                            .with_note("Casting a callable to `string` is ambiguous and may not yield a meaningful result.")
                            .with_help("Ensure the callable can be represented as a string or use a specific callable type that guarantees string representation."),
                    );

                    Ok(get_string())
                };
            }
            TAtomic::Object(object) => {
                let class_like_name = match object {
                    TObject::Any => {
                        context.collector.report_with_code(
                            IssueCode::InvalidTypeCast,
                            Issue::error("Cannot reliably cast generic `object` to `string`.")
                            .with_annotation(
                                Annotation::primary(expression_span.span()).with_message("Casting generic `object` to `string`")
                            )
                            .with_note(
                                "The object might implement `Stringable` or have a `__toString()` method, but this cannot be determined statically for a generic `object` type."
                            )
                            .with_note(
                                "If the object is not stringable at runtime, this cast will cause a fatal error."
                            )
                            .with_help(
                                "Ensure the object is stringable before casting, use a more specific object type, or avoid the cast."
                            ),
                        );

                        return Ok(get_string());
                    }
                    TObject::Named(named_object) => named_object.get_name(),
                    TObject::Enum(enum_instance) => enum_instance.get_name(),
                };

                let Some(class_metadata) = get_class_like(context.codebase, &class_like_name) else {
                    context.collector.report_with_code(
                        IssueCode::InvalidTypeCast,
                        Issue::error(format!(
                            "Cannot cast object of type `{class_like_name}` to `string` because the class does not exist.",
                        ))
                        .with_annotation(
                            Annotation::primary(expression_span.span())
                                .with_message(format!("Class `{class_like_name}` does not exist."))
                        )
                        .with_note("Casting an object to `string` requires the class to exist and implement `Stringable` or have a `__toString()` method.")
                        .with_help("Ensure the class exists or avoid casting this object type to `string`."),
                    );

                    return Ok(get_string());
                };

                if class_metadata.kind.is_enum() {
                    context.collector.report_with_code(
                        IssueCode::InvalidTypeCast,
                        Issue::error(format!(
                            "Cannot cast enum instance of type `{}` to `string`.",
                            object.get_id(),
                        ))
                        .with_annotation(
                            Annotation::primary(expression_span.span())
                                .with_message(format!("Enum `{class_like_name}` cannot be cast to `string`."))
                        )
                        .with_note("Casting an enum instance to `string` is not allowed and will throw a fatal error at runtime.")
                        .with_help("Use the enum's name or value instead, or avoid casting the enum instance to `string`."),
                    );

                    return Ok(get_string());
                }

                let to_string_method_id = atom("__toString");
                let declaring_method_id = get_declaring_method_identifier(
                    context.codebase,
                    &MethodIdentifier::new(class_metadata.original_name, to_string_method_id),
                );

                let Some(to_string_metadata) = get_method(
                    context.codebase,
                    declaring_method_id.get_class_name(),
                    declaring_method_id.get_method_name(),
                ) else {
                    let class_name_str = class_metadata.original_name;

                    context.collector.report_with_code(
                        IssueCode::InvalidTypeCast,
                        Issue::error(format!(
                            "Cannot cast object of type `{class_name_str}` to `string` because it does not implement `Stringable`.",
                        ))
                        .with_code(IssueCode::InvalidTypeCast)
                        .with_annotation(
                            Annotation::primary(expression_span.span())
                                .with_message(format!("`{class_name_str}` does not implement `Stringable`."))
                        )
                        .with_note(
                            "Casting an object to `string` requires it to have a `__toString()` method (implicitly via `Stringable` interface)."
                        )
                        .with_note(
                            "This cast will cause a fatal error at runtime."
                        )
                        .with_help(
                            format!(
                                "Implement the `Stringable` interface (or add a `__toString()` method) on class `{class_name_str}` or avoid casting this object type to `string`.",
                            )
                        ),
                    );

                    return Ok(get_string());
                };

                return analyze_implicit_method_call(
                    context,
                    block_context,
                    artifacts,
                    object,
                    declaring_method_id,
                    class_metadata,
                    to_string_metadata,
                    None,
                    expression_span,
                );
            }
            TAtomic::Array(_) => {
                context.collector.report_with_code(
                    IssueCode::ArrayToStringConversion,
                    Issue::warning(
                        "Casting `array` to `string` is deprecated and produces the literal string 'Array'."
                    )
                    .with_annotation(
                        Annotation::primary(expression_span.span()).with_message("Casting `array` to `string`.")
                    )
                    .with_note(
                        "PHP raises an `E_WARNING` (or `E_NOTICE` in older versions) when an array is cast to a string, resulting in the literal string 'Array'."
                    )
                    .with_help(
                        "Do not cast arrays to strings directly. Use functions like `implode()`, `json_encode()`, or loop through the array to create a string representation."
                    ),
                );

                TAtomic::Scalar(TScalar::literal_string(atom("Array")))
            }
            TAtomic::Null | TAtomic::Void => TAtomic::Scalar(TScalar::literal_string(atom(""))),
            TAtomic::Resource(_) => TAtomic::Scalar(TScalar::non_empty_string()),
            TAtomic::Never => return Ok(get_never()),
            _ => return Ok(get_string()),
        };

        possibilities.push(possible);
    }

    Ok(TUnion::from_vec(combine(possibilities, context.codebase, false)))
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::test_analysis;

    test_analysis! {
        name = unary_increment_decrement_operators,
        code = indoc! {r#"
            <?php

            /**
             * @param 2 $_a
             * @param 2 $_b
             * @param 125 $_c
             */
            function example(int $_a, int $_b, int $_c): void
            {
            }

            $arr = ['a' => '123', 'b' => 2, 'c' => -1];
            $arr['a'] = '';
            $arr['b'] = null;
            $arr['c'] = 123;
            $arr['a']++;
            $arr['b']++;
            $arr['c']++;
            $arr['a']--;
            $arr['b']--;
            $arr['c']--;
            $arr['a'] = $arr['a']--;
            $arr['c'] = $arr['c']--;
            $arr['b'] = $arr['b']--;
            $arr['a'] = $arr['a']++;
            $arr['b'] = $arr['b']++;
            $arr['c'] = $arr['c']++;
            $arr['a'] = $arr['a'] + 1;
            $arr['b'] = $arr['b'] + 1;
            $arr['c'] = $arr['c'] + 1;
            $arr['a'] = --$arr['a'];
            $arr['b'] = --$arr['b'];
            $arr['c'] = --$arr['c'];
            $arr['a'] = $arr['a'] + 1;
            $arr['b'] = $arr['b'] + 1;
            $arr['c'] = $arr['c'] + 1;
            $arr['a'] = ++$arr['a'];
            $arr['b'] = ++$arr['b'];
            $arr['c'] = ++$arr['c'];

            example($arr['a'], $arr['b'], $arr['c']);
        "#}
    }

    test_analysis! {
        name = implicit_to_string_call,
        code = indoc! {r#"
            <?php

            /**
             * @param non-empty-string $command
             * @param non-empty-list<non-empty-string> $args
             * @param non-empty-string $cwd
             */
            function shell_execute(string $command, array $args, string $cwd = ''): void {
                echo "Executing command: $command [" . $args[0] . ", ..] in directory: $cwd\n";
            }

            final class CheckedOutRepository {
                /** @param non-empty-string $path */
                private function __construct(
                    private readonly string $path,
                ) {}

                /** @param non-empty-string $path */
                public static function fromPath(string $path): self {
                    return new self($path);
                }

                /** @return non-empty-string */
                public function __toString(): string {
                    return $this->path;
                }
            }

            final class GetVersionCollectionFromGitRepository {
                private CheckedOutRepository $repoPath;

                public function __construct(CheckedOutRepository $repoPath) {
                    $this->repoPath = $repoPath;
                }

                /** @param non-empty-string $tagName */
                public function makeTag(string $tagName): void {
                    $path = (string) $this->repoPath;

                    shell_execute('git', ['tag', $tagName], $path);
                }
            }
        "#}
    }

    test_analysis! {
        name = negate_integer_ranges,
        code = indoc! {r#"
            <?php

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
                public function __construct(
                    public int $hours,
                    public int $minutes = 0,
                    public int $seconds = 0,
                    public int $nanoseconds = 0,
                ) {}

                /**
                 * @return Duration
                 */
                public function invert(): Duration
                {
                    return new Duration(-$this->hours, -$this->minutes, -$this->seconds, -$this->nanoseconds);
                }
            }
        "#}
    }

    test_analysis! {
        name = cast_stdclass_to_array,
        code = indoc! {r#"
            <?php

            class stdClass
            {
                // built-in
            }

            /** @return array<string, mixed> */
            function example(stdClass $obj): array
            {
                return (array) $obj;
            }
        "#}
    }
}
