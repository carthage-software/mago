#[cfg(feature = "serde")]
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::expression::Expression;
use crate::ir::name::Name;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum Argument<'arena, I, S, E> {
    Value(&'arena Expression<'arena, I, S, E>),
    Variadic(&'arena Expression<'arena, I, S, E>),
    Named(Name<'arena>, &'arena Expression<'arena, I, S, E>),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum PartialArgument<'arena, I, S, E> {
    Value(&'arena Expression<'arena, I, S, E>),
    Variadic(&'arena Expression<'arena, I, S, E>),
    Named(Name<'arena>, &'arena Expression<'arena, I, S, E>),
    Placeholder(Span),
    NamedPlaceholder(Name<'arena>),
    VariadicPlaceholder(Span),
}

impl<I, S, E> HasSpan for Argument<'_, I, S, E> {
    fn span(&self) -> Span {
        match self {
            Argument::Value(expression) | Argument::Variadic(expression) => expression.span(),
            Argument::Named(name, expression) => name.span().join(expression.span()),
        }
    }
}

impl<I, S, E> HasSpan for PartialArgument<'_, I, S, E> {
    fn span(&self) -> Span {
        match self {
            PartialArgument::Value(expression) | PartialArgument::Variadic(expression) => expression.span(),
            PartialArgument::Named(name, expression) => name.span().join(expression.span()),
            PartialArgument::NamedPlaceholder(name) => name.span(),
            PartialArgument::Placeholder(span) | PartialArgument::VariadicPlaceholder(span) => *span,
        }
    }
}
