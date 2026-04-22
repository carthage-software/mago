use bumpalo::collections::Vec as BVec;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Identifier;
use crate::ast::Keyword;
use crate::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct DeprecatedOption<'arena> {
    pub name: Identifier<'arena>,
    pub equal: Span,
    pub value: Expression<'arena>,
}

impl HasSpan for DeprecatedOption<'_> {
    fn span(&self) -> Span {
        self.name.span.join(self.value.span())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Deprecated<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub message: Expression<'arena>,
    pub options: BVec<'arena, DeprecatedOption<'arena>>,
    pub close_tag: Span,
}

impl HasSpan for Deprecated<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.close_tag)
    }
}
