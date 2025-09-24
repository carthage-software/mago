use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::terminator::Terminator;
use crate::ast::sequence::TokenSeparatedSequence;

/// Represents a PHP `<?=` statement.
///
/// # Examples
///
/// ```php
/// <?= "Hello, World!" ?>
/// <?= $a, $b, $c;
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct EchoTag<'arena> {
    pub tag: Span,
    pub values: TokenSeparatedSequence<'arena, Expression<'arena>>,
    pub terminator: Terminator<'arena>,
}

/// Represents a PHP `echo` statement.
///
/// # Examples
///
/// ```php
/// <?php
///
/// echo "Hello, World!";
/// echo $a, $b, $c;
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Echo<'arena> {
    pub echo: Keyword<'arena>,
    pub values: TokenSeparatedSequence<'arena, Expression<'arena>>,
    pub terminator: Terminator<'arena>,
}

impl HasSpan for EchoTag<'_> {
    fn span(&self) -> Span {
        self.tag.join(self.terminator.span())
    }
}

impl HasSpan for Echo<'_> {
    fn span(&self) -> Span {
        self.echo.span().join(self.terminator.span())
    }
}
