use bumpalo::collections::Vec as BVec;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Number<'arena> {
    pub raw: &'arena str,
    pub is_float: bool,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct StringLiteral<'arena> {
    pub raw: &'arena str,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct InterpolatedLiteral<'arena> {
    pub value: &'arena str,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Interpolation<'arena> {
    pub open_brace: Span,
    pub expression: &'arena Expression<'arena>,
    pub close_brace: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum StringPart<'arena> {
    Literal(InterpolatedLiteral<'arena>),
    Interpolation(Interpolation<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct InterpolatedString<'arena> {
    pub open_quote: Span,
    pub parts: BVec<'arena, StringPart<'arena>>,
    pub close_quote: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Bool {
    pub value: bool,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Null {
    pub span: Span,
}

impl HasSpan for Number<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for StringLiteral<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for InterpolatedLiteral<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for Interpolation<'_> {
    fn span(&self) -> Span {
        self.open_brace.join(self.close_brace)
    }
}

impl HasSpan for StringPart<'_> {
    fn span(&self) -> Span {
        match self {
            StringPart::Literal(l) => l.span(),
            StringPart::Interpolation(i) => i.span(),
        }
    }
}

impl HasSpan for InterpolatedString<'_> {
    fn span(&self) -> Span {
        self.open_quote.join(self.close_quote)
    }
}

impl HasSpan for Bool {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for Null {
    fn span(&self) -> Span {
        self.span
    }
}
