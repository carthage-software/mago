use mago_allocator::Arena;
use mago_formatter_core::document::Document;
use mago_span::HasSpan;
use mago_twig_syntax::cst::Expression;

use crate::document::DocumentBuilder;

mod access;
mod array;
mod call;
pub(super) mod operator;

impl<'arena, A: Arena> DocumentBuilder<'arena, A> {
    pub(super) fn expression(&self, expression: &Expression<'arena>) -> Document<'arena, A> {
        match expression {
            Expression::Name(_)
            | Expression::Number(_)
            | Expression::String(_)
            | Expression::InterpolatedString(_)
            | Expression::StringConcatenation(_)
            | Expression::Bool(_)
            | Expression::Null(_) => self.raw(expression.span()),
            Expression::Array(array) => self.array_expression(array),
            Expression::HashMap(map) => self.hash_map(map),
            Expression::Unary(unary) => self.unary(unary),
            Expression::Binary(binary) => self.binary(binary),
            Expression::Conditional(conditional) => self.conditional(conditional),
            Expression::GetAttribute(access) => self.attribute(access),
            Expression::GetItem(access) => self.item(access),
            Expression::Slice(slice) => self.slice(slice),
            Expression::Call(call) => self.call(call),
            Expression::MethodCall(call) => self.method_call(call),
            Expression::Filter(filter) => self.filter(filter),
            Expression::Test(test) => self.test(test),
            Expression::Parenthesized(group) => self.parenthesized(group),
            Expression::ArrowFunction(arrow) => self.arrow(arrow),
        }
    }
}
