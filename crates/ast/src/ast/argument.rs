use bumpalo::boxed::Box;
use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;
use crate::ast::identifier::LocalIdentifier;
use crate::sequence::TokenSeparatedSequence;

/// Represents a list of arguments.
///
/// Example: `($bar, 42)` in `foo($bar, 42)`
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ArgumentList<'a> {
    pub left_parenthesis: Span,
    pub arguments: TokenSeparatedSequence<'a, Argument<'a>>,
    pub right_parenthesis: Span,
}

/// Represents an argument.
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum Argument<'a> {
    Positional(PositionalArgument<'a>),
    Named(NamedArgument<'a>),
}

/// Represents a positional argument.
///
/// Example: `$foo` in `foo($foo)`, `...$bar` in `foo(...$bar)`
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct PositionalArgument<'a> {
    pub ellipsis: Option<Span>,
    pub value: Box<'a, Expression<'a>>,
}

/// Represents a named argument.
///
/// Example: `foo: 42` in `foo(foo: 42)`, `bar: ...$bar` in `foo(bar: ...$bar)`
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct NamedArgument<'a> {
    pub name: LocalIdentifier,
    pub colon: Span,
    pub ellipsis: Option<Span>,
    pub value: Box<'a, Expression<'a>>,
}

impl<'a> Argument<'a> {
    pub fn value(&self) -> &Expression<'a> {
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
