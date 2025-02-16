use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::block::Block;
use crate::ast::identifier::Identifier;
use crate::ast::keyword::Keyword;
use crate::ast::statement::Statement;
use crate::ast::terminator::Terminator;

use crate::sequence::Sequence;

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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Namespace<'a> {
    pub namespace: Keyword,
    pub name: Option<Identifier>,
    pub body: NamespaceBody<'a>,
}

/// Represents the body of a PHP `namespace` declaration.
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum NamespaceBody<'a> {
    Implicit(NamespaceImplicitBody<'a>),
    BraceDelimited(Block<'a>),
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct NamespaceImplicitBody<'a> {
    pub terminator: Terminator,
    pub statements: Sequence<'a, Statement<'a>>,
}

impl<'a> Namespace<'a> {
    pub fn statements(&self) -> &Sequence<Statement<'a>> {
        match &self.body {
            NamespaceBody::Implicit(body) => &body.statements,
            NamespaceBody::BraceDelimited(body) => &body.statements,
        }
    }
}

impl HasSpan for Namespace<'_> {
    fn span(&self) -> Span {
        self.namespace.span().join(self.body.span())
    }
}

impl HasSpan for NamespaceBody<'_> {
    fn span(&self) -> Span {
        match self {
            NamespaceBody::Implicit(body) => body.span(),
            NamespaceBody::BraceDelimited(body) => body.span(),
        }
    }
}

impl HasSpan for NamespaceImplicitBody<'_> {
    fn span(&self) -> Span {
        self.terminator.span().join(self.statements.span(self.terminator.span().end))
    }
}
