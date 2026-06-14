#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_php_version::PHPVersionRange;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::expression::Expression;
use crate::ir::identifier::Identifier;
use crate::ir::item::Item;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::attribute::Attribute;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Constant<'arena, I, S, E> {
    pub span: Span,
    pub annotation: Option<&'arena ItemAnnotation<'arena, I, S, E>>,
    pub attributes: &'arena [Attribute<'arena, I, S, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub items: &'arena [ConstantItem<'arena, I, S, E>],
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ConstantItem<'arena, I, S, E> {
    pub span: Span,
    pub name: Identifier<'arena>,
    pub value: &'arena Expression<'arena, I, S, E>,
}

impl<I, S, E> CopyInto for Constant<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Constant<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Constant {
            span: self.span,
            annotation: self.annotation.map(|node| copy_ref_into(node, arena)),
            attributes: copy_slice_into(self.attributes, arena),
            version_constraint: arena.alloc_slice_copy(self.version_constraint),
            items: copy_slice_into(self.items, arena),
        }
    }
}

impl<I, S, E> CopyInto for ConstantItem<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = ConstantItem<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ConstantItem { span: self.span, name: self.name.copy_into(arena), value: copy_ref_into(self.value, arena) }
    }
}

impl<'arena, I, S, E> Item<'arena, I, S, E> for Constant<'arena, I, S, E> {
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

impl<I, S, E> HasSpan for ConstantItem<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for Constant<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}
