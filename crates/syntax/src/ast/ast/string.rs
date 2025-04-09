use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_interner::StringIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::sequence::Sequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum CompositeString {
    ShellExecute(ShellExecuteString),
    Interpolated(InterpolatedString),
    Document(DocumentString),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ShellExecuteString {
    pub left_backtick: Span,
    pub parts: Sequence<StringPart>,
    pub right_backtick: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct InterpolatedString {
    pub left_double_quote: Span,
    pub parts: Sequence<StringPart>,
    pub right_double_quote: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C)]
pub enum DocumentKind {
    Heredoc,
    Nowdoc,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum DocumentIndentation {
    None,
    Whitespace(usize),
    Tab(usize),
    Mixed(usize, usize),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct DocumentString {
    pub open: Span,
    pub kind: DocumentKind,
    pub indentation: DocumentIndentation,
    pub label: StringIdentifier,
    pub parts: Sequence<StringPart>,
    pub close: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum StringPart {
    Literal(LiteralStringPart),
    Expression(Box<Expression>),
    BracedExpression(BracedExpressionStringPart),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct LiteralStringPart {
    pub span: Span,
    pub value: StringIdentifier,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct BracedExpressionStringPart {
    pub left_brace: Span,
    pub expression: Box<Expression>,
    pub right_brace: Span,
}

impl CompositeString {
    pub fn parts(&self) -> &Sequence<StringPart> {
        match self {
            CompositeString::ShellExecute(s) => &s.parts,
            CompositeString::Interpolated(i) => &i.parts,
            CompositeString::Document(d) => &d.parts,
        }
    }
}

impl HasSpan for CompositeString {
    fn span(&self) -> Span {
        match self {
            CompositeString::ShellExecute(s) => s.span(),
            CompositeString::Interpolated(i) => i.span(),
            CompositeString::Document(d) => d.span(),
        }
    }
}

impl HasSpan for ShellExecuteString {
    fn span(&self) -> Span {
        self.left_backtick.join(self.right_backtick)
    }
}

impl HasSpan for InterpolatedString {
    fn span(&self) -> Span {
        self.left_double_quote.join(self.right_double_quote)
    }
}

impl HasSpan for DocumentString {
    fn span(&self) -> Span {
        self.open
    }
}

impl HasSpan for StringPart {
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

impl HasSpan for BracedExpressionStringPart {
    fn span(&self) -> Span {
        self.left_brace.join(self.right_brace)
    }
}
