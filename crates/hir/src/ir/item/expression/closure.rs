#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_flags::U8Flags;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::delimited::Delimited;
use crate::ir::item::Item;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::attribute::Attribute;
use crate::ir::item::parameter::Parameter;
use crate::ir::statement::Statement;
use crate::ir::r#type::Type;
use crate::ir::variable::DirectVariable;
use mago_allocator::copy::CopyInto;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ClosureFlag {
    Static = 1 << 0,
    ReturnsByReference = 1 << 1,
    AssertionsInferred = 1 << 2,
    Yields = 1 << 3,
    Throws = 1 << 4,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum ClosureUseClauseVariableFlag {
    ByReference = 1 << 0,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Closure<'arena, I, S, E> {
    pub span: Span,
    pub annotation: Option<&'arena ItemAnnotation<'arena, I, S, E>>,
    pub attributes: &'arena [Attribute<'arena, I, S, E>],
    pub flags: U8Flags<ClosureFlag>,
    pub parameters: Delimited<'arena, Parameter<'arena, I, S, E>>,
    pub return_type: Option<&'arena Type<'arena>>,
    pub use_variables: Option<Delimited<'arena, ClosureUseClauseVariable<'arena>>>,
    pub direct_accessed_globals: &'arena [DirectVariable<'arena>],
    pub body: &'arena Statement<'arena, I, S, E>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ClosureUseClauseVariable<'arena> {
    pub span: Span,
    pub flags: U8Flags<ClosureUseClauseVariableFlag>,
    pub variable: DirectVariable<'arena>,
}

impl<I, S, E> Closure<'_, I, S, E> {
    #[must_use]
    pub fn has_annotation(&self) -> bool {
        self.annotation.is_some()
    }

    #[must_use]
    pub const fn is_static(&self) -> bool {
        self.flags.contains_bits(ClosureFlag::Static as u8)
    }

    #[must_use]
    pub const fn returns_by_reference(&self) -> bool {
        self.flags.contains_bits(ClosureFlag::ReturnsByReference as u8)
    }

    #[must_use]
    pub const fn assertions_inferred(&self) -> bool {
        self.flags.contains_bits(ClosureFlag::AssertionsInferred as u8)
    }
}

impl ClosureUseClauseVariable<'_> {
    #[must_use]
    pub const fn is_by_reference(&self) -> bool {
        self.flags.contains_bits(ClosureUseClauseVariableFlag::ByReference as u8)
    }
}

impl From<ClosureFlag> for u8 {
    fn from(flag: ClosureFlag) -> Self {
        flag as u8
    }
}

impl From<ClosureUseClauseVariableFlag> for u8 {
    fn from(flag: ClosureUseClauseVariableFlag) -> Self {
        flag as u8
    }
}

impl<I, S, E> HasSpan for Closure<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for ClosureUseClauseVariable<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'arena, I, S, E> Item<'arena, I, S, E> for Closure<'arena, I, S, E> {
    fn attributes(&self) -> &'arena [Attribute<'arena, I, S, E>] {
        self.attributes
    }

    fn annotation(&self) -> Option<&'arena ItemAnnotation<'arena, I, S, E>> {
        self.annotation
    }
}

impl CopyInto for ClosureUseClauseVariable<'_> {
    type Output<'arena> = ClosureUseClauseVariable<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ClosureUseClauseVariable { span: self.span, flags: self.flags, variable: self.variable.copy_into(arena) }
    }
}
