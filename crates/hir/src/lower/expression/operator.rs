use mago_allocator::Arena;
use mago_syntax::cst;

use crate::ir::expression::operator::AssignmentOperator;
use crate::ir::expression::operator::BinaryOperator;
use crate::ir::expression::operator::UnaryPostfixOperator;
use crate::ir::expression::operator::UnaryPrefixOperator;
use crate::lower::Lowering;

impl<'scratch, S, A> Lowering<'_, 'scratch, '_, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_binary_operator(&self, operator: &cst::BinaryOperator<'scratch>) -> BinaryOperator {
        match operator {
            cst::BinaryOperator::Addition(_) => BinaryOperator::Addition,
            cst::BinaryOperator::Subtraction(_) => BinaryOperator::Subtraction,
            cst::BinaryOperator::Multiplication(_) => BinaryOperator::Multiplication,
            cst::BinaryOperator::Division(_) => BinaryOperator::Division,
            cst::BinaryOperator::Modulo(_) => BinaryOperator::Modulo,
            cst::BinaryOperator::Exponentiation(_) => BinaryOperator::Exponentiation,
            cst::BinaryOperator::BitwiseAnd(_) => BinaryOperator::BitwiseAnd,
            cst::BinaryOperator::BitwiseOr(_) => BinaryOperator::BitwiseOr,
            cst::BinaryOperator::BitwiseXor(_) => BinaryOperator::BitwiseXor,
            cst::BinaryOperator::LeftShift(_) => BinaryOperator::LeftShift,
            cst::BinaryOperator::RightShift(_) => BinaryOperator::RightShift,
            cst::BinaryOperator::NullCoalesce(_) => BinaryOperator::NullCoalesce,
            cst::BinaryOperator::Equal(_) => BinaryOperator::Equal,
            cst::BinaryOperator::AngledNotEqual(_) | cst::BinaryOperator::NotEqual(_) => BinaryOperator::NotEqual,
            cst::BinaryOperator::Identical(_) => BinaryOperator::Identical,
            cst::BinaryOperator::NotIdentical(_) => BinaryOperator::NotIdentical,
            cst::BinaryOperator::LessThan(_) => BinaryOperator::LessThan,
            cst::BinaryOperator::LessThanOrEqual(_) => BinaryOperator::LessThanOrEqual,
            cst::BinaryOperator::GreaterThan(_) => BinaryOperator::GreaterThan,
            cst::BinaryOperator::GreaterThanOrEqual(_) => BinaryOperator::GreaterThanOrEqual,
            cst::BinaryOperator::Spaceship(_) => BinaryOperator::Spaceship,
            cst::BinaryOperator::StringConcat(_) => BinaryOperator::StringConcat,
            cst::BinaryOperator::Instanceof(_) => BinaryOperator::Instanceof,
            cst::BinaryOperator::LowAnd(_) | cst::BinaryOperator::And(_) => BinaryOperator::And,
            cst::BinaryOperator::LowOr(_) | cst::BinaryOperator::Or(_) => BinaryOperator::Or,
            cst::BinaryOperator::LowXor(_) => BinaryOperator::Xor,
        }
    }

    pub(crate) fn lower_assignment_operator(&self, operator: &cst::AssignmentOperator) -> Option<AssignmentOperator> {
        match operator {
            cst::AssignmentOperator::Assign(_) => None,
            cst::AssignmentOperator::Addition(_) => Some(AssignmentOperator::Addition),
            cst::AssignmentOperator::Subtraction(_) => Some(AssignmentOperator::Subtraction),
            cst::AssignmentOperator::Multiplication(_) => Some(AssignmentOperator::Multiplication),
            cst::AssignmentOperator::Division(_) => Some(AssignmentOperator::Division),
            cst::AssignmentOperator::Modulo(_) => Some(AssignmentOperator::Modulo),
            cst::AssignmentOperator::Exponentiation(_) => Some(AssignmentOperator::Exponentiation),
            cst::AssignmentOperator::Concat(_) => Some(AssignmentOperator::Concat),
            cst::AssignmentOperator::BitwiseAnd(_) => Some(AssignmentOperator::BitwiseAnd),
            cst::AssignmentOperator::BitwiseOr(_) => Some(AssignmentOperator::BitwiseOr),
            cst::AssignmentOperator::BitwiseXor(_) => Some(AssignmentOperator::BitwiseXor),
            cst::AssignmentOperator::LeftShift(_) => Some(AssignmentOperator::LeftShift),
            cst::AssignmentOperator::RightShift(_) => Some(AssignmentOperator::RightShift),
            cst::AssignmentOperator::Coalesce(_) => Some(AssignmentOperator::Coalesce),
        }
    }

    pub(crate) fn lower_unary_prefix_operator(
        &self,
        operator: &cst::UnaryPrefixOperator<'scratch>,
    ) -> UnaryPrefixOperator {
        match operator {
            cst::UnaryPrefixOperator::ErrorControl(_) => UnaryPrefixOperator::ErrorControl,
            cst::UnaryPrefixOperator::Reference(_) => UnaryPrefixOperator::Reference,
            cst::UnaryPrefixOperator::ArrayCast(_, _) => UnaryPrefixOperator::ArrayCast,
            cst::UnaryPrefixOperator::BoolCast(_, _) | cst::UnaryPrefixOperator::BooleanCast(_, _) => {
                UnaryPrefixOperator::BoolCast
            }
            cst::UnaryPrefixOperator::DoubleCast(_, _)
            | cst::UnaryPrefixOperator::RealCast(_, _)
            | cst::UnaryPrefixOperator::FloatCast(_, _) => UnaryPrefixOperator::FloatCast,
            cst::UnaryPrefixOperator::IntCast(_, _) | cst::UnaryPrefixOperator::IntegerCast(_, _) => {
                UnaryPrefixOperator::IntCast
            }
            cst::UnaryPrefixOperator::ObjectCast(_, _) => UnaryPrefixOperator::ObjectCast,
            cst::UnaryPrefixOperator::UnsetCast(_, _) => UnaryPrefixOperator::UnsetCast,
            cst::UnaryPrefixOperator::StringCast(_, _) | cst::UnaryPrefixOperator::BinaryCast(_, _) => {
                UnaryPrefixOperator::StringCast
            }
            cst::UnaryPrefixOperator::VoidCast(_, _) => UnaryPrefixOperator::VoidCast,
            cst::UnaryPrefixOperator::BitwiseNot(_) => UnaryPrefixOperator::BitwiseNot,
            cst::UnaryPrefixOperator::Not(_) => UnaryPrefixOperator::Not,
            cst::UnaryPrefixOperator::PreIncrement(_) => UnaryPrefixOperator::PreIncrement,
            cst::UnaryPrefixOperator::PreDecrement(_) => UnaryPrefixOperator::PreDecrement,
            cst::UnaryPrefixOperator::Plus(_) => UnaryPrefixOperator::Plus,
            cst::UnaryPrefixOperator::Negation(_) => UnaryPrefixOperator::Negation,
        }
    }

    pub(crate) fn lower_unary_postfix_operator(&self, operator: &cst::UnaryPostfixOperator) -> UnaryPostfixOperator {
        match operator {
            cst::UnaryPostfixOperator::PostIncrement(_) => UnaryPostfixOperator::PostIncrement,
            cst::UnaryPostfixOperator::PostDecrement(_) => UnaryPostfixOperator::PostDecrement,
        }
    }
}
