use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::terminator::Terminator;

/// Represents a `goto` statement in PHP.
///
/// Example:
///
/// ```php
/// <?php
///
/// goto foo;
/// ```
///
/// or
///
/// ```php
/// <?php
///
/// goto foo
///
/// ?>
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Goto<'arena> {
    pub goto: Keyword<'arena>,
    pub label: LocalIdentifier<'arena>,
    pub terminator: Terminator<'arena>,
}

/// Represents a Go-To label statement in PHP.
///
/// Example:
///
/// ```php
/// <?php
///
/// foo:
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Label<'arena> {
    pub name: LocalIdentifier<'arena>,
    pub colon: Span,
}

impl HasSpan for Goto<'_> {
    fn span(&self) -> Span {
        Span::between(self.goto.span(), self.terminator.span())
    }
}

impl HasSpan for Label<'_> {
    fn span(&self) -> Span {
        Span::between(self.name.span(), self.colon)
    }
}
