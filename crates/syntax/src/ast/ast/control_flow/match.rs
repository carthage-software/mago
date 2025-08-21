use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::sequence::TokenSeparatedSequence;

/// Represents a PHP match expression.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Match<'arena> {
    pub r#match: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub expression: &'arena Expression<'arena>,
    pub right_parenthesis: Span,
    pub left_brace: Span,
    pub arms: TokenSeparatedSequence<'arena, MatchArm<'arena>>,
    pub right_brace: Span,
}

/// Represents a single arm within a match expression.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum MatchArm<'arena> {
    Expression(MatchExpressionArm<'arena>),
    Default(MatchDefaultArm<'arena>),
}

/// Represents a single arm within a match statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MatchExpressionArm<'arena> {
    pub conditions: TokenSeparatedSequence<'arena, Expression<'arena>>,
    pub arrow: Span,
    pub expression: &'arena Expression<'arena>,
}

/// Represents the default arm within a match statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MatchDefaultArm<'arena> {
    pub default: Keyword<'arena>,
    pub arrow: Span,
    pub expression: &'arena Expression<'arena>,
}

impl<'arena> MatchArm<'arena> {
    #[inline]
    pub const fn is_default(&self) -> bool {
        matches!(self, MatchArm::Default(_))
    }

    #[inline]
    pub const fn is_conditional(&self) -> bool {
        matches!(self, MatchArm::Expression(_))
    }

    #[inline]
    pub fn expression(&self) -> &Expression<'arena> {
        match self {
            MatchArm::Expression(arm) => arm.expression,
            MatchArm::Default(arm) => arm.expression,
        }
    }
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
        Span::between(self.conditions.span(self.arrow.file_id, self.arrow.start), self.expression.span())
    }
}

impl HasSpan for MatchDefaultArm<'_> {
    fn span(&self) -> Span {
        Span::between(self.default.span(), self.expression.span())
    }
}
