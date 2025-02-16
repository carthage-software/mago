use bumpalo::boxed::Box;
use serde::Serialize;
use strum::Display;

use mago_interner::StringIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::sequence::Sequence;

#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum CompositeString<'a> {
    ShellExecute(ShellExecuteString<'a>),
    Interpolated(InterpolatedString<'a>),
    Document(DocumentString<'a>),
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ShellExecuteString<'a> {
    pub left_backtick: Span,
    pub parts: Sequence<'a, StringPart<'a>>,
    pub right_backtick: Span,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct InterpolatedString<'a> {
    pub left_double_quote: Span,
    pub parts: Sequence<'a, StringPart<'a>>,
    pub right_double_quote: Span,
}

#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C)]
pub enum DocumentKind {
    Heredoc,
    Nowdoc,
}

#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum DocumentIndentation {
    None,
    Whitespace(usize),
    Tab(usize),
    Mixed(usize, usize),
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct DocumentString<'a> {
    pub open: Span,
    pub kind: DocumentKind,
    pub indentation: DocumentIndentation,
    pub label: StringIdentifier,
    pub parts: Sequence<'a, StringPart<'a>>,
    pub close: Span,
}

#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum StringPart<'a> {
    Literal(LiteralStringPart),
    Expression(Box<'a, Expression<'a>>),
    BracedExpression(BracedExpressionStringPart<'a>),
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct LiteralStringPart {
    pub span: Span,
    pub value: StringIdentifier,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct BracedExpressionStringPart<'a> {
    pub left_brace: Span,
    pub expression: Box<'a, Expression<'a>>,
    pub right_brace: Span,
}

impl<'a> CompositeString<'a> {
    pub fn parts(&self) -> &[StringPart<'a>] {
        match self {
            CompositeString::ShellExecute(s) => s.parts.as_slice(),
            CompositeString::Interpolated(i) => i.parts.as_slice(),
            CompositeString::Document(d) => d.parts.as_slice(),
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

impl HasSpan for LiteralStringPart {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for BracedExpressionStringPart<'_> {
    fn span(&self) -> Span {
        self.left_brace.join(self.right_brace)
    }
}
