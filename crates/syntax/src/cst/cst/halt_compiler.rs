use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::cst::keyword::Keyword;
use crate::cst::cst::terminator::Terminator;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct HaltCompiler<'arena> {
    pub halt_compiler: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub right_parenthesis: Span,
    pub terminator: Terminator<'arena>,
}

impl HasSpan for HaltCompiler<'_> {
    fn span(&self) -> Span {
        self.halt_compiler.span().join(self.terminator.span())
    }
}
