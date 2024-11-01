use fennec_ast::*;
use fennec_interner::ThreadedInterner;
use fennec_reflection::r#type::kind::*;
use fennec_reflection::r#type::TypeReflection;
use fennec_semantics::Semantics;
use fennec_span::HasSpan;
use ordered_float::OrderedFloat;

pub fn infere<'i, 'ast>(
    interner: &'i ThreadedInterner,
    semantics: &'ast Semantics,
    expression: &'ast Expression,
) -> Option<TypeReflection> {
    let kind = infere_kind(interner, semantics, expression)?;

    Some(TypeReflection { kind, inferred: true, span: expression.span() })
}

fn infere_kind<'i, 'ast>(
    interner: &'i ThreadedInterner,
    semantics: &'ast Semantics,
    expression: &'ast Expression,
) -> Option<TypeKind> {
    match &expression {
        Expression::Parenthesized(parenthesized) => infere_kind(interner, semantics, &parenthesized.expression),
        Expression::Referenced(referenced) => infere_kind(interner, semantics, &referenced.expression),
        Expression::Suppressed(suppressed) => infere_kind(interner, semantics, &suppressed.expression),
        Expression::Literal(literal) => Some(match &literal {
            Literal::String(string) => {
                let value = interner.lookup(string.value);

                value_string_kind(interner.intern(&value[1..value.len() - 1]))
            }
            Literal::Integer(integer) => {
                if let Some(value) = integer.value {
                    if value > i64::MAX as u64 {
                        integer_kind()
                    } else {
                        // we can safely cast `value` to an `i64`
                        value_integer_kind(value as i64)
                    }
                } else {
                    integer_kind()
                }
            }
            Literal::Float(_) => float_kind(),
            Literal::True(_) => true_kind(),
            Literal::False(_) => false_kind(),
            Literal::Null(_) => null_kind(),
        }),
        Expression::CompositeString(_) => Some(string_kind()),
        Expression::ArithmeticOperation(arithmetic_operation) => match arithmetic_operation.as_ref() {
            ArithmeticOperation::Prefix(arithmetic_prefix_operation) => {
                let value_kind = infere_kind(interner, semantics, &arithmetic_prefix_operation.value);

                match value_kind {
                    Some(TypeKind::Value(ValueTypeKind::Integer { value })) => {
                        match &arithmetic_prefix_operation.operator {
                            ArithmeticPrefixOperator::Increment(_) => {
                                let new_value = value.wrapping_add(1);
                                Some(value_integer_kind(new_value))
                            }
                            ArithmeticPrefixOperator::Decrement(_) => {
                                let new_value = value.wrapping_sub(1);
                                Some(value_integer_kind(new_value))
                            }
                            ArithmeticPrefixOperator::Plus(_) => Some(value_integer_kind(value)),
                            ArithmeticPrefixOperator::Minus(_) => Some(value_integer_kind(-value)),
                        }
                    }
                    Some(TypeKind::Value(ValueTypeKind::Float { value })) => {
                        match &arithmetic_prefix_operation.operator {
                            ArithmeticPrefixOperator::Increment(_) => {
                                let new_value = value + 1.0;
                                Some(value_float_kind(new_value))
                            }
                            ArithmeticPrefixOperator::Decrement(_) => {
                                let new_value = value - 1.0;
                                Some(value_float_kind(new_value))
                            }
                            ArithmeticPrefixOperator::Plus(_) => Some(value_float_kind(value)),
                            ArithmeticPrefixOperator::Minus(_) => Some(value_float_kind(-value)),
                        }
                    }
                    Some(TypeKind::Scalar(ScalarTypeKind::Integer)) => match &arithmetic_prefix_operation.operator {
                        ArithmeticPrefixOperator::Increment(_) | ArithmeticPrefixOperator::Decrement(_) => {
                            Some(integer_kind())
                        }
                        ArithmeticPrefixOperator::Plus(_) | ArithmeticPrefixOperator::Minus(_) => Some(integer_kind()),
                    },
                    Some(TypeKind::Scalar(ScalarTypeKind::Float)) => Some(float_kind()),
                    _ => None,
                }
            }
            ArithmeticOperation::Infix(arithmetic_infix_operation) => {
                let lhs_kind = infere_kind(interner, semantics, &arithmetic_infix_operation.lhs);
                let rhs_kind = infere_kind(interner, semantics, &arithmetic_infix_operation.rhs);

                match (&lhs_kind, &rhs_kind) {
                    (
                        Some(TypeKind::Value(ValueTypeKind::Integer { value: lhs_value })),
                        Some(TypeKind::Value(ValueTypeKind::Integer { value: rhs_value })),
                    ) => {
                        match &arithmetic_infix_operation.operator {
                            ArithmeticInfixOperator::Addition(_) => {
                                let result = lhs_value.wrapping_add(*rhs_value);
                                Some(value_integer_kind(result))
                            }
                            ArithmeticInfixOperator::Subtraction(_) => {
                                let result = lhs_value.wrapping_sub(*rhs_value);
                                Some(value_integer_kind(result))
                            }
                            ArithmeticInfixOperator::Multiplication(_) => {
                                let result = lhs_value.wrapping_mul(*rhs_value);
                                Some(value_integer_kind(result))
                            }
                            ArithmeticInfixOperator::Division(_) => {
                                if *rhs_value != 0 {
                                    if lhs_value % rhs_value == 0 {
                                        // Division is exact, result is integer
                                        let result = lhs_value / rhs_value;
                                        Some(value_integer_kind(result))
                                    } else {
                                        // Division results in float
                                        let result = (*lhs_value as f64) / (*rhs_value as f64);
                                        Some(value_float_kind(OrderedFloat(result)))
                                    }
                                } else {
                                    // Division by zero; in PHP, this throws, resulting in `never`
                                    Some(never_kind())
                                }
                            }
                            ArithmeticInfixOperator::Modulo(_) => {
                                if *rhs_value != 0 {
                                    let result = lhs_value % rhs_value;
                                    Some(value_integer_kind(result))
                                } else {
                                    // Division by zero; in PHP, this throws, resulting in `never`
                                    Some(never_kind())
                                }
                            }
                            ArithmeticInfixOperator::Exponentiation(_) => {
                                // Exponentiation of integers
                                let base = *lhs_value as f64;
                                let exponent = *rhs_value as f64;
                                let result = base.powf(exponent);

                                if result.fract() == 0.0 && result >= i64::MIN as f64 && result <= i64::MAX as f64 {
                                    // Result is an integer
                                    Some(value_integer_kind(result as i64))
                                } else {
                                    // Result is a float
                                    Some(value_float_kind(OrderedFloat(result)))
                                }
                            }
                        }
                    }
                    // Both operands are numeric literals (integer or float)
                    (Some(lhs_value_kind), Some(rhs_value_kind))
                        if is_numeric_value_kind(lhs_value_kind) && is_numeric_value_kind(rhs_value_kind) =>
                    {
                        let lhs_value = extract_numeric_value(lhs_value_kind);
                        let rhs_value = extract_numeric_value(rhs_value_kind);

                        match (lhs_value, rhs_value) {
                            (Some(lhs_num), Some(rhs_num)) => {
                                let result = match &arithmetic_infix_operation.operator {
                                    ArithmeticInfixOperator::Addition(_) => lhs_num + rhs_num,
                                    ArithmeticInfixOperator::Subtraction(_) => lhs_num - rhs_num,
                                    ArithmeticInfixOperator::Multiplication(_) => lhs_num * rhs_num,
                                    ArithmeticInfixOperator::Division(_) => {
                                        if rhs_num != 0.0 {
                                            lhs_num / rhs_num
                                        } else {
                                            return Some(never_kind()); // Division by zero
                                        }
                                    }
                                    ArithmeticInfixOperator::Modulo(_) => {
                                        if rhs_num != 0.0 {
                                            lhs_num % rhs_num
                                        } else {
                                            return Some(never_kind()); // Division by zero
                                        }
                                    }
                                    ArithmeticInfixOperator::Exponentiation(_) => OrderedFloat(lhs_num.powf(*rhs_num)),
                                };

                                Some(value_float_kind(result))
                            }
                            _ => Some(float_kind()),
                        }
                    }
                    // One or both operands are not literals
                    _ => infer_numeric_operation_type(
                        lhs_kind.clone(),
                        rhs_kind.clone(),
                        &arithmetic_infix_operation.operator,
                    ),
                }
            }
            ArithmeticOperation::Postfix(arithmetic_postfix_operation) => {
                let value_kind = infere_kind(interner, semantics, &arithmetic_postfix_operation.value);

                match value_kind {
                    Some(TypeKind::Value(ValueTypeKind::Integer { value })) => {
                        match &arithmetic_postfix_operation.operator {
                            ArithmeticPostfixOperator::Increment(_) => {
                                // Postfix increment: value is used before increment
                                Some(value_integer_kind(value))
                            }
                            ArithmeticPostfixOperator::Decrement(_) => {
                                // Postfix decrement: value is used before decrement
                                Some(value_integer_kind(value))
                            }
                        }
                    }
                    Some(TypeKind::Value(ValueTypeKind::Float { value })) => {
                        match &arithmetic_postfix_operation.operator {
                            ArithmeticPostfixOperator::Increment(_) => Some(value_float_kind(value)),
                            ArithmeticPostfixOperator::Decrement(_) => Some(value_float_kind(value)),
                        }
                    }
                    Some(TypeKind::Scalar(ScalarTypeKind::Integer)) => Some(integer_kind()),
                    Some(TypeKind::Scalar(ScalarTypeKind::Float)) => Some(float_kind()),
                    _ => None,
                }
            }
        },
        Expression::AssignmentOperation(assignment_operation) => {
            infere_kind(interner, semantics, &assignment_operation.rhs)
        }
        Expression::BitwiseOperation(_) => Some(integer_kind()),
        Expression::ComparisonOperation(comparison_operation) => Some(match &comparison_operation.operator {
            ComparisonOperator::Spaceship(_) => integer_kind(),
            _ => bool_kind(),
        }),
        Expression::LogicalOperation(_) => Some(bool_kind()),
        Expression::CastOperation(cast_operation) => Some(match &cast_operation.operator {
            CastOperator::Array(_, _) => array_kind(array_key_kind(), mixed_kind()),
            CastOperator::Bool(_, _) | CastOperator::Boolean(_, _) => bool_kind(),
            CastOperator::Double(_, _) | CastOperator::Real(_, _) | CastOperator::Float(_, _) => float_kind(),
            CastOperator::Int(_, _) | CastOperator::Integer(_, _) => integer_kind(),
            CastOperator::Object(_, _) => any_object_kind(),
            CastOperator::Unset(_, _) => null_kind(),
            CastOperator::String(_, _) | CastOperator::Binary(_, _) => string_kind(),
        }),
        Expression::ConcatOperation(_) => Some(string_kind()),
        Expression::InstanceofOperation(_) => Some(bool_kind()),
        Expression::Array(_) => Some(array_kind(array_key_kind(), mixed_kind())),
        Expression::LegacyArray(_) => Some(array_kind(array_key_kind(), mixed_kind())),
        Expression::AnonymousClass(_) => Some(any_object_kind()),
        // TODO: improve this
        Expression::Closure(_) => Some(closure_kind(vec![callable_parameter(mixed_kind(), false, true)], mixed_kind())),
        // TODO: improve this
        Expression::ArrowFunction(_) => {
            Some(closure_kind(vec![callable_parameter(mixed_kind(), false, true)], mixed_kind()))
        }
        Expression::Throw(_) => Some(never_kind()),
        Expression::Clone(_) => Some(any_object_kind()),
        Expression::ClosureCreation(_) => {
            Some(closure_kind(vec![callable_parameter(mixed_kind(), false, true)], mixed_kind()))
        }
        Expression::MagicConstant(magic_constant) => Some(match &magic_constant {
            MagicConstant::Line(_) => integer_kind(),
            MagicConstant::File(_) => non_empty_string_kind(),
            MagicConstant::Directory(_) => non_empty_string_kind(),
            MagicConstant::Trait(_) => non_empty_string_kind(),
            MagicConstant::Method(_) => non_empty_string_kind(),
            MagicConstant::Function(_) => non_empty_string_kind(),
            MagicConstant::Property(_) => non_empty_string_kind(),
            MagicConstant::Namespace(_) => non_empty_string_kind(),
            MagicConstant::Class(_) => non_empty_string_kind(),
        }),
        Expression::Identifier(identifier) => {
            let value = if semantics.names.is_imported(identifier) {
                interner.lookup(semantics.names.get(identifier))
            } else {
                let name = interner.lookup(identifier.value());

                if name.starts_with('\\') {
                    &name[1..]
                } else {
                    name
                }
            };

            Some(match value.to_ascii_uppercase().as_str() {
                "INF" | "NAN" | "PHP_FLOAT_EPSILON " | "PHP_FLOAT_MIN" | "PHP_FLOAT_MAX" => float_kind(),
                "PHP_VERSION" | "PHP_OS" | "PHP_SAPI" | "PHP_EOL" => non_empty_string_kind(),
                "PHP_EXTRA_VERSION" => string_kind(),
                "PHP_ZTS"
                | "PHP_DEBUG"
                | "PHP_MAXPATHLEN"
                | "PHP_INT_SIZE"
                | "PHP_FLOAT_DIG"
                | "PHP_INT_MIN"
                | "PHP_INT_MAX"
                | "PHP_MAJOR_VERSION"
                | "PHP_MINOR_VERSION"
                | "PHP_RELEASE_VERSION"
                | "PHP_VERSION_ID" => integer_kind(),
                "ZEND_THREAD_SAFE" | "ZEND_DEBUG_BUILD" => bool_kind(),
                _ => return None,
            })
        }
        _ => None,
    }
}

