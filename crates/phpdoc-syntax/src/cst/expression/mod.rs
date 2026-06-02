use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::keyword::Keyword;

pub use crate::cst::expression::access::*;
pub use crate::cst::expression::array::*;
pub use crate::cst::expression::literal::*;
pub use crate::cst::expression::unary::*;

pub mod access;
pub mod array;
pub mod literal;
pub mod unary;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "kind", content = "value")]
pub enum ConstantExpression<'arena> {
    Integer(IntegerConstant<'arena>),
    Float(FloatConstant<'arena>),
    String(StringConstant<'arena>),
    True(Keyword<'arena>),
    False(Keyword<'arena>),
    Null(Keyword<'arena>),
    UnaryPrefix(UnaryPrefixConstantExpression<'arena>),
    ConstantAccess(ConstantAccessExpression<'arena>),
    ClassLikeConstantAccess(ClassLikeConstantAccessExpression<'arena>),
    Array(ArrayConstant<'arena>),
}

impl HasSpan for ConstantExpression<'_> {
    fn span(&self) -> Span {
        match self {
            ConstantExpression::Integer(expression) => expression.span(),
            ConstantExpression::Float(expression) => expression.span(),
            ConstantExpression::String(expression) => expression.span(),
            ConstantExpression::True(keyword) => keyword.span(),
            ConstantExpression::False(keyword) => keyword.span(),
            ConstantExpression::Null(keyword) => keyword.span(),
            ConstantExpression::UnaryPrefix(expression) => expression.span(),
            ConstantExpression::ConstantAccess(expression) => expression.span(),
            ConstantExpression::ClassLikeConstantAccess(expression) => expression.span(),
            ConstantExpression::Array(expression) => expression.span(),
        }
    }
}
