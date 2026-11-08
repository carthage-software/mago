use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_formatter_core::document::Document;
use mago_twig_syntax::cst::{
    Cache, CacheOption, Deprecated, From, IgnoreMissingClause, Keyword, Use, WithExpressionClause,
};

use crate::document::DocumentBuilder;

impl<'arena, A: Arena> DocumentBuilder<'arena, A> {
    pub(super) fn include_options(
        &self,
        ignore_missing: Option<&IgnoreMissingClause<'arena>>,
        variables: Option<&WithExpressionClause<'arena>>,
        only: Option<Keyword<'arena>>,
    ) -> Document<'arena, A> {
        self.array([
            ignore_missing.map_or_else(Document::empty, |_| Document::String(b" ignore missing")),
            variables.map_or_else(Document::empty, |clause| {
                self.array([Document::String(b" with "), self.expression(&clause.variables)])
            }),
            only.map_or_else(Document::empty, |_| Document::String(b" only")),
        ])
    }

    pub(super) fn use_header(&self, node: &Use<'arena>) -> Document<'arena, A> {
        let aliases = self.separated(&node.aliases, |alias| {
            self.array([
                self.raw(alias.from.span),
                alias
                    .to
                    .map_or_else(Document::empty, |name| self.array([Document::String(b" as "), self.raw(name.span)])),
            ])
        });

        self.array([
            self.keyword_expression(node.keyword.span, &node.template),
            node.with_keyword.map_or_else(Document::empty, |_| self.array([Document::String(b" with "), aliases])),
        ])
    }

    pub(super) fn macro_import_header(&self, node: &From<'arena>) -> Document<'arena, A> {
        let names = self.separated(&node.names, |name| {
            self.array([
                self.raw(name.from.span),
                name.to
                    .map_or_else(Document::empty, |name| self.array([Document::String(b" as "), self.raw(name.span)])),
            ])
        });

        self.array([self.keyword_expression(node.keyword.span, &node.template), Document::String(b" import "), names])
    }

    pub(super) fn deprecated_header(&self, node: &Deprecated<'arena>) -> Document<'arena, A> {
        let mut documents = Vec::with_capacity_in(1 + node.options.len() * 4, self.arena);
        documents.push(self.keyword_expression(node.keyword.span, &node.message));
        for option in node.options {
            documents.push(Document::space());
            documents.push(self.raw(option.name.span));
            documents.push(Document::String(b" = "));
            documents.push(self.expression(&option.value));
        }

        Document::Array(documents)
    }

    pub(super) fn cache_header(&self, node: &Cache<'arena>) -> Document<'arena, A> {
        let mut documents = Vec::with_capacity_in(3, self.arena);
        documents.push(self.keyword_expression(node.keyword.span, &node.key));
        documents.extend(node.options().map(|option| self.cache_option(option)));
        Document::Array(documents)
    }

    fn cache_option(&self, option: &CacheOption<'arena>) -> Document<'arena, A> {
        self.array([
            Document::space(),
            self.raw(option.keyword.span),
            self.enclosed(
                self.raw(option.left_parenthesis),
                self.expression(&option.value),
                self.raw(option.right_parenthesis),
            ),
        ])
    }
}
