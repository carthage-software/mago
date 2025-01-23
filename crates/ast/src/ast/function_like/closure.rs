use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::attribute::AttributeList;
use crate::ast::block::Block;
use crate::ast::function_like::parameter::FunctionLikeParameterList;
use crate::ast::function_like::r#return::FunctionLikeReturnTypeHint;
use crate::ast::keyword::Keyword;
use crate::ast::variable::DirectVariable;
use crate::sequence::Sequence;
use crate::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Closure {
    pub attribute_lists: Sequence<AttributeList>,
    pub r#static: Option<Keyword>,
    pub function: Keyword,
    pub ampersand: Option<Span>,
    pub parameter_list: FunctionLikeParameterList,
    pub use_clause: Option<ClosureUseClause>,
    pub return_type_hint: Option<FunctionLikeReturnTypeHint>,
    pub body: Block,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ClosureUseClause {
    pub r#use: Keyword,
    pub left_parenthesis: Span,
    pub variables: TokenSeparatedSequence<ClosureUseClauseVariable>,
    pub right_parenthesis: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ClosureUseClauseVariable {
    pub ampersand: Option<Span>,
    pub variable: DirectVariable,
}

impl HasSpan for Closure {
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

impl HasSpan for ClosureUseClause {
    fn span(&self) -> Span {
        Span::between(self.r#use.span(), self.right_parenthesis)
    }
}

impl HasSpan for ClosureUseClauseVariable {
    fn span(&self) -> Span {
        if let Some(ampersand) = self.ampersand {
            Span::between(ampersand, self.variable.span())
        } else {
            self.variable.span()
        }
    }
}
