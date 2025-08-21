use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::terminator::Terminator;
use crate::ast::ast::variable::Variable;
use crate::ast::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Global<'arena> {
    pub global: Keyword<'arena>,
    pub variables: TokenSeparatedSequence<'arena, Variable<'arena>>,
    pub terminator: Terminator<'arena>,
}

impl HasSpan for Global<'_> {
    fn span(&self) -> Span {
        self.global.span().join(self.terminator.span())
    }
}
