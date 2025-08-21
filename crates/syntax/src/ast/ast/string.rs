use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::sequence::Sequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum CompositeString<'arena> {
    ShellExecute(ShellExecuteString<'arena>),
    Interpolated(InterpolatedString<'arena>),
    Document(DocumentString<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ShellExecuteString<'arena> {
    pub left_backtick: Span,
    pub parts: Sequence<'arena, StringPart<'arena>>,
    pub right_backtick: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct InterpolatedString<'arena> {
    pub left_double_quote: Span,
    pub parts: Sequence<'arena, StringPart<'arena>>,
    pub right_double_quote: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum DocumentKind {
    Heredoc,
    Nowdoc,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum DocumentIndentation {
    None,
    Whitespace(usize),
    Tab(usize),
    Mixed(usize, usize),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct DocumentString<'arena> {
    pub open: Span,
    pub kind: DocumentKind,
    pub indentation: DocumentIndentation,
    pub label: &'arena str,
    pub parts: Sequence<'arena, StringPart<'arena>>,
    pub close: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum StringPart<'arena> {
    Literal(LiteralStringPart<'arena>),
    Expression(&'arena Expression<'arena>),
    BracedExpression(BracedExpressionStringPart<'arena>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct LiteralStringPart<'arena> {
    pub span: Span,
    pub value: &'arena str,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct BracedExpressionStringPart<'arena> {
    pub left_brace: Span,
    pub expression: &'arena Expression<'arena>,
    pub right_brace: Span,
}

impl<'arena> CompositeString<'arena> {
    pub fn parts(&self) -> &Sequence<'arena, StringPart<'arena>> {
        match self {
            CompositeString::ShellExecute(s) => &s.parts,
            CompositeString::Interpolated(i) => &i.parts,
            CompositeString::Document(d) => &d.parts,
        }
    }
}

impl HasSpan for CompositeString<'_> {
    fn span(&self) -> Span {
        match self {
            CompositeString::ShellExecute(s) => s.span(),
            CompositeString::Interpolated(i) => i.span(),
            CompositeString::Document(d) => d.span(),
        }
    }
}

impl HasSpan for ShellExecuteString<'_> {
    fn span(&self) -> Span {
        self.left_backtick.join(self.right_backtick)
    }
}

impl HasSpan for InterpolatedString<'_> {
    fn span(&self) -> Span {
        self.left_double_quote.join(self.right_double_quote)
    }
}

impl HasSpan for DocumentString<'_> {
    fn span(&self) -> Span {
        self.open
    }
}

impl HasSpan for StringPart<'_> {
    fn span(&self) -> Span {
        match self {
            StringPart::Literal(l) => l.span(),
            StringPart::Expression(e) => e.span(),
            StringPart::BracedExpression(b) => b.span(),
        }
    }
}

impl HasSpan for LiteralStringPart<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for BracedExpressionStringPart<'_> {
    fn span(&self) -> Span {
        self.left_brace.join(self.right_brace)
    }
}
