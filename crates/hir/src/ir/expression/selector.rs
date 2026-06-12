use mago_span::HasSpan;
use mago_span::Span;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::ir::expression::Expression;
use crate::ir::name::Name;
use crate::ir::variable::DirectVariable;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct MemberSelector<'arena, I, S, E> {
    pub span: Span,
    pub kind: MemberSelectorKind<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum MemberSelectorKind<'arena, I, S, E> {
    Missing,
    Name(Name<'arena>),
    Variable(DirectVariable<'arena>),
    Expression(&'arena Expression<'arena, I, S, E>),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ConstantSelector<'arena, I, S, E> {
    pub span: Span,
    pub kind: ConstantSelectorKind<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum ConstantSelectorKind<'arena, I, S, E> {
    Missing,
    Name(Name<'arena>),
    Expression(&'arena Expression<'arena, I, S, E>),
}

impl<I, S, E> HasSpan for MemberSelector<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for ConstantSelector<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}
