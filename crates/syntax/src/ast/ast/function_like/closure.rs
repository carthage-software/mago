use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::attribute::AttributeList;
use crate::ast::ast::block::Block;
use crate::ast::ast::function_like::parameter::FunctionLikeParameterList;
use crate::ast::ast::function_like::r#return::FunctionLikeReturnTypeHint;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::variable::DirectVariable;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Closure<'arena> {
    pub attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    pub r#static: Option<Keyword<'arena>>,
    pub function: Keyword<'arena>,
    pub ampersand: Option<Span>,
    pub parameter_list: FunctionLikeParameterList<'arena>,
    pub use_clause: Option<ClosureUseClause<'arena>>,
    pub return_type_hint: Option<FunctionLikeReturnTypeHint<'arena>>,
    pub body: Block<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ClosureUseClause<'arena> {
    pub r#use: Keyword<'arena>,
    pub left_parenthesis: Span,
    pub variables: TokenSeparatedSequence<'arena, ClosureUseClauseVariable<'arena>>,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ClosureUseClauseVariable<'arena> {
    pub ampersand: Option<Span>,
    pub variable: DirectVariable<'arena>,
}

impl HasSpan for Closure<'_> {
    fn span(&self) -> Span {
        if let Some(attribute_list) = self.attribute_lists.first() {
            return attribute_list.span().join(self.body.span());
        }

        if let Some(r#static) = &self.r#static {
            return r#static.span().join(self.body.span());
        }

        self.function.span.join(self.body.span())
    }
}

impl HasSpan for ClosureUseClause<'_> {
    fn span(&self) -> Span {
        Span::between(self.r#use.span(), self.right_parenthesis)
    }
}

impl HasSpan for ClosureUseClauseVariable<'_> {
    fn span(&self) -> Span {
        if let Some(ampersand) = self.ampersand {
            Span::between(ampersand, self.variable.span())
        } else {
            self.variable.span()
        }
    }
}
