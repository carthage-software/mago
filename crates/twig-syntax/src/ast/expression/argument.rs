use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::Identifier;
use crate::ast::TokenSeparatedSequence;
use crate::ast::expression::Expression;

/// A parenthesised argument list attached to a call, method call, filter,
/// or test.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ArgumentList<'arena> {
    pub left_parenthesis: Span,
    pub arguments: TokenSeparatedSequence<'arena, Argument<'arena>>,
    pub right_parenthesis: Span,
}

/// A single argument in a call or filter/test argument list.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum Argument<'arena> {
    Positional(PositionalArgument<'arena>),
    Named(NamedArgument<'arena>),
}

/// A positional argument. `...x` (spread) is represented as a positional
/// argument with `ellipsis = Some(span)`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PositionalArgument<'arena> {
    pub ellipsis: Option<Span>,
    pub value: &'arena Expression<'arena>,
}

/// The separator between a named argument's name and its value. Twig
/// accepts both `name = value` and `name: value`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum NamedArgumentSeparator {
    Equal(Span),
    Colon(Span),
}

/// A named argument: `name = value` or `name: value`.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct NamedArgument<'arena> {
    pub name: Identifier<'arena>,
    pub separator: NamedArgumentSeparator,
    pub value: &'arena Expression<'arena>,
}

impl<'arena> Argument<'arena> {
    #[inline]
    #[must_use]
    pub const fn is_positional(&self) -> bool {
        matches!(self, Argument::Positional(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_named(&self) -> bool {
        matches!(self, Argument::Named(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_unpacked(&self) -> bool {
        match self {
            Argument::Positional(arg) => arg.ellipsis.is_some(),
            Argument::Named(_) => false,
        }
    }

    #[inline]
    #[must_use]
    pub const fn value(&self) -> &'arena Expression<'arena> {
        match self {
            Argument::Positional(arg) => arg.value,
            Argument::Named(arg) => arg.value,
        }
    }
}

impl HasSpan for ArgumentList<'_> {
    fn span(&self) -> Span {
        self.left_parenthesis.join(self.right_parenthesis)
    }
}

impl HasSpan for Argument<'_> {
    fn span(&self) -> Span {
        match self {
            Argument::Positional(a) => a.span(),
            Argument::Named(a) => a.span(),
        }
    }
}

impl HasSpan for PositionalArgument<'_> {
    fn span(&self) -> Span {
        if let Some(ellipsis) = self.ellipsis { ellipsis.join(self.value.span()) } else { self.value.span() }
    }
}

impl HasSpan for NamedArgumentSeparator {
    fn span(&self) -> Span {
        match self {
            NamedArgumentSeparator::Equal(s) | NamedArgumentSeparator::Colon(s) => *s,
        }
    }
}

impl HasSpan for NamedArgument<'_> {
    fn span(&self) -> Span {
        self.name.span.join(self.value.span())
    }
}
