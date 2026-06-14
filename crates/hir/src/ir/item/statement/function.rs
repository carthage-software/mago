#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_flags::U8Flags;
use mago_php_version::PHPVersionRange;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::delimited::Delimited;
use crate::ir::identifier::Identifier;
use crate::ir::item::Item;
use crate::ir::item::annotation::ItemAnnotation;
use crate::ir::item::attribute::Attribute;
use crate::ir::item::parameter::Parameter;
use crate::ir::statement::Statement;
use crate::ir::r#type::Type;
use crate::ir::variable::DirectVariable;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_allocator::copy::copy_slice_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum FunctionFlag {
    AssertionsInferred = 1 << 0,
    ReturnsByReference = 1 << 1,
    Yields = 1 << 2,
    Throws = 1 << 3,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Function<'arena, I, S, E> {
    pub span: Span,
    pub annotation: Option<&'arena ItemAnnotation<'arena, I, S, E>>,
    pub attributes: &'arena [Attribute<'arena, I, S, E>],
    pub version_constraint: &'arena [PHPVersionRange],
    pub flags: U8Flags<FunctionFlag>,
    pub name: Identifier<'arena>,
    pub parameters: Delimited<'arena, Parameter<'arena, I, S, E>>,
    pub return_type: Option<&'arena Type<'arena>>,
    pub direct_accessed_globals: &'arena [DirectVariable<'arena>],
    pub body: &'arena Statement<'arena, I, S, E>,
}

impl CopyInto for FunctionFlag {
    type Output<'arena> = FunctionFlag;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl<I, S, E> CopyInto for Function<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = Function<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Function {
            span: self.span,
            annotation: self.annotation.map(|node| copy_ref_into(node, arena)),
            attributes: copy_slice_into(self.attributes, arena),
            version_constraint: arena.alloc_slice_copy(self.version_constraint),
            flags: self.flags,
            name: self.name.copy_into(arena),
            parameters: self.parameters.copy_into(arena),
            return_type: self.return_type.map(|node| copy_ref_into(node, arena)),
            direct_accessed_globals: copy_slice_into(self.direct_accessed_globals, arena),
            body: copy_ref_into(self.body, arena),
        }
    }
}

impl From<FunctionFlag> for u8 {
    fn from(flag: FunctionFlag) -> Self {
        flag as u8
    }
}

impl<'arena, I, S, E> Item<'arena, I, S, E> for Function<'arena, I, S, E> {
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

impl<I, S, E> HasSpan for Function<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}
