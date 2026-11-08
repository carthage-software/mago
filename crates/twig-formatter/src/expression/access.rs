use crate::document::DocumentBuilder;
use mago_allocator::Arena;
use mago_formatter_core::document::Document;
use mago_span::HasSpan;
use mago_twig_syntax::cst::{Call, Expression, GetAttribute, GetItem, MethodCall, Parenthesized, Slice};

impl<'arena, A: Arena> DocumentBuilder<'arena, A> {
    pub(super) fn parenthesized(&self, group: &Parenthesized<'arena>) -> Document<'arena, A> {
        self.enclosed(self.raw(group.left_parenthesis), self.expression(group.inner), self.raw(group.right_parenthesis))
    }

    pub(super) fn attribute(&self, access: &GetAttribute<'arena>) -> Document<'arena, A> {
        let separate_dot = !access.null_safe
            && matches!(
                access.object,
                Expression::Number(_) | Expression::GetAttribute(GetAttribute { attribute: Expression::Number(_), .. })
            );
        self.array([
            self.expression(access.object),
            if separate_dot { Document::space() } else { Document::empty() },
            self.raw(access.dot),
            self.expression(access.attribute),
        ])
    }

    pub(super) fn item(&self, access: &GetItem<'arena>) -> Document<'arena, A> {
        self.array([
            self.expression(access.object),
            self.enclosed(self.raw(access.left_bracket), self.expression(access.index), self.raw(access.right_bracket)),
        ])
    }

    pub(super) fn slice(&self, slice: &Slice<'arena>) -> Document<'arena, A> {
        self.array([
            self.expression(slice.object),
            self.raw(slice.left_bracket),
            slice.start.map_or_else(Document::empty, |start| self.expression(start)),
            Document::String(b":"),
            slice.length.map_or_else(Document::empty, |length| self.expression(length)),
            self.raw(slice.right_bracket),
        ])
    }

    pub(super) fn call(&self, call: &Call<'arena>) -> Document<'arena, A> {
        self.array([self.expression(call.callee), self.arguments(&call.argument_list)])
    }

    pub(super) fn method_call(&self, call: &MethodCall<'arena>) -> Document<'arena, A> {
        self.array([
            self.expression(call.object),
            self.raw(call.dot),
            self.raw(call.method.span()),
            self.arguments(&call.argument_list),
        ])
    }
}
