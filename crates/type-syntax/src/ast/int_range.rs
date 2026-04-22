use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::keyword::Keyword;
use crate::ast::literal::LiteralIntType;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum IntOrKeyword<'arena> {
    NegativeInt { minus: Span, int: LiteralIntType<'arena> },
    Int(LiteralIntType<'arena>),
    Keyword(Keyword<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IntRangeType<'arena> {
    pub keyword: Keyword<'arena>,
    pub less_than: Span,
    pub min: IntOrKeyword<'arena>,
    pub comma: Span,
    pub max: IntOrKeyword<'arena>,
    pub greater_than: Span,
}

impl HasSpan for IntOrKeyword<'_> {
    fn span(&self) -> Span {
        match self {
            IntOrKeyword::NegativeInt { minus, int } => minus.join(int.span()),
            IntOrKeyword::Int(literal) => literal.span(),
            IntOrKeyword::Keyword(keyword) => keyword.span(),
        }
    }
}

impl HasSpan for IntRangeType<'_> {
    fn span(&self) -> Span {
        self.keyword.span().join(self.greater_than.span())
    }
}

impl std::fmt::Display for IntRangeType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}<{}..{}>", self.keyword, self.min, self.max)
    }
}

impl std::fmt::Display for IntOrKeyword<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntOrKeyword::NegativeInt { int, .. } => write!(f, "-{int}"),
            IntOrKeyword::Int(int) => write!(f, "{int}"),
            IntOrKeyword::Keyword(keyword) => write!(f, "{keyword}"),
        }
    }
}
