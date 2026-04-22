use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::ast::Sequence;
use crate::ast::TokenSeparatedSequence;
use crate::ast::expression::Expression;
use crate::ast::statement::Statement;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct SetInline<'arena> {
    pub equal: Span,
    pub values: TokenSeparatedSequence<'arena, Expression<'arena>>,
    pub close_tag: Span,
}

impl HasSpan for SetInline<'_> {
    fn span(&self) -> Span {
        self.equal.join(self.close_tag)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct SetCapture<'arena> {
    pub close_tag: Span,
    pub body: Sequence<'arena, Statement<'arena>>,
    pub end_open_tag: Span,
    pub end_keyword: Keyword<'arena>,
    pub end_close_tag: Span,
}

impl HasSpan for SetCapture<'_> {
    fn span(&self) -> Span {
        self.close_tag.join(self.end_close_tag)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum SetBody<'arena> {
    Inline(SetInline<'arena>),
    Capture(SetCapture<'arena>),
}

impl HasSpan for SetBody<'_> {
    fn span(&self) -> Span {
        match self {
            SetBody::Inline(i) => i.span(),
            SetBody::Capture(c) => c.span(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Set<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub names: TokenSeparatedSequence<'arena, Identifier<'arena>>,
    pub body: SetBody<'arena>,
}

impl HasSpan for Set<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.body.span())
    }
}
