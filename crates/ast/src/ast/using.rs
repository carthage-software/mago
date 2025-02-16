use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::keyword::Keyword;
use crate::ast::statement::Statement;
use crate::ast::variable::DirectVariable;
use crate::sequence::TokenSeparatedSequence;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Using {
    pub using: Keyword,
    pub left_parenthesis: Span,
    pub items: TokenSeparatedSequence<UsingItem>,
    pub right_parenthesis: Span,
    pub statement: Box<Statement>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum UsingItem {
    Abstract(DirectVariable),
    Concrete(DirectVariable, Span, Expression),
}

impl HasSpan for UsingItem {
    fn span(&self) -> Span {
        match self {
            UsingItem::Abstract(direct_variable) => direct_variable.span(),
            UsingItem::Concrete(direct_variable, _, expression) => direct_variable.span().join(expression.span()),
        }
    }
}

impl HasSpan for Using {
    fn span(&self) -> Span {
        self.using.span().join(self.statement.span())
    }
}
