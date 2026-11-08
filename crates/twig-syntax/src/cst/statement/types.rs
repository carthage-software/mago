use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::Identifier;
use crate::cst::Keyword;
use crate::cst::StringLiteral;
use crate::cst::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TypeDeclaration<'arena> {
    pub name: Identifier<'arena>,
    pub question_mark: Option<Span>,
    pub colon: Span,
    pub value: StringLiteral<'arena>,
}

impl HasSpan for TypeDeclaration<'_> {
    fn span(&self) -> Span {
        self.name.span.join(self.value.span)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Types<'arena> {
    pub open_tag: Span,
    pub keyword: Keyword<'arena>,
    pub left_brace: Option<Span>,
    pub declarations: TokenSeparatedSequence<'arena, TypeDeclaration<'arena>>,
    pub right_brace: Option<Span>,
    pub close_tag: Span,
}

impl HasSpan for Types<'_> {
    fn span(&self) -> Span {
        self.open_tag.join(self.close_tag)
    }
}
