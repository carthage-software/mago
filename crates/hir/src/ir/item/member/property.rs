#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_php_version::PHPVersionRange;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::delimited::Delimited;
use crate::ir::expression::Expression;
use crate::ir::item::Item;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::attribute::Attribute;
use crate::ir::item::member::hook::Hook;
use crate::ir::item::modifier::Modifier;
use crate::ir::r#type::Type;
use crate::ir::variable::DirectVariable;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Property<'arena, I, S, E> {
    pub span: Span,
    pub annotation: Option<&'arena ItemAnnotation<'arena, I, S, E>>,
    pub attributes: &'arena [Attribute<'arena, I, S, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub modifiers: &'arena [Modifier],
    pub r#type: Option<&'arena Type<'arena>>,
    pub items: &'arena [PropertyItem<'arena, I, S, E>],
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct HookedProperty<'arena, I, S, E> {
    pub span: Span,
    pub annotation: Option<&'arena ItemAnnotation<'arena, I, S, E>>,
    pub attributes: &'arena [Attribute<'arena, I, S, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub modifiers: &'arena [Modifier],
    pub r#type: Option<&'arena Type<'arena>>,
    pub item: PropertyItem<'arena, I, S, E>,
    pub hooks: Delimited<'arena, Hook<'arena, I, S, E>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct PropertyItem<'arena, I, S, E> {
    pub span: Span,
    pub variable: DirectVariable<'arena>,
    pub default_value: Option<&'arena Expression<'arena, I, S, E>>,
}

impl<I, S, E> CopyInto for Property<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Property<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Property {
            span: self.span,
            annotation: self.annotation.map(|node| copy_ref_into(node, arena)),
            attributes: copy_slice_into(self.attributes, arena),
            version_constraint: arena.alloc_slice_copy(self.version_constraint),
            modifiers: arena.alloc_slice_copy(self.modifiers),
            r#type: self.r#type.map(|node| copy_ref_into(node, arena)),
            items: copy_slice_into(self.items, arena),
        }
    }
}

impl<I, S, E> CopyInto for HookedProperty<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = HookedProperty<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        HookedProperty {
            span: self.span,
            annotation: self.annotation.map(|node| copy_ref_into(node, arena)),
            attributes: copy_slice_into(self.attributes, arena),
            version_constraint: arena.alloc_slice_copy(self.version_constraint),
            modifiers: arena.alloc_slice_copy(self.modifiers),
            r#type: self.r#type.map(|node| copy_ref_into(node, arena)),
            item: self.item.copy_into(arena),
            hooks: self.hooks.copy_into(arena),
        }
    }
}

impl<I, S, E> CopyInto for PropertyItem<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = PropertyItem<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        PropertyItem {
            span: self.span,
            variable: self.variable.copy_into(arena),
            default_value: self.default_value.map(|node| copy_ref_into(node, arena)),
        }
    }
}

impl<I, S, E> Property<'_, I, S, E> {
    #[must_use]
    pub fn has_annotation(&self) -> bool {
        self.annotation.is_some()
    }
}

impl<I, S, E> HasSpan for Property<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for HookedProperty<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<I, S, E> HasSpan for PropertyItem<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena, I, S, E> Item<'arena, I, S, E> for Property<'arena, I, S, E> {
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

impl<'arena, I, S, E> Item<'arena, I, S, E> for HookedProperty<'arena, I, S, E> {
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
