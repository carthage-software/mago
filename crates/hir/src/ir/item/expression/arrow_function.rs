#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::delimited::Delimited;
use crate::ir::expression::Expression;
use crate::ir::item::Item;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::attribute::Attribute;
use crate::ir::item::parameter::Parameter;
use crate::ir::r#type::Type;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ArrowFunctionFlag {
    Static = 1 << 0,
    ReturnsByReference = 1 << 1,
    AssertionsInferred = 1 << 2,
    Yields = 1 << 3,
    Throws = 1 << 4,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ArrowFunction<'arena, I, S, E> {
    pub span: Span,
    pub annotation: Option<&'arena ItemAnnotation<'arena, I, S, E>>,
    pub attributes: &'arena [Attribute<'arena, I, S, E>],
    pub flags: U8Flags<ArrowFunctionFlag>,
    pub parameters: Delimited<'arena, Parameter<'arena, I, S, E>>,
    pub return_type: Option<&'arena Type<'arena>>,
    pub expression: &'arena Expression<'arena, I, S, E>,
}

impl<I, S, E> ArrowFunction<'_, I, S, E> {
    #[must_use]
    pub fn has_annotation(&self) -> bool {
        self.annotation.is_some()
    }

    #[must_use]
    pub const fn is_static(&self) -> bool {
        self.flags.contains_bits(ArrowFunctionFlag::Static as u8)
    }

    #[must_use]
    pub const fn returns_by_reference(&self) -> bool {
        self.flags.contains_bits(ArrowFunctionFlag::ReturnsByReference as u8)
    }

    #[must_use]
    pub const fn assertions_inferred(&self) -> bool {
        self.flags.contains_bits(ArrowFunctionFlag::AssertionsInferred as u8)
    }
}

impl From<ArrowFunctionFlag> for u8 {
    fn from(flag: ArrowFunctionFlag) -> Self {
        flag as u8
    }
}

impl<I, S, E> HasSpan for ArrowFunction<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena, I, S, E> Item<'arena, I, S, E> for ArrowFunction<'arena, I, S, E> {
    fn attributes(&self) -> &'arena [Attribute<'arena, I, S, E>] {
        self.attributes
    }

    fn annotation(&self) -> Option<&'arena ItemAnnotation<'arena, I, S, E>> {
        self.annotation
    }
}
