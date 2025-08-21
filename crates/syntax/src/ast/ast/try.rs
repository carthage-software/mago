use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::block::Block;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::type_hint::Hint;
use crate::ast::ast::variable::DirectVariable;
use crate::ast::sequence::Sequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Try<'arena> {
    pub r#try: Keyword<'arena>,
    pub block: Block<'arena>,
    pub catch_clauses: Sequence<'arena, TryCatchClause<'arena>>,
    pub finally_clause: Option<TryFinallyClause<'arena>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TryCatchClause<'arena> {
    pub r#catch: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub hint: Hint<'arena>,
    pub variable: Option<DirectVariable<'arena>>,
    pub right_parenthesis: Span,
    pub block: Block<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TryFinallyClause<'arena> {
    pub r#finally: Keyword<'arena>,
    pub block: Block<'arena>,
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
