use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::statement::Statement;
use crate::ast::ast::terminator::Terminator;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;

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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct For<'arena> {
    pub r#for: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub initializations: TokenSeparatedSequence<'arena, Expression<'arena>>,
    pub initializations_semicolon: Span,
    pub conditions: TokenSeparatedSequence<'arena, Expression<'arena>>,
    pub conditions_semicolon: Span,
    pub increments: TokenSeparatedSequence<'arena, Expression<'arena>>,
    pub right_parenthesis: Span,
    pub body: ForBody<'arena>,
}

/// Represents the body of a for statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum ForBody<'arena> {
    Statement(&'arena Statement<'arena>),
    ColonDelimited(ForColonDelimitedBody<'arena>),
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ForColonDelimitedBody<'arena> {
    pub colon: Span,
    pub statements: Sequence<'arena, Statement<'arena>>,
    pub end_for: Keyword<'arena>,
    pub terminator: Terminator<'arena>,
}

impl<'arena> ForBody<'arena> {
    #[inline]
    pub fn statements(&self) -> &[Statement<'arena>] {
        match self {
            ForBody::Statement(statement) => std::slice::from_ref(statement),
            ForBody::ColonDelimited(body) => body.statements.as_slice(),
        }
    }
}

impl HasSpan for For<'_> {
    fn span(&self) -> Span {
        self.r#for.span().join(self.body.span())
    }
}

impl HasSpan for ForBody<'_> {
    fn span(&self) -> Span {
        match self {
            ForBody::Statement(statement) => statement.span(),
            ForBody::ColonDelimited(body) => body.span(),
        }
    }
}

impl HasSpan for ForColonDelimitedBody<'_> {
    fn span(&self) -> Span {
        self.colon.join(self.terminator.span())
    }
}
