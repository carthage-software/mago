use serde::Deserialize;
use serde::Serialize;

use fennec_span::HasSpan;
use fennec_span::Span;
use strum::Display;

use crate::ast::expression::Expression;
use crate::ast::identifier::LocalIdentifier;
use crate::sequence::TokenSeparatedSequence;

/// Represents a list of arguments.
///
/// Example: `($bar, 42)` in `foo($bar, 42)`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ArgumentList {
    pub left_parenthesis: Span,
    pub arguments: TokenSeparatedSequence<Argument>,
    pub right_parenthesis: Span,
}

/// Represents an argument.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum Argument {
    Positional(PositionalArgument),
    Named(NamedArgument),
}

/// Represents a positional argument.
///
/// Example: `$foo` in `foo($foo)`, `...$bar` in `foo(...$bar)`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PositionalArgument {
    pub ellipsis: Option<Span>,
    pub value: Expression,
}

/// Represents a named argument.
///
/// Example: `foo: 42` in `foo(foo: 42)`, `bar: ...$bar` in `foo(bar: ...$bar)`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct NamedArgument {
    pub name: LocalIdentifier,
    pub colon: Span,
    pub ellipsis: Option<Span>,
    pub value: Expression,
}

impl Argument {
    pub fn value(&self) -> &Expression {
        match self {
            Argument::Positional(arg) => &arg.value,
            Argument::Named(arg) => &arg.value,
        }
    }
}

impl HasSpan for ArgumentList {
    fn span(&self) -> Span {
        Span::between(self.left_parenthesis, self.right_parenthesis)
    }
}

impl HasSpan for Argument {
    fn span(&self) -> Span {
        match self {
            Argument::Positional(argument) => argument.span(),
            Argument::Named(argument) => argument.span(),
        }
    }
}

impl HasSpan for PositionalArgument {
    fn span(&self) -> Span {
        if let Some(ellipsis) = &self.ellipsis {
            Span::between(*ellipsis, self.value.span())
        } else {
            self.value.span()
        }
    }
}

impl HasSpan for NamedArgument {
    fn span(&self) -> Span {
        Span::between(self.name.span(), self.value.span())
    }
}
