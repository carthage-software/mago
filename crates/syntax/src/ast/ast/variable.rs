use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::expression::Expression;

/// Represents a variable.
///
/// # Examples
///
/// ```php
/// $foo
/// ${foo}
/// $$foo
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum Variable<'arena> {
    Direct(DirectVariable<'arena>),
    Indirect(IndirectVariable<'arena>),
    Nested(NestedVariable<'arena>),
}

/// Represents a direct variable.
///
/// A direct variable is a variable that is directly referenced by its name.
///
/// # Examples
///
/// ```php
/// $foo
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct DirectVariable<'arena> {
    pub span: Span,
    pub name: &'arena str,
}

/// Represents an indirect variable.
///
/// An indirect variable is a variable whose name is determined by evaluating an expression at runtime.
///
/// The expression is enclosed in curly braces `{}` following a dollar sign `$`.
///
/// # Examples
///
/// ```php
/// ${foo}
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct IndirectVariable<'arena> {
    pub dollar_left_brace: Span,
    pub expression: &'arena Expression<'arena>,
    pub right_brace: Span,
}

/// Represents a nested variable.
///
/// A nested variable is a variable that is nested inside another variable, commonly known as a variable variable.
///
/// # Examples
///
/// ```php
/// $$foo
/// $${foo}
/// $$$foo
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct NestedVariable<'arena> {
    pub dollar: Span,
    pub variable: &'arena Variable<'arena>,
}

impl HasSpan for Variable<'_> {
    fn span(&self) -> Span {
        match self {
            Variable::Direct(node) => node.span(),
            Variable::Indirect(node) => node.span(),
            Variable::Nested(node) => node.span(),
        }
    }
}

impl HasSpan for DirectVariable<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for IndirectVariable<'_> {
    fn span(&self) -> Span {
        Span::between(self.dollar_left_brace, self.right_brace)
    }
}

impl HasSpan for NestedVariable<'_> {
    fn span(&self) -> Span {
        Span::between(self.dollar, self.variable.span())
    }
}
