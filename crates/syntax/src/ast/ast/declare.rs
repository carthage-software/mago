use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::statement::Statement;
use crate::ast::ast::terminator::Terminator;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;

/// Represents the declare construct statement in PHP.
///
/// Example:
///
/// ```php
/// <?php
///
/// declare(strict_types=1);
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Declare<'arena> {
    pub declare: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub items: TokenSeparatedSequence<'arena, DeclareItem<'arena>>,
    pub right_parenthesis: Span,
    pub body: DeclareBody<'arena>,
}

/// Represents a single name-value pair within a declare statement.
///
/// Example: `strict_types=1`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct DeclareItem<'arena> {
    pub name: LocalIdentifier<'arena>,
    pub equal: Span,
    pub value: Expression<'arena>,
}

/// Represents the body of a declare statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum DeclareBody<'arena> {
    Statement(&'arena Statement<'arena>),
    ColonDelimited(DeclareColonDelimitedBody<'arena>),
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct DeclareColonDelimitedBody<'arena> {
    pub colon: Span,
    pub statements: Sequence<'arena, Statement<'arena>>,
    pub end_declare: Keyword<'arena>,
    pub terminator: Terminator<'arena>,
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
