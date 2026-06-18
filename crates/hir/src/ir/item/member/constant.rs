#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;
use mago_php_version::PHPVersionRange;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::expression::Expression;
use crate::ir::item::Item;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::attribute::Attribute;
use crate::ir::item::modifier::Modifier;
use crate::ir::name::Name;
use crate::ir::r#type::Type;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ClassLikeConstant<'arena, I, S, E> {
    pub span: Span,
    pub annotation: Option<&'arena ItemAnnotation<'arena, I, S, E>>,
    pub attributes: &'arena [Attribute<'arena, I, S, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub modifiers: &'arena [Modifier],
    pub r#type: Option<&'arena Type<'arena>>,
    pub name: Name<'arena>,
    pub value: &'arena Expression<'arena, I, S, E>,
}

impl<I, S, E> CopyInto for ClassLikeConstant<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = ClassLikeConstant<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ClassLikeConstant {
            span: self.span,
            annotation: self.annotation.map(|node| copy_ref_into(node, arena)),
            attributes: copy_slice_into(self.attributes, arena),
            version_constraint: arena.alloc_slice_copy(self.version_constraint),
            modifiers: arena.alloc_slice_copy(self.modifiers),
            r#type: self.r#type.map(|node| copy_ref_into(node, arena)),
            name: self.name.copy_into(arena),
            value: copy_ref_into(self.value, arena),
        }
    }
}

impl<I, S, E> ClassLikeConstant<'_, I, S, E> {
    #[must_use]
    pub fn has_annotation(&self) -> bool {
        self.annotation.is_some()
    }
}

impl<I, S, E> HasSpan for ClassLikeConstant<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena, I, S, E> Item<'arena, I, S, E> for ClassLikeConstant<'arena, I, S, E> {
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
