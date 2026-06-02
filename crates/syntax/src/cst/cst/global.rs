use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::cst::keyword::Keyword;
use crate::cst::cst::terminator::Terminator;
use crate::cst::cst::variable::Variable;
use crate::cst::sequence::TokenSeparatedSequence;

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
