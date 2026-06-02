use mago_syntax_core::cst::TokenSeparatedSequence;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::expression::ConstantExpression;
use crate::cst::keyword::Keyword;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ArrayConstant<'arena> {
    pub keyword: Option<Keyword<'arena>>,
    pub left_delimiter: Span,
    pub items: TokenSeparatedSequence<'arena, ArrayConstantItem<'arena>, Span>,
    pub right_delimiter: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ArrayConstantItem<'arena> {
    pub key: Option<&'arena ConstantExpression<'arena>>,
    pub double_arrow: Option<Span>,
    pub value: &'arena ConstantExpression<'arena>,
}

impl HasSpan for ArrayConstant<'_> {
    fn span(&self) -> Span {
        match &self.keyword {
            Some(keyword) => keyword.span().join(self.right_delimiter),
            None => self.left_delimiter.join(self.right_delimiter),
        }
    }
}

impl HasSpan for ArrayConstantItem<'_> {
    fn span(&self) -> Span {
        let start = self.key.map_or_else(|| self.value.span(), HasSpan::span);

        start.join(self.value.span())
    }
}
