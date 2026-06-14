#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_flags::U16Flags;
use mago_php_version::PHPVersionRange;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::delimited::Delimited;
use crate::ir::identifier::Identifier;
use crate::ir::item::Item;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::attribute::Attribute;
use crate::ir::item::attribute::AttributeTarget;
use crate::ir::item::inheritance::Extends;
use crate::ir::item::inheritance::Implements;
use crate::ir::item::member::MemberItem;
use crate::ir::item::modifier::Modifier;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Class<'arena, I, S, E> {
    pub span: Span,
    pub annotation: Option<&'arena ItemAnnotation<'arena, I, S, E>>,
    pub attributes: &'arena [Attribute<'arena, I, S, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub attribute_target: Option<U16Flags<AttributeTarget>>,
    pub modifiers: &'arena [Modifier],
    pub name: Identifier<'arena>,
    pub extends: Option<&'arena Extends<'arena>>,
    pub implements: Option<&'arena Implements<'arena>>,
    pub members: Delimited<'arena, MemberItem<'arena, I, S, E>>,
}

impl<I, S, E> CopyInto for Class<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Class<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Class {
            span: self.span,
            annotation: self.annotation.map(|node| copy_ref_into(node, arena)),
            attributes: copy_slice_into(self.attributes, arena),
            version_constraint: arena.alloc_slice_copy(self.version_constraint),
            attribute_target: self.attribute_target,
            modifiers: arena.alloc_slice_copy(self.modifiers),
            name: self.name.copy_into(arena),
            extends: self.extends.map(|node| copy_ref_into(node, arena)),
            implements: self.implements.map(|node| copy_ref_into(node, arena)),
            members: self.members.copy_into(arena),
        }
    }
}

impl<I, S, E> HasSpan for Class<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena, I, S, E> Item<'arena, I, S, E> for Class<'arena, I, S, E> {
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
