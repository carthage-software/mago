use bumpalo::boxed::Box;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::identifier::LocalIdentifier;
use crate::ast::keyword::Keyword;
use crate::ast::statement::Statement;
use crate::ast::terminator::Terminator;
use crate::sequence::Sequence;
use crate::sequence::TokenSeparatedSequence;

/// Represents the declare construct statement in PHP.
///
/// Example:
///
/// ```php
/// <?php
///
/// declare(strict_types=1);
/// ```
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Declare<'a> {
    pub declare: Keyword,
    pub left_parenthesis: Span,
    pub items: TokenSeparatedSequence<'a, DeclareItem<'a>>,
    pub right_parenthesis: Span,
    pub body: DeclareBody<'a>,
}

/// Represents a single name-value pair within a declare statement.
///
/// Example: `strict_types=1`
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct DeclareItem<'a> {
    pub name: LocalIdentifier,
    pub equal: Span,
    pub value: Box<'a, Expression<'a>>,
}

/// Represents the body of a declare statement.
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum DeclareBody<'a> {
    Statement(Box<'a, Statement<'a>>),
    ColonDelimited(DeclareColonDelimitedBody<'a>),
}

/// Represents a colon-delimited body of a declare statement.
///
/// Example:
///
/// ```php
/// declare(ticks=1):
///   echo "Hello, world!";
///   echo "Goodbye, world!";
/// enddeclare;
/// ```
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct DeclareColonDelimitedBody<'a> {
    pub colon: Span,
    pub statements: Sequence<'a, Statement<'a>>,
    pub end_declare: Keyword,
    pub terminator: Terminator,
}

impl HasSpan for Declare<'_> {
    fn span(&self) -> Span {
        self.declare.span().join(self.body.span())
    }
}

impl HasSpan for DeclareItem<'_> {
    fn span(&self) -> Span {
        self.name.span().join(self.value.span())
    }
}

impl HasSpan for DeclareBody<'_> {
    fn span(&self) -> Span {
        match self {
            DeclareBody::Statement(s) => s.span(),
            DeclareBody::ColonDelimited(c) => c.span(),
        }
    }
}

impl HasSpan for DeclareColonDelimitedBody<'_> {
    fn span(&self) -> Span {
        self.colon.join(self.terminator.span())
    }
}
