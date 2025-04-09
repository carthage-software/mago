use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::terminator::Terminator;
use crate::ast::sequence::TokenSeparatedSequence;

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
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Echo {
    pub echo: Keyword,
    pub values: TokenSeparatedSequence<Expression>,
    pub terminator: Terminator,
}

impl HasSpan for Echo {
    fn span(&self) -> Span {
        self.echo.span().join(self.terminator.span())
    }
}
