use serde::Deserialize;
use serde::Serialize;

use fennec_span::HasSpan;
use fennec_span::Span;

use crate::ast::keyword::Keyword;
use crate::ast::terminator::Terminator;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
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
