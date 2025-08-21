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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Namespace<'arena> {
    pub namespace: Keyword<'arena>,
    pub name: Option<Identifier<'arena>>,
    pub body: NamespaceBody<'arena>,
}

/// Represents the body of a PHP `namespace` declaration.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum NamespaceBody<'arena> {
    Implicit(NamespaceImplicitBody<'arena>),
    BraceDelimited(Block<'arena>),
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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct NamespaceImplicitBody<'arena> {
    pub terminator: Terminator<'arena>,
    pub statements: Sequence<'arena, Statement<'arena>>,
}

impl<'arena> Namespace<'arena> {
    pub fn statements(&self) -> &Sequence<'arena, Statement<'arena>> {
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
        let terminator_span = self.terminator.span();

        terminator_span.join(self.statements.span(terminator_span.file_id, terminator_span.end))
    }
}
