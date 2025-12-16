use bumpalo::Bump;
use bumpalo::collections::Vec;
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

/// Represents a list of arguments in a partial function application.
///
/// Example: `(1, ?, 3, ...)` in `foo(1, ?, 3, ...)`
///
/// Reference: <https://wiki.php.net/rfc/partial_function_application_v2>
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PartialArgumentList<'arena> {
    pub left_parenthesis: Span,
    pub arguments: TokenSeparatedSequence<'arena, PartialArgument<'arena>>,
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

/// Represents an argument or placeholder in a partial function application.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum PartialArgument<'arena> {
    Positional(PositionalArgument<'arena>),
    Named(NamedArgument<'arena>),
    NamedPlaceholder(NamedPlaceholderArgument<'arena>),
    Placeholder(PlaceholderArgument),
    VariadicPlaceholder(VariadicPlaceholderArgument),
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

/// Represents a named placeholder in a partial function application.
///
/// Example: `foo: ?` in `foo(foo: ?, bar: 2)`
///
/// Reference: <https://wiki.php.net/rfc/partial_function_application_v2>
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct NamedPlaceholderArgument<'arena> {
    pub name: LocalIdentifier<'arena>,
    pub colon: Span,
    pub question_mark: Span,
}

/// Represents a placeholder in a partial function application.
///
/// Example: `?` in `foo(1, ?, 3)`
///
/// Reference: <https://wiki.php.net/rfc/partial_function_application_v2>
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PlaceholderArgument {
    pub span: Span,
}

/// Represents a variadic placeholder in a partial function application.
///
/// Example: `...` in `foo(1, 2, ...)`
///
/// Reference: <https://wiki.php.net/rfc/partial_function_application_v2>
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct VariadicPlaceholderArgument {
    pub span: Span,
}

impl<'arena> PartialArgumentList<'arena> {
    #[inline]
    #[must_use]
    pub fn is_first_class_callable(&self) -> bool {
        self.arguments.len() == 1 && matches!(self.arguments.first(), Some(PartialArgument::VariadicPlaceholder(_)))
    }

    #[inline]
    pub(crate) fn has_placeholders(&self) -> bool {
        self.arguments.iter().any(|arg| {
            matches!(
                arg,
                PartialArgument::Placeholder(_)
                    | PartialArgument::VariadicPlaceholder(_)
                    | PartialArgument::NamedPlaceholder(_)
            )
        })
    }

    #[inline]
    pub(crate) fn into_argument_list(self, arena: &'arena Bump) -> ArgumentList<'arena> {
        debug_assert!(!self.has_placeholders(), "Cannot convert PartialArgumentList with placeholders to ArgumentList");

        let mut arguments = Vec::new_in(arena);
        for arg in self.arguments.nodes {
            arguments.push(match arg {
                PartialArgument::Positional(p) => Argument::Positional(p),
                PartialArgument::Named(n) => Argument::Named(n),
                _ => unreachable!("has_placeholders should have caught this"),
            });
        }

        ArgumentList {
            left_parenthesis: self.left_parenthesis,
            arguments: TokenSeparatedSequence::new(arguments, self.arguments.tokens),
            right_parenthesis: self.right_parenthesis,
        }
    }
}

impl<'arena> Argument<'arena> {
    #[inline]
    #[must_use]
    pub const fn is_positional(&self) -> bool {
        matches!(self, Argument::Positional(_))
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
    pub const fn value(&self) -> &Expression<'arena> {
        match self {
            Argument::Positional(arg) => &arg.value,
            Argument::Named(arg) => &arg.value,
        }
    }
}

impl PartialArgument<'_> {
    #[inline]
    #[must_use]
    pub const fn is_positional(&self) -> bool {
        matches!(self, PartialArgument::Positional(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_named(&self) -> bool {
        matches!(self, PartialArgument::Named(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_named_placeholder(&self) -> bool {
        matches!(self, PartialArgument::NamedPlaceholder(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_placeholder(&self) -> bool {
        matches!(self, PartialArgument::Placeholder(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_variadic_placeholder(&self) -> bool {
        matches!(self, PartialArgument::VariadicPlaceholder(_))
    }
}

impl HasSpan for ArgumentList<'_> {
    fn span(&self) -> Span {
        Span::between(self.left_parenthesis, self.right_parenthesis)
    }
}

impl HasSpan for PartialArgumentList<'_> {
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

impl HasSpan for PartialArgument<'_> {
    fn span(&self) -> Span {
        match self {
            PartialArgument::Positional(argument) => argument.span(),
            PartialArgument::Named(argument) => argument.span(),
            PartialArgument::NamedPlaceholder(argument) => argument.span(),
            PartialArgument::Placeholder(placeholder) => placeholder.span(),
            PartialArgument::VariadicPlaceholder(placeholder) => placeholder.span(),
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

impl HasSpan for NamedPlaceholderArgument<'_> {
    fn span(&self) -> Span {
        Span::between(self.name.span(), self.question_mark)
    }
}

impl HasSpan for PlaceholderArgument {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for VariadicPlaceholderArgument {
    fn span(&self) -> Span {
        self.span
    }
}
