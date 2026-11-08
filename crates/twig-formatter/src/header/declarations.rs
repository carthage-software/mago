use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_formatter_core::document::Document;
use mago_span::HasSpan;
use mago_twig_syntax::cst::{Apply, Macro, Types};

use crate::document::DocumentBuilder;

impl<'arena, A: Arena> DocumentBuilder<'arena, A> {
    pub(super) fn macro_header(&self, node: &Macro<'arena>) -> Document<'arena, A> {
        let arguments = self.separated(&node.arguments, |argument| {
            self.array([
                argument.ellipsis.map_or_else(Document::empty, |span| self.raw(span)),
                self.raw(argument.name.span),
                argument.default.as_ref().map_or_else(Document::empty, |value| {
                    self.array([Document::String(b" = "), self.expression(value)])
                }),
            ])
        });

        self.array([
            self.raw(node.keyword.span),
            Document::space(),
            self.raw(node.name.span),
            self.enclosed(self.raw(node.left_parenthesis), arguments, self.raw(node.right_parenthesis)),
        ])
    }

    pub(super) fn apply_header(&self, node: &Apply<'arena>) -> Document<'arena, A> {
        let mut filters = Vec::with_capacity_in(node.filters.len() * 2, self.arena);
        for (index, filter) in node.filters.iter().enumerate() {
            if index != 0 {
                filters.push(Document::String(b"|"));
            }
            filters.push(self.array([
                self.raw(filter.name.span),
                filter.argument_list.as_ref().map_or_else(Document::empty, |arguments| self.arguments(arguments)),
            ]));
        }

        self.array([self.raw(node.keyword.span), Document::space(), Document::Array(filters)])
    }

    pub(super) fn types_header(&self, node: &Types<'arena>) -> Document<'arena, A> {
        let entries = self.separated(&node.declarations, |declaration| {
            self.array([
                self.raw(declaration.name.span),
                declaration.question_mark.map_or_else(Document::empty, |span| self.raw(span)),
                Document::String(b": "),
                self.raw(declaration.value.span()),
            ])
        });
        let entries = if let (Some(open), Some(close)) = (node.left_brace, node.right_brace) {
            self.enclosed(self.raw(open), entries, self.raw(close))
        } else {
            entries
        };

        self.array([self.raw(node.keyword.span), Document::space(), entries])
    }
}
