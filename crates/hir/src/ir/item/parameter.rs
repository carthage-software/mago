use mago_php_version::PHPVersionRange;
#[cfg(feature = "serde")]
use serde::Serialize;

use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::delimited::Delimited;
use crate::ir::expression::Expression;
use crate::ir::item::attribute::Attribute;
use crate::ir::item::member::hook::Hook;
use crate::ir::item::modifier::Modifier;
use crate::ir::r#type::Type;
use crate::ir::variable::DirectVariable;
use crate::ir::variable::annotation::VariableAnnotation;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ParameterFlag {
    ByReference = 1 << 0,
    IsVariadic = 1 << 1,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Parameter<'arena, I, S, E> {
    pub span: Span,
    pub annotation: Option<&'arena VariableAnnotation<'arena>>,
    pub attributes: &'arena [Attribute<'arena, I, S, E>],
    pub flags: U8Flags<ParameterFlag>,
    pub version_constraint: &'arena [PHPVersionRange],
    pub modifiers: &'arena [Modifier],
    pub r#type: Option<&'arena Type<'arena>>,
    pub variable: DirectVariable<'arena>,
    pub default_value: Option<&'arena Expression<'arena, I, S, E>>,
    pub hooks: Option<Delimited<'arena, Hook<'arena, I, S, E>>>,
}

impl From<ParameterFlag> for u8 {
    fn from(flag: ParameterFlag) -> Self {
        flag as u8
    }
}

impl<I, S, E> HasSpan for Parameter<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}
