use mago_syntax::ast::BinaryOperator;
use mago_syntax::token::GetPrecedence;

pub fn should_flatten<'arena>(operator: &'arena BinaryOperator, parent_op: &'arena BinaryOperator) -> bool {
    if operator.is_low_precedence() {
        return false;
    }

    let self_precedence = operator.precedence();
    let parent_precedence = parent_op.precedence();

    if self_precedence != parent_precedence {
        // Do not flatten if operators have different precedence
        return false;
    }

    if operator.is_concatenation() && parent_op.is_concatenation() {
        return true;
    }

    if operator.is_arithmetic() && parent_op.is_arithmetic() {
        // Prevent flattening for non-associative operators
        if matches!((operator, parent_op), (BinaryOperator::Exponentiation(_), BinaryOperator::Exponentiation(_))) {
            return false;
        }

        if matches!(operator, BinaryOperator::Subtraction(_) | BinaryOperator::Division(_))
            || matches!(parent_op, BinaryOperator::Subtraction(_) | BinaryOperator::Division(_))
        {
            return false;
        }
    }

    if operator.is_bitwise() && parent_op.is_bitwise() && (operator.is_bit_shift() || parent_op.is_bit_shift()) {
        return false;
    }

    // Flatten if operators are the same
    operator.is_same_as(parent_op)
}
