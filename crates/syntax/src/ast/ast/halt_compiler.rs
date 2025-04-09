use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::terminator::Terminator;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct HaltCompiler {
    pub halt_compiler: Keyword,
    pub left_parenthesis: Span,
    pub right_parenthesis: Span,
    pub terminator: Terminator,
}

impl HasSpan for HaltCompiler {
    fn span(&self) -> Span {
        self.halt_compiler.span().join(self.terminator.span())
    }
}
