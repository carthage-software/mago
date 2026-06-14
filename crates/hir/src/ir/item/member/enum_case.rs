#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_php_version::PHPVersionRange;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::expression::Expression;
use crate::ir::item::Item;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::attribute::Attribute;
use crate::ir::name::Name;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct EnumCase<'arena, I, S, E> {
    pub span: Span,
    pub annotation: Option<&'arena ItemAnnotation<'arena, I, S, E>>,
    pub attributes: &'arena [Attribute<'arena, I, S, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub name: Name<'arena>,
    pub value: Option<&'arena Expression<'arena, I, S, E>>,
}

impl<I, S, E> CopyInto for EnumCase<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = EnumCase<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        EnumCase {
            span: self.span,
            annotation: self.annotation.map(|node| copy_ref_into(node, arena)),
            attributes: copy_slice_into(self.attributes, arena),
            version_constraint: arena.alloc_slice_copy(self.version_constraint),
            name: self.name.copy_into(arena),
            value: self.value.map(|node| copy_ref_into(node, arena)),
        }
    }
}

impl<I, S, E> EnumCase<'_, I, S, E> {
    #[must_use]
    pub fn has_annotation(&self) -> bool {
        self.annotation.is_some()
    }
}

impl<I, S, E> HasSpan for EnumCase<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena, I, S, E> Item<'arena, I, S, E> for EnumCase<'arena, I, S, E> {
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
