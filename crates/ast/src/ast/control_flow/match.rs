use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::sequence::TokenSeparatedSequence;

/// Represents a PHP match expression.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Match {
    pub r#match: Keyword,
    pub left_parenthesis: Span,
    pub expression: Box<Expression>,
    pub right_parenthesis: Span,
    pub left_brace: Span,
    pub arms: TokenSeparatedSequence<MatchArm>,
    pub right_brace: Span,
}

/// Represents a single arm within a match expression.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
pub enum MatchArm {
    Expression(MatchExpressionArm),
    Default(MatchDefaultArm),
}

/// Represents a single arm within a match statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct MatchExpressionArm {
    pub conditions: TokenSeparatedSequence<Expression>,
    pub arrow: Span,
    pub expression: Box<Expression>,
}

/// Represents the default arm within a match statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct MatchDefaultArm {
    pub default: Keyword,
    pub arrow: Span,
    pub expression: Box<Expression>,
}

impl HasSpan for Match {
    fn span(&self) -> Span {
        Span::between(self.r#match.span(), self.right_brace)
    }
}

impl HasSpan for MatchArm {
    fn span(&self) -> Span {
        match &self {
            MatchArm::Expression(e) => e.span(),
            MatchArm::Default(d) => d.span(),
        }
    }
}

impl HasSpan for MatchExpressionArm {
    fn span(&self) -> Span {
        Span::between(self.conditions.span(self.arrow.start), self.expression.span())
    }
}

impl HasSpan for MatchDefaultArm {
    fn span(&self) -> Span {
        Span::between(self.default.span(), self.expression.span())
    }
}
