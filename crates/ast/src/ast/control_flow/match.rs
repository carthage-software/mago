use bumpalo::boxed::Box;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::sequence::TokenSeparatedSequence;

/// Represents a PHP match expression.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Match<'a> {
    pub r#match: Keyword,
    pub left_parenthesis: Span,
    pub expression: Box<'a, Expression<'a>>,
    pub right_parenthesis: Span,
    pub left_brace: Span,
    pub arms: TokenSeparatedSequence<'a, MatchArm<'a>>,
    pub right_brace: Span,
}

/// Represents a single arm within a match expression.
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum MatchArm<'a> {
    Expression(MatchExpressionArm<'a>),
    Default(MatchDefaultArm<'a>),
}

/// Represents a single arm within a match statement.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct MatchExpressionArm<'a> {
    pub conditions: TokenSeparatedSequence<'a, Expression<'a>>,
    pub arrow: Span,
    pub expression: Box<'a, Expression<'a>>,
}

/// Represents the default arm within a match statement.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct MatchDefaultArm<'a> {
    pub default: Keyword,
    pub arrow: Span,
    pub expression: Box<'a, Expression<'a>>,
}

impl HasSpan for Match<'_> {
    fn span(&self) -> Span {
        Span::between(self.r#match.span(), self.right_brace)
    }
}

impl HasSpan for MatchArm<'_> {
    fn span(&self) -> Span {
        match &self {
            MatchArm::Expression(e) => e.span(),
            MatchArm::Default(d) => d.span(),
        }
    }
}

impl HasSpan for MatchExpressionArm<'_> {
    fn span(&self) -> Span {
        Span::between(self.conditions.span(self.arrow.start), self.expression.span())
    }
}

impl HasSpan for MatchDefaultArm<'_> {
    fn span(&self) -> Span {
        Span::between(self.default.span(), self.expression.span())
    }
}
