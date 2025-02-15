use serde::Deserialize;
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Declare {
    pub declare: Keyword,
    pub left_parenthesis: Span,
    pub items: TokenSeparatedSequence<DeclareItem>,
    pub right_parenthesis: Span,
    pub body: DeclareBody,
}

/// Represents a single name-value pair within a declare statement.
///
/// Example: `strict_types=1`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct DeclareItem {
    pub name: LocalIdentifier,
    pub equal: Span,
    pub value: Expression,
}

/// Represents the body of a declare statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum DeclareBody {
    Statement(Box<Statement>),
    ColonDelimited(DeclareColonDelimitedBody),
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct DeclareColonDelimitedBody {
    pub colon: Span,
    pub statements: Sequence<Statement>,
    pub end_declare: Keyword,
    pub terminator: Terminator,
}

impl HasSpan for Declare {
    fn span(&self) -> Span {
        self.declare.span().join(self.body.span())
    }
}

impl HasSpan for DeclareItem {
    fn span(&self) -> Span {
        self.name.span().join(self.value.span())
    }
}

impl HasSpan for DeclareBody {
    fn span(&self) -> Span {
        match self {
            DeclareBody::Statement(s) => s.span(),
            DeclareBody::ColonDelimited(c) => c.span(),
        }
    }
}

impl HasSpan for DeclareColonDelimitedBody {
    fn span(&self) -> Span {
        self.colon.join(self.terminator.span())
    }
}
