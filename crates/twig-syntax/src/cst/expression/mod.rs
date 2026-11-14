use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

pub use crate::cst::expression::argument::*;
pub use crate::cst::expression::array::*;
pub use crate::cst::expression::arrow::*;
pub use crate::cst::expression::attribute::*;
pub use crate::cst::expression::binary::*;
pub use crate::cst::expression::call::*;
pub use crate::cst::expression::conditional::*;
pub use crate::cst::expression::filter::*;
pub use crate::cst::expression::group::*;
pub use crate::cst::expression::literal::*;
pub use crate::cst::expression::name::*;
pub use crate::cst::expression::test::*;
pub use crate::cst::expression::unary::*;

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
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type", content = "value"))]
pub enum Expression<'arena> {
    Name(Name<'arena>),
    Number(Number<'arena>),
    String(StringLiteral<'arena>),
    InterpolatedString(InterpolatedString<'arena>),
    StringConcatenation(StringConcatenation<'arena>),
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
            Expression::Name(e) => return e.span(),
            Expression::Number(e) => return e.span(),
            Expression::String(e) => return e.span(),
            Expression::InterpolatedString(e) => return e.span(),
            Expression::Bool(e) => return e.span(),
            Expression::Null(e) => return e.span(),
            Expression::Array(e) => return e.span(),
            Expression::HashMap(e) => return e.span(),
            Expression::Parenthesized(e) => return e.span(),
            _ => {}
        }

        let mut left = self;
        let start = loop {
            left = match left {
                Expression::Unary(e) => break e.operator.span(),
                Expression::Binary(e) => e.lhs,
                Expression::StringConcatenation(e) => e.lhs,
                Expression::Conditional(e) => e.condition,
                Expression::GetAttribute(e) => e.object,
                Expression::GetItem(e) => e.object,
                Expression::Slice(e) => e.object,
                Expression::Call(e) => e.callee,
                Expression::MethodCall(e) => e.object,
                Expression::Filter(e) => e.operand,
                Expression::Test(e) => e.operand,
                Expression::ArrowFunction(e) => {
                    break e
                        .left_parenthesis
                        .or_else(|| e.parameters.nodes.first().map(|parameter| parameter.span))
                        .unwrap_or(e.fat_arrow);
                }
                _ => break left.span(),
            };
        };

        let mut right = self;
        let end = loop {
            right = match right {
                Expression::Unary(e) => e.operand,
                Expression::Binary(e) => e.rhs,
                Expression::StringConcatenation(e) => e.rhs,
                Expression::Conditional(e) => {
                    if let Some(otherwise) = e.r#else {
                        otherwise
                    } else if let Some(colon) = e.colon {
                        break colon;
                    } else if let Some(then) = e.then {
                        then
                    } else {
                        break e.question_mark;
                    }
                }
                Expression::GetAttribute(e) => e.attribute,
                Expression::GetItem(e) => break e.right_bracket,
                Expression::Slice(e) => break e.right_bracket,
                Expression::Call(e) => break e.argument_list.right_parenthesis,
                Expression::MethodCall(e) => break e.argument_list.right_parenthesis,
                Expression::Filter(e) => {
                    break e.argument_list.as_ref().map_or(e.name.span, |arguments| arguments.right_parenthesis);
                }
                Expression::Test(e) => match &e.arguments {
                    TestArguments::Bare(expression) => expression,
                    TestArguments::Parenthesised(arguments) => break arguments.right_parenthesis,
                    TestArguments::None => break e.second_word.map_or(e.name.span, |word| word.span),
                },
                Expression::ArrowFunction(e) => e.body,
                _ => break right.span(),
            };
        };

        start.join(end)
    }
}
