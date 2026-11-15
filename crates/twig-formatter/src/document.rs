use std::cell::Cell;

use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_formatter_core::document::{Align, BreakMode, Document, Group, Line};
use mago_span::{HasSpan, Span};
use mago_twig_syntax::cst::{Expression, TokenSeparatedSequence};

use crate::expression::operator::starts_with_not;
use crate::tag::CodeTag;

pub(super) struct DocumentBuilder<'arena, A: Arena> {
    pub arena: &'arena A,
    pub source: &'arena [u8],
    pub preserved_break: Cell<bool>,
}

impl<'arena, A: Arena> DocumentBuilder<'arena, A> {
    /// Whether the source breaks the line between two offsets.
    pub(super) fn breaks_between(&self, start: u32, end: u32) -> bool {
        start < end && memchr::memchr2(b'\n', b'\r', &self.source[start as usize..end as usize]).is_some()
    }

    /// A group that keeps the line break the source has, as `preserve-breaking-array-like` does for PHP.
    pub(super) fn preserved_group(&self, group: Document<'arena, A>, preserve: bool) -> Document<'arena, A> {
        match group {
            Document::Group(group) if preserve => {
                self.preserved_break.set(true);
                Document::Group(group.with_break_mode(BreakMode::Preserve))
            }
            document => document,
        }
    }

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
        self.preserved_break.set(false);
        let header = self.header(tag);
        let has_preserved_break = self.preserved_break.replace(false);
        let contents = &self.source[tag.open.end.offset as usize..tag.close.start.offset as usize];
        let leading = contents.len() - contents.trim_ascii_start().len();
        let trailing = contents.len() - contents.trim_ascii_end().len();
        let breaks_after_open = self.breaks_between(tag.open.end.offset, tag.open.end.offset + leading as u32);
        let breaks_before_close = self.breaks_between(tag.close.start.offset - trailing as u32, tag.close.start.offset);

        let document = if has_preserved_break && !breaks_after_open {
            self.array([
                self.raw(tag.open),
                Document::space(),
                header,
                if breaks_before_close { Document::Line(Line::hard()) } else { Document::space() },
                self.raw(tag.close),
            ])
        } else {
            self.preserved_group(
                self.group([
                    self.raw(tag.open),
                    Document::Indent(self.list([Document::Line(Line::default()), header])),
                    Document::Line(Line::default()),
                    self.raw(tag.close),
                ]),
                has_preserved_break,
            )
        };

        if alignment.is_empty() {
            document
        } else {
            Document::Align(Align { alignment, contents: self.list([document]) })
        }
    }
}
