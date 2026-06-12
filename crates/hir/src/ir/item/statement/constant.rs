#[cfg(feature = "serde")]
use serde::Serialize;

use mago_php_version::PHPVersionRange;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::expression::Expression;
use crate::ir::identifier::Identifier;
use crate::ir::item::Item;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::attribute::Attribute;

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
