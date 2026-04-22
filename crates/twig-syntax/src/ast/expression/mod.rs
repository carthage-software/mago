use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

pub use crate::ast::expression::argument::*;
pub use crate::ast::expression::array::*;
pub use crate::ast::expression::arrow::*;
pub use crate::ast::expression::attribute::*;
pub use crate::ast::expression::binary::*;
pub use crate::ast::expression::call::*;
pub use crate::ast::expression::conditional::*;
pub use crate::ast::expression::filter::*;
pub use crate::ast::expression::group::*;
pub use crate::ast::expression::literal::*;
pub use crate::ast::expression::name::*;
pub use crate::ast::expression::test::*;
pub use crate::ast::expression::unary::*;

pub mod argument;
pub mod array;
pub mod arrow;
pub mod attribute;
pub mod binary;
pub mod call;
pub mod conditional;
pub mod filter;
pub mod group;
pub mod literal;
pub mod name;
pub mod test;
pub mod unary;

/// A Twig expression.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum Expression<'arena> {
    Name(Name<'arena>),
    Number(Number<'arena>),
    String(StringLiteral<'arena>),
    InterpolatedString(InterpolatedString<'arena>),
    Bool(Bool),
    Null(Null),
    Array(Array<'arena>),
    HashMap(HashMap<'arena>),
    Unary(Unary<'arena>),
    Binary(Binary<'arena>),
    Conditional(Conditional<'arena>),
    GetAttribute(GetAttribute<'arena>),
    GetItem(GetItem<'arena>),
    Slice(Slice<'arena>),
    Call(Call<'arena>),
    MethodCall(MethodCall<'arena>),
    Filter(Filter<'arena>),
    Test(Test<'arena>),
    Parenthesized(Parenthesized<'arena>),
    ArrowFunction(ArrowFunction<'arena>),
}

impl HasSpan for Expression<'_> {
    fn span(&self) -> Span {
        match self {
            Expression::Name(e) => e.span(),
            Expression::Number(e) => e.span(),
            Expression::String(e) => e.span(),
            Expression::InterpolatedString(e) => e.span(),
            Expression::Bool(e) => e.span(),
            Expression::Null(e) => e.span(),
            Expression::Array(e) => e.span(),
            Expression::HashMap(e) => e.span(),
            Expression::Unary(e) => e.span(),
            Expression::Binary(e) => e.span(),
            Expression::Conditional(e) => e.span(),
            Expression::GetAttribute(e) => e.span(),
            Expression::GetItem(e) => e.span(),
            Expression::Slice(e) => e.span(),
            Expression::Call(e) => e.span(),
            Expression::MethodCall(e) => e.span(),
            Expression::Filter(e) => e.span(),
            Expression::Test(e) => e.span(),
            Expression::Parenthesized(e) => e.span(),
            Expression::ArrowFunction(e) => e.span(),
        }
    }
}
