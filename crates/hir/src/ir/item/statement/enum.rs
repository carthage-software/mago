#[cfg(feature = "serde")]
use serde::Serialize;

use mago_php_version::PHPVersionRange;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::delimited::Delimited;
use crate::ir::identifier::Identifier;
use crate::ir::item::Item;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::attribute::Attribute;
use crate::ir::item::inheritance::Implements;
use crate::ir::item::member::MemberItem;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Enum<'arena, I, S, E> {
    pub span: Span,
    pub annotation: Option<&'arena ItemAnnotation<'arena, I, S, E>>,
    pub attributes: &'arena [Attribute<'arena, I, S, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub name: Identifier<'arena>,
    pub backing_type: Option<EnumBackingType>,
    pub implements: Option<&'arena Implements<'arena>>,
    pub members: Delimited<'arena, MemberItem<'arena, I, S, E>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct EnumBackingType {
    pub span: Span,
    pub kind: EnumBackingTypeKind,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum EnumBackingTypeKind {
    Int,
    String,
}

impl<'arena, I, S, E> Item<'arena, I, S, E> for Enum<'arena, I, S, E> {
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

impl<I, S, E> HasSpan for Enum<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for EnumBackingType {
    fn span(&self) -> Span {
        self.span
    }
}
