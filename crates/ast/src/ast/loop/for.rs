use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::ast::statement::Statement;
use crate::ast::terminator::Terminator;
use crate::sequence::Sequence;
use crate::sequence::TokenSeparatedSequence;

/// Represents a for statement in PHP.
///
/// Example:
///
/// ```php
/// <?php
///
/// for ($i = 0; $i < 10; $i++) {
///   echo $i;
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct For {
    pub r#for: Keyword,
    pub left_parenthesis: Span,
    pub initializations: TokenSeparatedSequence<Expression>,
    pub initializations_semicolon: Span,
    pub conditions: TokenSeparatedSequence<Expression>,
    pub conditions_semicolon: Span,
    pub increments: TokenSeparatedSequence<Expression>,
    pub right_parenthesis: Span,
    pub body: ForBody,
}

/// Represents the body of a for statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum ForBody {
    Statement(Box<Statement>),
    ColonDelimited(ForColonDelimitedBody),
}

/// Represents a colon-delimited for statement body.
///
/// Example:
///
/// ```php
/// <?php
///
/// for ($i = 0; $i < 10; $i++):
///   echo $i;
/// endfor;
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct ForColonDelimitedBody {
    pub colon: Span,
    pub statements: Sequence<Statement>,
    pub end_for: Keyword,
    pub terminator: Terminator,
}

impl HasSpan for For {
    fn span(&self) -> Span {
        self.r#for.span().join(self.body.span())
    }
}

impl HasSpan for ForBody {
    fn span(&self) -> Span {
        match self {
            ForBody::Statement(statement) => statement.span(),
            ForBody::ColonDelimited(body) => body.span(),
        }
    }
}

impl HasSpan for ForColonDelimitedBody {
    fn span(&self) -> Span {
        self.colon.join(self.terminator.span())
    }
}
