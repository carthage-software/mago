#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::delimited::Delimited;
use crate::ir::item::annotation::generics::TypeParameterAnnotation;
use crate::ir::item::annotation::parameter::ParameterAnnotation;
use crate::ir::item::modifier::Visibility;
use crate::ir::name::Name;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct MethodAnnotation<'arena, I, S, E> {
    pub span: Span,
    pub visibility: Option<Visibility>,
    pub r#static: bool,
    pub name: Name<'arena>,
    pub type_parameters: Option<Delimited<'arena, TypeParameterAnnotation<'arena>>>,
    pub parameters: Delimited<'arena, ParameterAnnotation<'arena, I, S, E>>,
    pub return_type: Option<&'arena TypeAnnotation<'arena>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct PropertyAnnotation<'arena> {
    pub span: Span,
    pub kind: PropertyAnnotationKind,
    pub r#type: Option<&'arena TypeAnnotation<'arena>>,
    pub variable: DirectVariable<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum PropertyAnnotationKind {
    Read,
    Write,
    ReadWrite,
}

impl<I, S, E> CopyInto for MethodAnnotation<'_, I, S, E>
where
    I: CopyInto,
    S: CopyInto,
    E: CopyInto,
{
    type Output<'arena> = MethodAnnotation<'arena, I::Output<'arena>, S::Output<'arena>, E::Output<'arena>>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        MethodAnnotation {
            span: self.span,
            visibility: self.visibility.map(|node| node.copy_into(arena)),
            r#static: self.r#static,
            name: self.name.copy_into(arena),
            type_parameters: self.type_parameters.map(|node| node.copy_into(arena)),
            parameters: self.parameters.copy_into(arena),
            return_type: self.return_type.map(|node| copy_ref_into(node, arena)),
        }
    }
}

impl CopyInto for PropertyAnnotation<'_> {
    type Output<'arena> = PropertyAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        PropertyAnnotation {
            span: self.span,
            kind: self.kind,
            r#type: self.r#type.map(|r#type| copy_ref_into(r#type, arena)),
            variable: self.variable.copy_into(arena),
        }
    }
}

impl CopyInto for PropertyAnnotationKind {
    type Output<'arena> = PropertyAnnotationKind;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl<I, S, E> HasSpan for MethodAnnotation<'_, I, S, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for PropertyAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
