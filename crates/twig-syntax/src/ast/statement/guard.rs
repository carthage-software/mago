use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::ast::Sequence;
use crate::ast::statement::Statement;
use crate::ast::statement::r#if::ElseBranch;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum GuardKind {
    Function,
    Filter,
    Test,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Guard<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub kind_keyword: Keyword<'arena>,
    pub kind: GuardKind,
    pub name: Identifier<'arena>,
    pub second_word: Option<Identifier<'arena>>,
    pub close_tag: Span,
    pub body: Sequence<'arena, Statement<'arena>>,
    pub else_branch: Option<ElseBranch<'arena>>,
    pub end_open_tag: Span,
    pub end_keyword: Keyword<'arena>,
    pub end_close_tag: Span,
}

impl HasSpan for Guard<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.end_close_tag)
    }
}
