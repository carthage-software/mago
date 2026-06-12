#[cfg(feature = "serde")]
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::argument::Argument;
use crate::ir::delimited::Delimited;
use crate::ir::identifier::Identifier;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u16)]
pub enum AttributeTarget {
    Class = 1 << 0,
    Function = 1 << 1,
    Method = 1 << 2,
    Property = 1 << 3,
    ClassConstant = 1 << 4,
    Parameter = 1 << 5,
    Constant = 1 << 6,
    Repeatable = 1 << 7,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Attribute<'arena, I, S, E> {
    pub span: Span,
    pub class: Identifier<'arena>,
    pub arguments: Option<Delimited<'arena, Argument<'arena, I, S, E>>>,
}

impl<I, S, E> Attribute<'_, I, S, E> {
    #[inline]
    #[must_use]
    pub const fn is_allow_dynamic_properties(&self) -> bool {
        self.class.value.eq_ignore_ascii_case(b"AllowDynamicProperties")
    }

    #[inline]
    #[must_use]
    pub const fn is_delayed_target_validation(&self) -> bool {
        self.class.value.eq_ignore_ascii_case(b"DelayedTargetValidation")
    }

    #[inline]
    #[must_use]
    pub const fn is_deprecated(&self) -> bool {
        self.class.value.eq_ignore_ascii_case(b"Deprecated")
    }

    #[inline]
    #[must_use]
    pub const fn is_no_discard(&self) -> bool {
        self.class.value.eq_ignore_ascii_case(b"NoDiscard")
    }

    #[inline]
    #[must_use]
    pub const fn is_override(&self) -> bool {
        self.class.value.eq_ignore_ascii_case(b"Override")
    }

    #[inline]
    #[must_use]
    pub const fn is_return_type_will_change(&self) -> bool {
        self.class.value.eq_ignore_ascii_case(b"ReturnTypeWillChange")
    }

    #[inline]
    #[must_use]
    pub const fn is_sensitive_parameter(&self) -> bool {
        self.class.value.eq_ignore_ascii_case(b"SensitiveParameter")
    }
}

impl From<AttributeTarget> for u16 {
    fn from(value: AttributeTarget) -> Self {
        value as u16
    }
}

impl<I, S, E> HasSpan for Attribute<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}
