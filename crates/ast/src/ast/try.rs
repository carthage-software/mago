use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::block::Block;
use crate::ast::keyword::Keyword;
use crate::ast::type_hint::Hint;
use crate::ast::variable::DirectVariable;
use crate::sequence::Sequence;

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Try<'a> {
    pub r#try: Keyword,
    pub block: Block<'a>,
    pub catch_clauses: Sequence<'a, TryCatchClause<'a>>,
    pub finally_clause: Option<TryFinallyClause<'a>>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct TryCatchClause<'a> {
    pub r#catch: Keyword,
    pub left_parenthesis: Span,
    pub hint: Hint<'a>,
    pub variable: Option<DirectVariable>,
    pub right_parenthesis: Span,
    pub block: Block<'a>,
}

#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct TryFinallyClause<'a> {
    pub r#finally: Keyword,
    pub block: Block<'a>,
}

impl HasSpan for Try<'_> {
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

impl HasSpan for TryCatchClause<'_> {
    fn span(&self) -> Span {
        Span::between(self.r#catch.span(), self.block.span())
    }
}

impl HasSpan for TryFinallyClause<'_> {
    fn span(&self) -> Span {
        Span::between(self.r#finally.span(), self.block.span())
    }
}
