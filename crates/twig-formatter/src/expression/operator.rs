use crate::document::DocumentBuilder;
use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_formatter_core::document::Document;
use mago_formatter_core::document::Line;
use mago_span::HasSpan;
use mago_twig_syntax::cst::{Binary, BinaryOperator, Conditional, Expression, Unary, UnaryOperator};

impl<'arena, A: Arena> DocumentBuilder<'arena, A> {
    pub(super) fn unary(&self, unary: &Unary<'arena>) -> Document<'arena, A> {
        self.array([
            self.raw(unary.operator.span()),
            if matches!(unary.operator, UnaryOperator::Not(_)) || matches!(unary.operand, Expression::Unary(_)) {
                Document::space()
            } else {
                Document::empty()
            },
            self.expression(unary.operand),
        ])
    }

    pub(super) fn binary(&self, binary: &Binary<'arena>) -> Document<'arena, A> {
        let operator = binary.operator.span();
        let operator = self.words(&self.source[operator.start.offset as usize..operator.end.offset as usize]);
        if matches!(binary.operator, BinaryOperator::Range(_)) {
            return self.array([
                self.expression(binary.lhs),
                operator,
                if starts_with_not(binary.rhs) { Document::String(b"  ") } else { Document::empty() },
                self.expression(binary.rhs),
            ]);
        }

        self.group([
            self.expression(binary.lhs),
            Document::Indent(self.list([
                Document::Line(Line::default()),
                operator,
                Document::space(),
                self.expression(binary.rhs),
            ])),
        ])
    }

    pub(super) fn conditional(&self, conditional: &Conditional<'arena>) -> Document<'arena, A> {
        let capacity = if conditional.colon.is_some() || conditional.r#else.is_some() { 8 } else { 4 };
        let mut contents = Vec::with_capacity_in(capacity, self.arena);
        contents.push(Document::Line(Line::default()));
        contents.push(Document::String(b"?"));
        if let Some(then) = conditional.then {
            contents.push(Document::space());
            contents.push(self.expression(then));
        }

        if conditional.colon.is_some() || conditional.r#else.is_some() {
            if conditional.then.is_some() {
                contents.push(Document::Line(Line::default()));
            }

            contents.push(Document::String(b":"));
        }

        if let Some(otherwise) = conditional.r#else {
            contents.push(Document::space());
            contents.push(self.expression(otherwise));
        }

        self.group([self.expression(conditional.condition), Document::Indent(contents)])
    }
}

pub(crate) fn starts_with_not(mut expression: &Expression<'_>) -> bool {
    loop {
        expression = match expression {
            Expression::Unary(unary) => return matches!(unary.operator, UnaryOperator::Not(_)),
            Expression::Binary(binary) => binary.lhs,
            Expression::Conditional(conditional) => conditional.condition,
            Expression::GetAttribute(access) => access.object,
            Expression::GetItem(access) => access.object,
            Expression::Slice(access) => access.object,
            Expression::Call(call) => call.callee,
            Expression::MethodCall(call) => call.object,
            Expression::Filter(filter) => filter.operand,
            Expression::Test(test) => test.operand,
            _ => return false,
        };
    }
}
