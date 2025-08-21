use crate::ast::*;

/// Determine if an expression is truthy.
///
/// This function will return true if the expression is truthy, and false otherwise.
///
/// When this function returns true, it does not necessarily mean that the expression will always evaluate to true.
/// It simply means that the expression is truthy in the context of PHP.
#[inline]
pub fn is_truthy(expression: &Expression<'_>) -> bool {
    match &expression {
        Expression::Parenthesized(parenthesized) => is_truthy(parenthesized.expression),
        Expression::Literal(Literal::True(_)) => true,
        Expression::AnonymousClass(_) => true,
        Expression::Closure(_) => true,
        Expression::ArrowFunction(_) => true,
        Expression::Array(array) => !array.elements.is_empty(),
        Expression::LegacyArray(array) => !array.elements.is_empty(),
        Expression::ClosureCreation(_) => true,
        Expression::Binary(operation) => match operation.operator {
            BinaryOperator::Or(_) | BinaryOperator::LowOr(_) => is_truthy(operation.lhs) || is_truthy(operation.rhs),
            BinaryOperator::And(_) | BinaryOperator::LowAnd(_) => is_truthy(operation.lhs) && is_truthy(operation.rhs),
            BinaryOperator::NullCoalesce(_) => is_truthy(operation.lhs),
            BinaryOperator::LowXor(_) => is_truthy(operation.lhs) ^ is_truthy(operation.rhs),
            _ => false,
        },
        Expression::UnaryPrefix(operation) => match operation.operator {
            UnaryPrefixOperator::ErrorControl(_) => is_truthy(operation.operand),
            UnaryPrefixOperator::Reference(_) => is_truthy(operation.operand),
            UnaryPrefixOperator::Not(_) => is_falsy(operation.operand),
            _ => false,
        },
        Expression::Assignment(assignment) => is_truthy(assignment.rhs),
        _ => false,
    }
}

/// Determine if an expression is falsy.
///
/// This function will return true if the expression is falsy, and false otherwise.
///
/// When this function returns false, it does not mean that the expression is truthy,
/// it just means that we could not determine if the expression is falsy.
#[inline]
pub fn is_falsy(expression: &Expression<'_>) -> bool {
    match &expression {
        Expression::Parenthesized(parenthesized) => is_falsy(parenthesized.expression),
        Expression::Literal(Literal::False(_) | Literal::Null(_)) => true,
        Expression::Array(array) => array.elements.is_empty(),
        Expression::LegacyArray(array) => array.elements.is_empty(),
        Expression::Assignment(assignment) => is_falsy(assignment.rhs),
        Expression::Binary(operation) => match operation.operator {
            BinaryOperator::Or(_) | BinaryOperator::LowOr(_) => is_falsy(operation.lhs) && is_falsy(operation.rhs),
            BinaryOperator::And(_) | BinaryOperator::LowAnd(_) => is_falsy(operation.lhs) || is_falsy(operation.rhs),
            BinaryOperator::NullCoalesce(_) => is_falsy(operation.lhs) && is_falsy(operation.rhs),
            BinaryOperator::LowXor(_) => is_falsy(operation.lhs) ^ is_falsy(operation.rhs),
            _ => false,
        },
        Expression::UnaryPrefix(operation) => match operation.operator {
            UnaryPrefixOperator::ErrorControl(_) => is_falsy(operation.operand),
            UnaryPrefixOperator::Reference(_) => is_falsy(operation.operand),
            UnaryPrefixOperator::Not(_) => is_truthy(operation.operand),
            _ => false,
        },
        _ => false,
    }
}
