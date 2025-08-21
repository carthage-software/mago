use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;
use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::sequence::TokenSeparatedSequence;

/// Represents a list of arguments.
///
/// Example: `($bar, 42)` in `foo($bar, 42)`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ArgumentList<'arena> {
    pub left_parenthesis: Span,
    pub arguments: TokenSeparatedSequence<'arena, Argument<'arena>>,
    pub right_parenthesis: Span,
}

/// Represents an argument.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum Argument<'arena> {
    Positional(PositionalArgument<'arena>),
    Named(NamedArgument<'arena>),
}

/// Represents a positional argument.
///
/// Example: `$foo` in `foo($foo)`, `...$bar` in `foo(...$bar)`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PositionalArgument<'arena> {
    pub ellipsis: Option<Span>,
    pub value: Expression<'arena>,
}

/// Represents a named argument.
///
/// Example: `foo: 42` in `foo(foo: 42)`, `bar: ...$bar` in `foo(bar: ...$bar)`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct NamedArgument<'arena> {
    pub name: LocalIdentifier<'arena>,
    pub colon: Span,
    pub value: Expression<'arena>,
}

impl<'arena> Argument<'arena> {
    #[inline]
    pub const fn is_positional(&self) -> bool {
        matches!(self, Argument::Positional(_))
    }

    #[inline]
    pub const fn is_unpacked(&self) -> bool {
        match self {
            Argument::Positional(arg) => arg.ellipsis.is_some(),
            Argument::Named(_) => false,
        }
    }

    #[inline]
    pub const fn value(&self) -> &Expression<'arena> {
        match self {
            Argument::Positional(arg) => &arg.value,
            Argument::Named(arg) => &arg.value,
        }
    }
}

impl HasSpan for ArgumentList<'_> {
    fn span(&self) -> Span {
        Span::between(self.left_parenthesis, self.right_parenthesis)
    }
}

impl HasSpan for Argument<'_> {
    fn span(&self) -> Span {
        match self {
            Argument::Positional(argument) => argument.span(),
            Argument::Named(argument) => argument.span(),
        }
    }
}

impl HasSpan for PositionalArgument<'_> {
    fn span(&self) -> Span {
        if let Some(ellipsis) = &self.ellipsis {
            Span::between(*ellipsis, self.value.span())
        } else {
            self.value.span()
        }
    }
}

impl HasSpan for NamedArgument<'_> {
    fn span(&self) -> Span {
        Span::between(self.name.span(), self.value.span())
    }
}
