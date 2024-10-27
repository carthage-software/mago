use fennec_ast::*;
use fennec_reflection::r#type::kind::TypeKind;
use fennec_reflection::r#type::TypeReflection;
use fennec_semantics::Semantics;
use fennec_span::HasSpan;

pub fn infere<'ast>(semantics: &'ast Semantics, expression: &'ast Expression) -> Option<TypeReflection> {
    let kind = infere_kind(semantics, expression)?;

    Some(TypeReflection { kind, inferred: true, span: expression.span() })
}

fn infere_kind<'ast>(semantics: &'ast Semantics, expression: &'ast Expression) -> Option<TypeKind> {
    match &expression {
        Expression::Parenthesized(parenthesized) => infere_kind(semantics, &parenthesized.expression),
        Expression::Referenced(referenced) => infere_kind(semantics, &referenced.expression),
        Expression::Suppressed(suppressed) => infere_kind(semantics, &suppressed.expression),
        Expression::Literal(literal) => Some(match &literal {
            Literal::String(_) => TypeKind::String,
            Literal::Integer(_) => TypeKind::Integer,
            Literal::Float(_) => TypeKind::Float,
            Literal::True(_) => TypeKind::True,
            Literal::False(_) => TypeKind::False,
            Literal::Null(_) => TypeKind::Null,
        }),
        Expression::CompositeString(_) => Some(TypeKind::String),
        Expression::ArithmeticOperation(arithmetic_operation) => match arithmetic_operation.as_ref() {
            ArithmeticOperation::Prefix(arithmetic_prefix_operation) => {
                let value_kind = infere_kind(semantics, &arithmetic_prefix_operation.value);

                match value_kind {
                    Some(TypeKind::Float) => Some(TypeKind::Float),
                    Some(TypeKind::Integer) => match &arithmetic_prefix_operation.operator {
                        ArithmeticPrefixOperator::Increment(_) | ArithmeticPrefixOperator::Decrement(_) => {
                            Some(TypeKind::Union(vec![TypeKind::Integer, TypeKind::Float]))
                        }
                        ArithmeticPrefixOperator::Plus(_) | ArithmeticPrefixOperator::Minus(_) => {
                            Some(TypeKind::Integer)
                        }
                    },
                    _ => None,
                }
            }
            ArithmeticOperation::Infix(arithmetic_infix_operation) => {
                let lhs_kind = infere_kind(semantics, &arithmetic_infix_operation.lhs);
                let rhs_kind = infere_kind(semantics, &arithmetic_infix_operation.rhs);

                match (lhs_kind, rhs_kind) {
                    (Some(TypeKind::Float), Some(TypeKind::Float))
                    | (Some(TypeKind::Float), Some(TypeKind::Integer))
                    | (Some(TypeKind::Integer), Some(TypeKind::Float)) => {
                        match &arithmetic_infix_operation.operator {
                            ArithmeticInfixOperator::Modulo(_) => Some(TypeKind::Integer),
                            _ => Some(TypeKind::Float),
                        }
                    }
                    (Some(TypeKind::Integer), Some(TypeKind::Integer)) => {
                        match &arithmetic_infix_operation.operator {
                            ArithmeticInfixOperator::Modulo(_) => Some(TypeKind::Integer),
                            _ => Some(TypeKind::Union(vec![TypeKind::Integer, TypeKind::Float])),
                        }
                    }
                    _ => None,
                }
            }
            ArithmeticOperation::Postfix(arithmetic_postfix_operation) => {
                let value_kind = infere_kind(semantics, &arithmetic_postfix_operation.value);

                match value_kind {
                    Some(TypeKind::Float) => Some(TypeKind::Float),
                    Some(TypeKind::Integer) => {
                        Some(TypeKind::Union(vec![TypeKind::Integer, TypeKind::Float]))
                    }
                    _ => None,
                }
            }
        },
        Expression::AssignmentOperation(assignment_operation) => {
            infere_kind(semantics, &assignment_operation.rhs)
        }
        Expression::BitwiseOperation(_) => Some(TypeKind::Integer),
        Expression::ComparisonOperation(comparison_operation) => Some(match &comparison_operation.operator {
            ComparisonOperator::Spaceship(_) => TypeKind::Integer,
            _ => TypeKind::Bool,
        }),
        Expression::LogicalOperation(_) => Some(TypeKind::Bool),
        Expression::CastOperation(cast_operation) => Some(match &cast_operation.operator {
            CastOperator::Array(_, _) => TypeKind::Array,
            CastOperator::Bool(_, _) | CastOperator::Boolean(_, _) => TypeKind::Bool,
            CastOperator::Double(_, _) | CastOperator::Real(_, _) | CastOperator::Float(_, _) => {
                TypeKind::Float
            }
            CastOperator::Int(_, _) | CastOperator::Integer(_, _) => TypeKind::Integer,
            CastOperator::Object(_, _) => TypeKind::Object,
            CastOperator::Unset(_, _) => TypeKind::Null,
            CastOperator::String(_, _) | CastOperator::Binary(_, _) => TypeKind::String,
        }),
        Expression::ConcatOperation(_) => Some(TypeKind::String),
        Expression::InstanceofOperation(_) => Some(TypeKind::Bool),
        Expression::Array(_) => Some(TypeKind::Array),
        Expression::LegacyArray(_) => Some(TypeKind::Array),
        Expression::AnonymousClass(_) => Some(TypeKind::Object),
        Expression::Closure(_) => Some(TypeKind::Callable),
        Expression::ArrowFunction(_) => Some(TypeKind::Callable),
        Expression::Throw(_) => Some(TypeKind::Never),
        Expression::Clone(_) => Some(TypeKind::Object),
        Expression::ClosureCreation(_) => Some(TypeKind::Callable),
        Expression::MagicConstant(magic_constant) => Some(match &magic_constant {
            MagicConstant::Line(_) => TypeKind::Integer,
            MagicConstant::File(_) => TypeKind::String,
            MagicConstant::Directory(_) => TypeKind::String,
            MagicConstant::Trait(_) => TypeKind::String,
            MagicConstant::Method(_) => TypeKind::String,
            MagicConstant::Function(_) => TypeKind::String,
            MagicConstant::Property(_) => TypeKind::String,
            MagicConstant::Namespace(_) => TypeKind::String,
            MagicConstant::Class(_) => TypeKind::String,
        }),
        _ => None,
    }
}
