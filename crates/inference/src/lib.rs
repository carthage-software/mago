use fennec_ast::*;
use fennec_interner::ThreadedInterner;
use fennec_reflection::r#type::kind::*;
use fennec_reflection::r#type::TypeReflection;
use fennec_semantics::Semantics;
use fennec_span::HasSpan;

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
                    if value > isize::MAX as usize {
                        integer_kind()
                    } else {
                        // we can safely cast `value` to an `isize`
                        value_integer_kind(value as isize)
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
                    kind @ Some(TypeKind::Scalar(ScalarTypeKind::Float)) => kind,
                    Some(TypeKind::Scalar(ScalarTypeKind::Integer)) => match &arithmetic_prefix_operation.operator {
                        ArithmeticPrefixOperator::Increment(_) | ArithmeticPrefixOperator::Decrement(_) => {
                            Some(union_kind(vec![integer_kind(), float_kind()]))
                        }
                        ArithmeticPrefixOperator::Plus(_) | ArithmeticPrefixOperator::Minus(_) => Some(integer_kind()),
                    },
                    Some(TypeKind::Value(ValueTypeKind::Integer { value })) => {
                        Some(match &arithmetic_prefix_operation.operator {
                            ArithmeticPrefixOperator::Increment(_) => {
                                if value == isize::MAX {
                                    float_kind()
                                } else {
                                    value_integer_kind(value + 1)
                                }
                            }
                            ArithmeticPrefixOperator::Decrement(_) => {
                                if value == isize::MIN {
                                    float_kind()
                                } else {
                                    value_integer_kind(value - 1)
                                }
                            }
                            ArithmeticPrefixOperator::Plus(_) => value_integer_kind(value),
                            ArithmeticPrefixOperator::Minus(_) => value_integer_kind(-value),
                        })
                    }
                    _ => {
                        // we can return `int|float` here, but some PHP extensions overload the operators
                        // making it possible to return other types ( e.g `BCMath\Number`, and `GMP`)
                        None
                    }
                }
            }
            ArithmeticOperation::Infix(arithmetic_infix_operation) => {
                let lhs_kind = infere_kind(interner, semantics, &arithmetic_infix_operation.lhs);
                let rhs_kind = infere_kind(interner, semantics, &arithmetic_infix_operation.rhs);

                match (lhs_kind, rhs_kind) {
                    (Some(TypeKind::Scalar(ScalarTypeKind::Float)), Some(TypeKind::Scalar(ScalarTypeKind::Float)))
                    | (
                        Some(TypeKind::Scalar(ScalarTypeKind::Float)),
                        Some(TypeKind::Scalar(ScalarTypeKind::Integer)),
                    )
                    | (
                        Some(TypeKind::Scalar(ScalarTypeKind::Integer)),
                        Some(TypeKind::Scalar(ScalarTypeKind::Float)),
                    ) => match &arithmetic_infix_operation.operator {
                        ArithmeticInfixOperator::Modulo(_) => Some(integer_kind()),
                        _ => Some(float_kind()),
                    },
                    (
                        Some(TypeKind::Scalar(ScalarTypeKind::Integer)),
                        Some(TypeKind::Scalar(ScalarTypeKind::Integer)),
                    ) => match &arithmetic_infix_operation.operator {
                        ArithmeticInfixOperator::Modulo(_) => Some(integer_kind()),
                        _ => Some(union_kind(vec![integer_kind(), float_kind()])),
                    },
                    _ => None,
                }
            }
            ArithmeticOperation::Postfix(arithmetic_postfix_operation) => {
                let value_kind = infere_kind(interner, semantics, &arithmetic_postfix_operation.value);

                match value_kind {
                    Some(TypeKind::Scalar(ScalarTypeKind::Float)) => Some(float_kind()),
                    Some(TypeKind::Scalar(ScalarTypeKind::Integer)) => {
                        Some(union_kind(vec![integer_kind(), float_kind()]))
                    }
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
