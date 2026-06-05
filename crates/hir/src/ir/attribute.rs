use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::argument::Argument;
use crate::ir::identifier::Identifier;

/// Represents a single attribute.
///
/// Example: `Foo` in `#[Foo]`, `Bar(1)` in `#[Bar(1)]`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Attribute<'arena, S, D, E> {
    pub span: Span,
    pub class: Identifier<'arena>,
    pub arguments: &'arena [Argument<'arena, S, D, E>],
}

impl<S, D, E> HasSpan for Attribute<'_, S, D, E> {
    fn span(&self) -> Span {
        self.span
    }
}

/// The declarations a `#[Attribute]` may be applied to, mirroring PHP's
/// `Attribute::TARGET_*` and `Attribute::IS_REPEATABLE` flags.
///
/// Lowered from the argument of an `#[Attribute(...)]` declaration so that
/// consumers can model where an attribute class is allowed without re-parsing
/// the attribute expression.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub enum AttributeTarget {
    Class = 1 << 0,
    Function = 1 << 1,
    Method = 1 << 2,
    Property = 1 << 3,
    ClassConstant = 1 << 4,
    Parameter = 1 << 5,
    Constant = 1 << 6,
    Repeatable = 1 << 7,
}

impl From<AttributeTarget> for u32 {
    fn from(value: AttributeTarget) -> Self {
        value as u32
    }
}
