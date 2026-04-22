use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::ast::Sequence;
use crate::ast::expression::Expression;
use crate::ast::statement::Statement;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct BlockShort<'arena> {
    pub expression: Expression<'arena>,
    pub close_tag: Span,
}

impl HasSpan for BlockShort<'_> {
    fn span(&self) -> Span {
        self.expression.span().join(self.close_tag)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct BlockLong<'arena> {
    pub close_tag: Span,
    pub body: Sequence<'arena, Statement<'arena>>,
    pub end_open_tag: Span,
    pub end_keyword: Keyword<'arena>,
    pub end_name: Option<Identifier<'arena>>,
    pub end_close_tag: Span,
}

impl HasSpan for BlockLong<'_> {
    fn span(&self) -> Span {
        self.close_tag.join(self.end_close_tag)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum BlockBody<'arena> {
    Short(BlockShort<'arena>),
    Long(BlockLong<'arena>),
}

impl HasSpan for BlockBody<'_> {
    fn span(&self) -> Span {
        match self {
            BlockBody::Short(s) => s.span(),
            BlockBody::Long(l) => l.span(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Block<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub name: Identifier<'arena>,
    pub body: BlockBody<'arena>,
}

impl HasSpan for Block<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.body.span())
    }
}
