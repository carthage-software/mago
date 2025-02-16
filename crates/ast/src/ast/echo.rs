use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::ast::terminator::Terminator;
use crate::sequence::TokenSeparatedSequence;

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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Echo<'a> {
    pub echo: Keyword,
    pub values: TokenSeparatedSequence<'a, Expression<'a>>,
    pub terminator: Terminator,
}

impl HasSpan for Echo<'_> {
    fn span(&self) -> Span {
        self.echo.span().join(self.terminator.span())
    }
}
