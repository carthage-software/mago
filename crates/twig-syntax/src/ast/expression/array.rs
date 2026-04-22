use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::TokenSeparatedSequence;
use crate::ast::expression::Expression;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Array<'arena> {
    pub left_bracket: Span,
    pub elements: TokenSeparatedSequence<'arena, ArrayElement<'arena>>,
    pub right_bracket: Span,
}

/// An element inside an array literal: plain values, variadic (spread)
/// elements, and missing holes used by destructuring (`[, second]`).
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum ArrayElement<'arena> {
    Value(ValueArrayElement<'arena>),
    Variadic(VariadicArrayElement<'arena>),
    Missing(MissingArrayElement),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ValueArrayElement<'arena> {
    pub value: &'arena Expression<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct VariadicArrayElement<'arena> {
    pub ellipsis: Span,
    pub value: &'arena Expression<'arena>,
}

/// A missing array element - produced for destructuring patterns like
/// `[, second]` where the first slot is skipped. The `comma` span points
/// at the comma that *follows* the empty slot.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MissingArrayElement {
    pub comma: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct HashMapEntry<'arena> {
    /// `Some(span)` only for spread entries (`...expr`).
    pub ellipsis: Option<Span>,
    /// `None` for spread entries; otherwise the key expression.
    pub key: Option<Expression<'arena>>,
    /// `Some(span)` iff the source had an explicit `key: value`.
    pub colon: Option<Span>,
    pub value: Expression<'arena>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct HashMap<'arena> {
    pub left_brace: Span,
    pub entries: TokenSeparatedSequence<'arena, HashMapEntry<'arena>>,
    pub right_brace: Span,
}

impl<'arena> ArrayElement<'arena> {
    #[inline]
    #[must_use]
    pub const fn is_variadic(&self) -> bool {
        matches!(self, ArrayElement::Variadic(_))
    }

    #[inline]
    #[must_use]
    pub const fn is_missing(&self) -> bool {
        matches!(self, ArrayElement::Missing(_))
    }

    #[inline]
    #[must_use]
    pub const fn value(&self) -> Option<&'arena Expression<'arena>> {
        match self {
            ArrayElement::Value(e) => Some(e.value),
            ArrayElement::Variadic(e) => Some(e.value),
            ArrayElement::Missing(_) => None,
        }
    }
}

impl HasSpan for Array<'_> {
    fn span(&self) -> Span {
        self.left_bracket.join(self.right_bracket)
    }
}

impl HasSpan for ArrayElement<'_> {
    fn span(&self) -> Span {
        match self {
            ArrayElement::Value(e) => e.span(),
            ArrayElement::Variadic(e) => e.span(),
            ArrayElement::Missing(e) => e.span(),
        }
    }
}

impl HasSpan for MissingArrayElement {
    fn span(&self) -> Span {
        self.comma
    }
}

impl HasSpan for ValueArrayElement<'_> {
    fn span(&self) -> Span {
        self.value.span()
    }
}

impl HasSpan for VariadicArrayElement<'_> {
    fn span(&self) -> Span {
        self.ellipsis.join(self.value.span())
    }
}

impl HasSpan for HashMapEntry<'_> {
    fn span(&self) -> Span {
        let start = self.ellipsis.or_else(|| self.key.as_ref().map(HasSpan::span)).unwrap_or_else(|| self.value.span());
        start.join(self.value.span())
    }
}

impl HasSpan for HashMap<'_> {
    fn span(&self) -> Span {
        self.left_brace.join(self.right_brace)
    }
}
