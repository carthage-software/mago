use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::block::Block;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::type_hint::Hint;
use crate::ast::ast::variable::DirectVariable;
use crate::ast::sequence::Sequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Try {
    pub r#try: Keyword,
    pub block: Block,
    pub catch_clauses: Sequence<TryCatchClause>,
    pub finally_clause: Option<TryFinallyClause>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct TryCatchClause {
    pub r#catch: Keyword,
    pub left_parenthesis: Span,
    pub hint: Hint,
    pub variable: Option<DirectVariable>,
    pub right_parenthesis: Span,
    pub block: Block,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct TryFinallyClause {
    pub r#finally: Keyword,
    pub block: Block,
}

impl HasSpan for Try {
    fn span(&self) -> Span {
        match &self.finally_clause {
            Some(finally) => Span::between(self.r#try.span(), finally.span()),
            None => match self.catch_clauses.iter().last() {
                Some(catch_block) => Span::between(self.r#try.span(), catch_block.span()),
                None => Span::between(self.r#try.span(), self.block.span()),
            },
        }
    }
}

impl HasSpan for TryCatchClause {
    fn span(&self) -> Span {
        Span::between(self.r#catch.span(), self.block.span())
    }
}

impl HasSpan for TryFinallyClause {
    fn span(&self) -> Span {
        Span::between(self.r#finally.span(), self.block.span())
    }
}
