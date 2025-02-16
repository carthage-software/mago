use bumpalo::boxed::Box;
use serde::Serialize;
use strum::Display;

use mago_interner::StringIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::expression::Expression;

/// Represents a variable.
///
/// # Examples
///
/// ```php
/// $foo
/// ${foo}
/// $$foo
/// ```
#[derive(Debug, Hash, Serialize, Display)]
#[serde(tag = "type", content = "value")]
#[repr(C, u8)]
pub enum Variable<'a> {
    Direct(DirectVariable),
    Indirect(IndirectVariable<'a>),
    Nested(NestedVariable<'a>),
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct DirectVariable {
    pub span: Span,
    pub name: StringIdentifier,
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct IndirectVariable<'a> {
    pub dollar_left_brace: Span,
    pub expression: Box<'a, Expression<'a>>,
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct NestedVariable<'a> {
    pub dollar: Span,
    pub variable: Box<'a, Variable<'a>>,
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

impl HasSpan for DirectVariable {
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
