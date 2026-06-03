use mago_span::HasSpan;
use mago_span::Span;
use serde::Serialize;

use crate::ir::expression::Expression;
use crate::ir::name::Name;
use crate::ir::variable::DirectVariable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum MemberSelector<'arena, S, D, E> {
    Name(Name<'arena>),
    Variable(DirectVariable<'arena>),
    Expression(&'arena Expression<'arena, S, D, E>),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum ConstantSelector<'arena, S, D, E> {
    Name(Name<'arena>),
    Expression(&'arena Expression<'arena, S, D, E>),
}

impl<S, D, E> HasSpan for MemberSelector<'_, S, D, E> {
    fn span(&self) -> Span {
        match self {
            MemberSelector::Name(name) => name.span(),
            MemberSelector::Variable(variable) => variable.span(),
            MemberSelector::Expression(expression) => expression.span(),
        }
    }
}

impl<S, D, E> HasSpan for ConstantSelector<'_, S, D, E> {
    fn span(&self) -> Span {
        match self {
            ConstantSelector::Name(name) => name.span(),
            ConstantSelector::Expression(expression) => expression.span(),
        }
    }
}
