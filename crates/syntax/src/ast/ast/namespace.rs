use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::block::Block;
use crate::ast::ast::identifier::Identifier;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::statement::Statement;
use crate::ast::ast::terminator::Terminator;

use crate::ast::sequence::Sequence;

/// Represents a PHP `namespace` declaration.
///
/// # Examples
///
/// ```php
/// <?php
///
/// namespace Foo\Bar {
///    // ...
/// }
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Namespace {
    pub namespace: Keyword,
    pub name: Option<Identifier>,
    pub body: NamespaceBody,
}

/// Represents the body of a PHP `namespace` declaration.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum NamespaceBody {
    Implicit(NamespaceImplicitBody),
    BraceDelimited(Block),
}

/// Represents an implicit body of a PHP `namespace` declaration.
///
/// # Examples
///
/// ```php
/// <?php
///
/// namespace Foo\Bar;
///
/// // ...
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct NamespaceImplicitBody {
    pub terminator: Terminator,
    pub statements: Sequence<Statement>,
}

impl Namespace {
    pub fn statements(&self) -> &Sequence<Statement> {
        match &self.body {
            NamespaceBody::Implicit(body) => &body.statements,
            NamespaceBody::BraceDelimited(body) => &body.statements,
        }
    }
}

impl HasSpan for Namespace {
    fn span(&self) -> Span {
        self.namespace.span().join(self.body.span())
    }
}

impl HasSpan for NamespaceBody {
    fn span(&self) -> Span {
        match self {
            NamespaceBody::Implicit(body) => body.span(),
            NamespaceBody::BraceDelimited(body) => body.span(),
        }
    }
}

impl HasSpan for NamespaceImplicitBody {
    fn span(&self) -> Span {
        self.terminator.span().join(self.statements.span(self.terminator.span().end))
    }
}
