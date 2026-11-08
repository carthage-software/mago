use mago_allocator::Arena;
use mago_formatter_core::document::Document;
use mago_span::Span;
use mago_twig_syntax::cst::{BlockBody, Expression, SetBody};

use crate::document::DocumentBuilder;
use crate::tag::{CodeTag, TagContents};

mod declarations;
mod templates;

impl<'arena, A: Arena> DocumentBuilder<'arena, A> {
    pub(super) fn header(&self, tag: CodeTag<'arena>) -> Document<'arena, A> {
        match tag.contents {
            TagContents::Print(expression) => self.expression(expression),
            TagContents::If(branch) => self.keyword_expression(branch.keyword.span, &branch.condition),
            TagContents::For(node) => self.array([
                self.raw(node.keyword.span),
                Document::space(),
                self.separated(&node.targets, |target| self.raw(target.span)),
                Document::String(b" in "),
                self.expression(&node.sequence),
                node.if_clause.as_ref().map_or_else(Document::empty, |clause| {
                    self.array([Document::String(b" if "), self.expression(&clause.condition)])
                }),
            ]),
            TagContents::Set(node) => self.array([
                self.raw(node.keyword.span),
                Document::space(),
                self.separated(&node.names, |name| self.raw(name.span)),
                match &node.body {
                    SetBody::Inline(body) => self.array([
                        Document::String(b" = "),
                        self.separated(&body.values, |value| self.expression(value)),
                    ]),
                    SetBody::Capture(_) => Document::empty(),
                },
            ]),
            TagContents::Block(node) => self.array([
                self.raw(node.keyword.span),
                Document::space(),
                self.raw(node.name.span),
                match &node.body {
                    BlockBody::Short(body) => self.array([Document::space(), self.expression(&body.expression)]),
                    BlockBody::Long(_) => Document::empty(),
                },
            ]),
            TagContents::Extends(node) => self.keyword_expression(node.keyword.span, &node.template),
            TagContents::Use(node) => self.use_header(node),
            TagContents::Include(node) => self.array([
                self.keyword_expression(node.keyword.span, &node.template),
                self.include_options(node.ignore_missing.as_ref(), node.with_clause.as_ref(), node.only_keyword),
            ]),
            TagContents::Embed(node) => self.array([
                self.keyword_expression(node.keyword.span, &node.template),
                self.include_options(node.ignore_missing.as_ref(), node.with_clause.as_ref(), node.only_keyword),
            ]),
            TagContents::Import(node) => self.array([
                self.keyword_expression(node.keyword.span, &node.template),
                Document::String(b" as "),
                self.raw(node.alias.span),
            ]),
            TagContents::From(node) => self.macro_import_header(node),
            TagContents::Macro(node) => self.macro_header(node),
            TagContents::With(node) => self.array([
                self.raw(node.keyword.span),
                self.optional_expression(node.variables.as_ref()),
                node.only_keyword.map_or_else(Document::empty, |_| Document::String(b" only")),
            ]),
            TagContents::Apply(node) => self.apply_header(node),
            TagContents::Autoescape(node) => {
                self.array([self.raw(node.keyword.span), self.optional_expression(node.strategy.as_ref())])
            }
            TagContents::Sandbox(node) => self.raw(node.keyword.span),
            TagContents::Deprecated(node) => self.deprecated_header(node),
            TagContents::Do(node) => self.keyword_expression(node.keyword.span, &node.expression),
            TagContents::Flush(node) => self.raw(node.keyword.span),
            TagContents::Guard(node) => self.array([
                self.raw(node.keyword.span),
                Document::space(),
                self.raw(node.kind_keyword.span),
                Document::space(),
                self.raw(node.name.span),
                node.second_word
                    .map_or_else(Document::empty, |word| self.array([Document::space(), self.raw(word.span)])),
            ]),
            TagContents::Cache(node) => self.cache_header(node),
            TagContents::Types(node) => self.types_header(node),
            TagContents::Verbatim(node) => self.raw(node.keyword.span),
            TagContents::Keyword => {
                self.words(&self.source[tag.open.end.offset as usize..tag.close.start.offset as usize])
            }
        }
    }

    fn keyword_expression(&self, keyword: Span, expression: &Expression<'arena>) -> Document<'arena, A> {
        self.array([self.raw(keyword), Document::space(), self.expression(expression)])
    }

    fn optional_expression(&self, expression: Option<&Expression<'arena>>) -> Document<'arena, A> {
        expression
            .map_or_else(Document::empty, |expression| self.array([Document::space(), self.expression(expression)]))
    }
}
