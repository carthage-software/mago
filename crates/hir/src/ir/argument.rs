#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::expression::Expression;
use crate::ir::name::Name;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;

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

impl<I, S, E> CopyInto for Argument<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Argument<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            Argument::Value(expression) => Argument::Value(copy_ref_into(*expression, arena)),
            Argument::Variadic(expression) => Argument::Variadic(copy_ref_into(*expression, arena)),
            Argument::Named(name, expression) => {
                Argument::Named(name.copy_into(arena), copy_ref_into(*expression, arena))
            }
        }
    }
}

impl<I, S, E> CopyInto for PartialArgument<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = PartialArgument<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            PartialArgument::Value(expression) => PartialArgument::Value(copy_ref_into(*expression, arena)),
            PartialArgument::Variadic(expression) => PartialArgument::Variadic(copy_ref_into(*expression, arena)),
            PartialArgument::Named(name, expression) => {
                PartialArgument::Named(name.copy_into(arena), copy_ref_into(*expression, arena))
            }
            PartialArgument::Placeholder(span) => PartialArgument::Placeholder(*span),
            PartialArgument::NamedPlaceholder(name) => PartialArgument::NamedPlaceholder(name.copy_into(arena)),
            PartialArgument::VariadicPlaceholder(span) => PartialArgument::VariadicPlaceholder(*span),
        }
    }
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
