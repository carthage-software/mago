use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_syntax::cst;

use crate::ir::expression::operator::AndBinaryOperatorKind;
use crate::ir::expression::operator::AssignmentOperator;
use crate::ir::expression::operator::AssignmentOperatorKind;
use crate::ir::expression::operator::BinaryOperator;
use crate::ir::expression::operator::BinaryOperatorKind;
use crate::ir::expression::operator::BoolCastUnaryPrefixOperatorKind;
use crate::ir::expression::operator::FloatCastUnaryPrefixOperatorKind;
use crate::ir::expression::operator::IntCastUnaryPrefixOperatorKind;
use crate::ir::expression::operator::NotEqualBinaryOperatorKind;
use crate::ir::expression::operator::OrBinaryOperatorKind;
use crate::ir::expression::operator::StringCastUnaryPrefixOperatorKind;
use crate::ir::expression::operator::UnaryPostfixOperator;
use crate::ir::expression::operator::UnaryPostfixOperatorKind;
use crate::ir::expression::operator::UnaryPrefixOperator;
use crate::ir::expression::operator::UnaryPrefixOperatorKind;
use crate::lower::Lowering;

impl<'scratch, S, A> Lowering<'_, 'scratch, '_, S, A>
where
    S: Arena,
    A: Arena,
{
    pub(crate) fn lower_binary_operator(&self, operator: &cst::BinaryOperator<'scratch>) -> BinaryOperator {
        let kind = match operator {
            cst::BinaryOperator::Addition(_) => BinaryOperatorKind::Addition,
            cst::BinaryOperator::Subtraction(_) => BinaryOperatorKind::Subtraction,
            cst::BinaryOperator::Multiplication(_) => BinaryOperatorKind::Multiplication,
            cst::BinaryOperator::Division(_) => BinaryOperatorKind::Division,
            cst::BinaryOperator::Modulo(_) => BinaryOperatorKind::Modulo,
            cst::BinaryOperator::Exponentiation(_) => BinaryOperatorKind::Exponentiation,
            cst::BinaryOperator::BitwiseAnd(_) => BinaryOperatorKind::BitwiseAnd,
            cst::BinaryOperator::BitwiseOr(_) => BinaryOperatorKind::BitwiseOr,
            cst::BinaryOperator::BitwiseXor(_) => BinaryOperatorKind::BitwiseXor,
            cst::BinaryOperator::LeftShift(_) => BinaryOperatorKind::LeftShift,
            cst::BinaryOperator::RightShift(_) => BinaryOperatorKind::RightShift,
            cst::BinaryOperator::NullCoalesce(_) => BinaryOperatorKind::NullCoalesce,
            cst::BinaryOperator::Equal(_) => BinaryOperatorKind::Equal,
            cst::BinaryOperator::NotEqual(_) => BinaryOperatorKind::NotEqual(NotEqualBinaryOperatorKind::BangEqual),
            cst::BinaryOperator::AngledNotEqual(_) => {
                BinaryOperatorKind::NotEqual(NotEqualBinaryOperatorKind::LessGreater)
            }
            cst::BinaryOperator::Identical(_) => BinaryOperatorKind::Identical,
            cst::BinaryOperator::NotIdentical(_) => BinaryOperatorKind::NotIdentical,
            cst::BinaryOperator::LessThan(_) => BinaryOperatorKind::LessThan,
            cst::BinaryOperator::LessThanOrEqual(_) => BinaryOperatorKind::LessThanOrEqual,
            cst::BinaryOperator::GreaterThan(_) => BinaryOperatorKind::GreaterThan,
            cst::BinaryOperator::GreaterThanOrEqual(_) => BinaryOperatorKind::GreaterThanOrEqual,
            cst::BinaryOperator::Spaceship(_) => BinaryOperatorKind::Spaceship,
            cst::BinaryOperator::StringConcat(_) => BinaryOperatorKind::StringConcat,
            cst::BinaryOperator::Instanceof(_) => BinaryOperatorKind::Instanceof,
            cst::BinaryOperator::And(_) => BinaryOperatorKind::And(AndBinaryOperatorKind::AmpersandAmpersand),
            cst::BinaryOperator::LowAnd(_) => BinaryOperatorKind::And(AndBinaryOperatorKind::KeywordAnd),
            cst::BinaryOperator::Or(_) => BinaryOperatorKind::Or(OrBinaryOperatorKind::PipePipe),
            cst::BinaryOperator::LowOr(_) => BinaryOperatorKind::Or(OrBinaryOperatorKind::KeywordOr),
            cst::BinaryOperator::LowXor(_) => BinaryOperatorKind::Xor,
        };

        BinaryOperator { span: operator.span(), kind }
    }

    pub(crate) fn lower_assignment_operator(&self, operator: &cst::AssignmentOperator) -> AssignmentOperator {
        let kind = match operator {
            cst::AssignmentOperator::Assign(_) => AssignmentOperatorKind::Assign,
            cst::AssignmentOperator::Addition(_) => AssignmentOperatorKind::Addition,
            cst::AssignmentOperator::Subtraction(_) => AssignmentOperatorKind::Subtraction,
            cst::AssignmentOperator::Multiplication(_) => AssignmentOperatorKind::Multiplication,
            cst::AssignmentOperator::Division(_) => AssignmentOperatorKind::Division,
            cst::AssignmentOperator::Modulo(_) => AssignmentOperatorKind::Modulo,
            cst::AssignmentOperator::Exponentiation(_) => AssignmentOperatorKind::Exponentiation,
            cst::AssignmentOperator::Concat(_) => AssignmentOperatorKind::Concat,
            cst::AssignmentOperator::BitwiseAnd(_) => AssignmentOperatorKind::BitwiseAnd,
            cst::AssignmentOperator::BitwiseOr(_) => AssignmentOperatorKind::BitwiseOr,
            cst::AssignmentOperator::BitwiseXor(_) => AssignmentOperatorKind::BitwiseXor,
            cst::AssignmentOperator::LeftShift(_) => AssignmentOperatorKind::LeftShift,
            cst::AssignmentOperator::RightShift(_) => AssignmentOperatorKind::RightShift,
            cst::AssignmentOperator::Coalesce(_) => AssignmentOperatorKind::Coalesce,
        };

        AssignmentOperator { span: operator.span(), kind }
    }

    pub(crate) fn lower_unary_prefix_operator(
        &self,
        operator: &cst::UnaryPrefixOperator<'scratch>,
    ) -> UnaryPrefixOperator {
        let kind = match operator {
            cst::UnaryPrefixOperator::ErrorControl(_) => UnaryPrefixOperatorKind::ErrorControl,
            cst::UnaryPrefixOperator::Reference(_) => UnaryPrefixOperatorKind::Reference,
            cst::UnaryPrefixOperator::ArrayCast(_, _) => UnaryPrefixOperatorKind::ArrayCast,
            cst::UnaryPrefixOperator::BoolCast(_, _) => {
                UnaryPrefixOperatorKind::BoolCast(BoolCastUnaryPrefixOperatorKind::Bool)
            }
            cst::UnaryPrefixOperator::BooleanCast(_, _) => {
                UnaryPrefixOperatorKind::BoolCast(BoolCastUnaryPrefixOperatorKind::Boolean)
            }
            cst::UnaryPrefixOperator::FloatCast(_, _) => {
                UnaryPrefixOperatorKind::FloatCast(FloatCastUnaryPrefixOperatorKind::Float)
            }
            cst::UnaryPrefixOperator::RealCast(_, _) => {
                UnaryPrefixOperatorKind::FloatCast(FloatCastUnaryPrefixOperatorKind::Real)
            }
            cst::UnaryPrefixOperator::DoubleCast(_, _) => {
                UnaryPrefixOperatorKind::FloatCast(FloatCastUnaryPrefixOperatorKind::Double)
            }
            cst::UnaryPrefixOperator::IntCast(_, _) => {
                UnaryPrefixOperatorKind::IntCast(IntCastUnaryPrefixOperatorKind::Int)
            }
            cst::UnaryPrefixOperator::IntegerCast(_, _) => {
                UnaryPrefixOperatorKind::IntCast(IntCastUnaryPrefixOperatorKind::Integer)
            }
            cst::UnaryPrefixOperator::ObjectCast(_, _) => UnaryPrefixOperatorKind::ObjectCast,
            cst::UnaryPrefixOperator::UnsetCast(_, _) => UnaryPrefixOperatorKind::UnsetCast,
            cst::UnaryPrefixOperator::StringCast(_, _) => {
                UnaryPrefixOperatorKind::StringCast(StringCastUnaryPrefixOperatorKind::String)
            }
            cst::UnaryPrefixOperator::BinaryCast(_, _) => {
                UnaryPrefixOperatorKind::StringCast(StringCastUnaryPrefixOperatorKind::Binary)
            }
            cst::UnaryPrefixOperator::VoidCast(_, _) => UnaryPrefixOperatorKind::VoidCast,
            cst::UnaryPrefixOperator::BitwiseNot(_) => UnaryPrefixOperatorKind::BitwiseNot,
            cst::UnaryPrefixOperator::Not(_) => UnaryPrefixOperatorKind::Not,
            cst::UnaryPrefixOperator::PreIncrement(_) => UnaryPrefixOperatorKind::PreIncrement,
            cst::UnaryPrefixOperator::PreDecrement(_) => UnaryPrefixOperatorKind::PreDecrement,
            cst::UnaryPrefixOperator::Plus(_) => UnaryPrefixOperatorKind::Plus,
            cst::UnaryPrefixOperator::Negation(_) => UnaryPrefixOperatorKind::Negation,
        };

        UnaryPrefixOperator { span: operator.span(), kind }
    }

    pub(crate) fn lower_unary_postfix_operator(&self, operator: &cst::UnaryPostfixOperator) -> UnaryPostfixOperator {
        let kind = match operator {
            cst::UnaryPostfixOperator::PostIncrement(_) => UnaryPostfixOperatorKind::PostIncrement,
            cst::UnaryPostfixOperator::PostDecrement(_) => UnaryPostfixOperatorKind::PostDecrement,
        };

        UnaryPostfixOperator { span: operator.span(), kind }
    }
}