// Check if a TypeKind is a numeric value kind (integer or float literal)
fn is_numeric_value_kind(kind: &TypeKind) -> bool {
    matches!(kind, TypeKind::Value(ValueTypeKind::Integer { .. }) | TypeKind::Value(ValueTypeKind::Float { .. }))
}

// Extract the numeric value (as f64) from a TypeKind
fn extract_numeric_value(kind: &TypeKind) -> Option<OrderedFloat<f64>> {
    match kind {
        TypeKind::Value(ValueTypeKind::Integer { value }) => Some(OrderedFloat(*value as f64)),
        TypeKind::Value(ValueTypeKind::Float { value }) => Some(*value),
        _ => None,
    }
}

// Infer the resulting type of a numeric operation when operands are not literals
fn infer_numeric_operation_type(
    lhs_kind: Option<TypeKind>,
    rhs_kind: Option<TypeKind>,
    operator: &ArithmeticInfixOperator,
) -> Option<TypeKind> {
    match (lhs_kind, rhs_kind) {
        (Some(TypeKind::Scalar(ScalarTypeKind::Integer)), Some(TypeKind::Scalar(ScalarTypeKind::Integer))) => {
            match operator {
                ArithmeticInfixOperator::Modulo(_) => Some(integer_kind()),
                _ => Some(union_kind(vec![integer_kind(), float_kind()])),
            }
        }
        (Some(TypeKind::Scalar(ScalarTypeKind::Float)), Some(TypeKind::Scalar(ScalarTypeKind::Float)))
        | (Some(TypeKind::Scalar(ScalarTypeKind::Integer)), Some(TypeKind::Scalar(ScalarTypeKind::Float)))
        | (Some(TypeKind::Scalar(ScalarTypeKind::Float)), Some(TypeKind::Scalar(ScalarTypeKind::Integer))) => {
            match operator {
                ArithmeticInfixOperator::Modulo(_) => Some(integer_kind()),
                _ => Some(float_kind()),
            }
        }
        _ => None,
    }
}
