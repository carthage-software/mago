use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_formatter_core::document::{Align, Document, Group, Line};
use mago_span::{HasSpan, Span};
use mago_twig_syntax::cst::{Expression, TokenSeparatedSequence};

use crate::expression::operator::starts_with_not;
use crate::tag::CodeTag;

pub(super) struct DocumentBuilder<'arena, A: Arena> {
    pub arena: &'arena A,
    pub source: &'arena [u8],
}

impl<'arena, A: Arena> DocumentBuilder<'arena, A> {
    pub(super) fn raw(&self, span: Span) -> Document<'arena, A> {
        Document::String(&self.source[span.start.offset as usize..span.end.offset as usize])
    }

    pub(super) fn list<const N: usize>(
        &self,
        documents: [Document<'arena, A>; N],
    ) -> Vec<'arena, Document<'arena, A>, A> {
        let mut result = Vec::with_capacity_in(N, self.arena);
        result.extend(documents);
        result
    }

    pub(super) fn array<const N: usize>(&self, documents: [Document<'arena, A>; N]) -> Document<'arena, A> {
        Document::Array(self.list(documents))
    }

    pub(super) fn group<const N: usize>(&self, documents: [Document<'arena, A>; N]) -> Document<'arena, A> {
        Document::Group(Group::new(self.list(documents)))
    }

    pub(super) fn words(&self, bytes: &'arena [u8]) -> Document<'arena, A> {
        let bytes = bytes.trim_ascii();
        if bytes.iter().all(|byte| !byte.is_ascii_whitespace()) {
            return Document::String(bytes);
        }

        let mut documents = Vec::new_in(self.arena);
        for word in bytes.split(u8::is_ascii_whitespace).filter(|word| !word.is_empty()) {
            if !documents.is_empty() {
                documents.push(Document::space());
            }

            documents.push(Document::String(word));
        }

        Document::Array(documents)
    }

    pub(super) fn spread(&self, ellipsis: Span, expression: &Expression<'arena>) -> Document<'arena, A> {
        self.array([
            self.raw(ellipsis),
            if starts_with_not(expression) { Document::String(b"  ") } else { Document::empty() },
            self.expression(expression),
        ])
    }

    pub(super) fn enclosed(
        &self,
        open: Document<'arena, A>,
        contents: Document<'arena, A>,
        close: Document<'arena, A>,
    ) -> Document<'arena, A> {
        self.group([
            open,
            Document::Indent(self.list([Document::Line(Line::soft()), contents])),
            Document::Line(Line::soft()),
            close,
        ])
    }

    pub(super) fn separated<T>(
        &self,
        sequence: &TokenSeparatedSequence<'arena, T>,
        format: impl Fn(&T) -> Document<'arena, A>,
    ) -> Document<'arena, A>
    where
        T: HasSpan,
    {
        if !sequence.has_trailing_token() && sequence.len() <= 1 {
            return sequence.iter().next().map_or_else(Document::empty, format);
        }

        let capacity = (sequence.len() * 3).saturating_sub(2) + usize::from(sequence.has_trailing_token());
        let mut documents = Vec::with_capacity_in(capacity, self.arena);
        for (index, item) in sequence.iter().enumerate() {
            if index != 0 {
                documents.push(Document::String(b","));
                documents.push(Document::Line(Line::default()));
            }

            documents.push(format(item));
        }

        if sequence.has_trailing_token() {
            documents.push(Document::String(b","));
        }

        Document::Array(documents)
    }

    pub fn tag(&self, tag: CodeTag<'arena>, alignment: &'arena [u8]) -> Document<'arena, A> {
        let document = self.group([
            self.raw(tag.open),
            Document::Indent(self.list([Document::Line(Line::default()), self.header(tag)])),
            Document::Line(Line::default()),
            self.raw(tag.close),
        ]);
        if alignment.is_empty() {
            document
        } else {
            Document::Align(Align { alignment, contents: self.list([document]) })
        }
    }
}
