#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;
use mago_flags::U8Flags;
use mago_php_version::PHPVersionRange;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::delimited::Delimited;
use crate::ir::expression::Expression;
use crate::ir::item::Item;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::attribute::Attribute;
use crate::ir::item::modifier::Modifier;
use crate::ir::item::parameter::Parameter;
use crate::ir::name::Name;
use crate::ir::statement::Block;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum HookFlag {
    ReturnsByReference = 1 << 0,
    IsVariadic = 1 << 1,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Hook<'arena, I, S, E> {
    pub span: Span,
    pub annotation: Option<&'arena ItemAnnotation<'arena, I, S, E>>,
    pub attributes: &'arena [Attribute<'arena, I, S, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub flags: U8Flags<HookFlag>,
    pub modifiers: &'arena [Modifier],
    pub name: Name<'arena>,
    pub parameters: Option<Delimited<'arena, Parameter<'arena, I, S, E>>>,
    pub body: Option<HookBody<'arena, I, S, E>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct HookBody<'arena, I, S, E> {
    pub span: Span,
    pub kind: HookBodyKind<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum HookBodyKind<'arena, I, S, E> {
    Expression(&'arena Expression<'arena, I, S, E>),
    Block(&'arena Block<'arena, I, S, E>),
}

impl CopyInto for HookFlag {
    type Output<'arena> = HookFlag;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl<I, S, E> CopyInto for Hook<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Hook<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Hook {
            span: self.span,
            annotation: self.annotation.map(|node| copy_ref_into(node, arena)),
            attributes: copy_slice_into(self.attributes, arena),
            version_constraint: arena.alloc_slice_copy(self.version_constraint),
            flags: self.flags,
            modifiers: arena.alloc_slice_copy(self.modifiers),
            name: self.name.copy_into(arena),
            parameters: self.parameters.as_ref().map(|node| node.copy_into(arena)),
            body: self.body.as_ref().map(|node| node.copy_into(arena)),
        }
    }
}

impl<I, S, E> CopyInto for HookBody<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = HookBody<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        HookBody { span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl<I, S, E> CopyInto for HookBodyKind<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = HookBodyKind<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match self {
            HookBodyKind::Expression(expression) => HookBodyKind::Expression(copy_ref_into(*expression, arena)),
            HookBodyKind::Block(block) => HookBodyKind::Block(copy_ref_into(*block, arena)),
        }
    }
}

impl<I, S, E> Hook<'_, I, S, E> {
    #[must_use]
    pub fn has_annotation(&self) -> bool {
        self.annotation.is_some()
    }

    #[must_use]
    pub const fn returns_by_reference(&self) -> bool {
        self.flags.contains_bits(HookFlag::ReturnsByReference as u8)
    }

    #[must_use]
    pub const fn is_variadic(&self) -> bool {
        self.flags.contains_bits(HookFlag::IsVariadic as u8)
    }
}

impl From<HookFlag> for u8 {
    fn from(flag: HookFlag) -> Self {
        flag as u8
    }
}

impl<I, S, E> HasSpan for Hook<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for HookBody<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena, I, S, E> Item<'arena, I, S, E> for Hook<'arena, I, S, E> {
    fn attributes(&self) -> &'arena [Attribute<'arena, I, S, E>] {
        self.attributes
    }

    fn annotation(&self) -> Option<&'arena ItemAnnotation<'arena, I, S, E>> {
        self.annotation
    }

    fn version_constraint(&self) -> &'arena [PHPVersionRange] {
        self.version_constraint
    }
}
