#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U8Flags;
use mago_php_version::PHPVersionRange;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::delimited::Delimited;
use crate::ir::item::Item;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::attribute::Attribute;
use crate::ir::item::modifier::Modifier;
use crate::ir::item::parameter::Parameter;
use crate::ir::name::Name;
use crate::ir::statement::Statement;
use crate::ir::r#type::Type;
use crate::ir::variable::DirectVariable;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum MethodFlag {
    AssertionsInferred = 1 << 0,
    ReturnsByReference = 1 << 1,
    Yields = 1 << 2,
    Throws = 1 << 3,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Method<'arena, I, S, E> {
    pub span: Span,
    pub annotation: Option<&'arena ItemAnnotation<'arena, I, S, E>>,
    pub attributes: &'arena [Attribute<'arena, I, S, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub flags: U8Flags<MethodFlag>,
    pub modifiers: &'arena [Modifier],
    pub name: Name<'arena>,
    pub parameters: Delimited<'arena, Parameter<'arena, I, S, E>>,
    pub return_type: Option<&'arena Type<'arena>>,
    pub direct_accessed_globals: &'arena [DirectVariable<'arena>],
    pub body: Option<&'arena Statement<'arena, I, S, E>>,
}

impl From<MethodFlag> for u8 {
    fn from(flags: MethodFlag) -> Self {
        flags as u8
    }
}

impl<I, S, E> HasSpan for Method<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena, I, S, E> Item<'arena, I, S, E> for Method<'arena, I, S, E> {
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
