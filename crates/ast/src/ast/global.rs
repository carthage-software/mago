use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::keyword::Keyword;
use crate::ast::terminator::Terminator;
use crate::ast::variable::Variable;
use crate::sequence::TokenSeparatedSequence;

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Global<'a> {
    pub global: Keyword,
    pub variables: TokenSeparatedSequence<'a, Variable<'a>>,
    pub terminator: Terminator,
}

impl HasSpan for Global<'_> {
    fn span(&self) -> Span {
        self.global.span().join(self.terminator.span())
    }
}
