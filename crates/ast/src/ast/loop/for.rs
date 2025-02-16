use bumpalo::boxed::Box;
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct For<'a> {
    pub r#for: Keyword,
    pub left_parenthesis: Span,
    pub initializations: TokenSeparatedSequence<'a, Expression<'a>>,
    pub initializations_semicolon: Span,
    pub conditions: TokenSeparatedSequence<'a, Expression<'a>>,
    pub conditions_semicolon: Span,
    pub increments: TokenSeparatedSequence<'a, Expression<'a>>,
    pub right_parenthesis: Span,
    pub body: ForBody<'a>,
}

/// Represents the body of a for statement.
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum ForBody<'a> {
    Statement(Box<'a, Statement<'a>>),
    ColonDelimited(ForColonDelimitedBody<'a>),
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ForColonDelimitedBody<'a> {
    pub colon: Span,
    pub statements: Sequence<'a, Statement<'a>>,
    pub end_for: Keyword,
    pub terminator: Terminator,
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
