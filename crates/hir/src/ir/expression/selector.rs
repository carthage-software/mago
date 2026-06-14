use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;
#[cfg(feature = "serde")]
use serde::Serialize;

use crate::ir::expression::Expression;
use crate::ir::name::Name;
use crate::ir::variable::DirectVariable;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;

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

impl<I, S, E> CopyInto for MemberSelector<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = MemberSelector<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        MemberSelector { span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for MemberSelectorKind<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = MemberSelectorKind<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            MemberSelectorKind::Missing => MemberSelectorKind::Missing,
            MemberSelectorKind::Name(name) => MemberSelectorKind::Name(name.copy_into(arena)),
            MemberSelectorKind::Variable(variable) => MemberSelectorKind::Variable(variable.copy_into(arena)),
            MemberSelectorKind::Expression(expression) => {
                MemberSelectorKind::Expression(copy_ref_into(*expression, arena))
            }
        }
    }
}

impl<I, S, E> CopyInto for ConstantSelector<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = ConstantSelector<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ConstantSelector { span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for ConstantSelectorKind<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = ConstantSelectorKind<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            ConstantSelectorKind::Missing => ConstantSelectorKind::Missing,
            ConstantSelectorKind::Name(name) => ConstantSelectorKind::Name(name.copy_into(arena)),
            ConstantSelectorKind::Expression(expression) => {
                ConstantSelectorKind::Expression(copy_ref_into(*expression, arena))
            }
        }
    }
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
