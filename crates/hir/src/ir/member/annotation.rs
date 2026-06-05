use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::generics::annotation::TypeParameterAnnotation;
use crate::ir::identifier::Identifier;
use crate::ir::modifier::Visibility;
use crate::ir::name::Name;
use crate::ir::parameter::annotation::ParameterAnnotation;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TypeAliasAnnotation<'arena> {
    pub span: Span,
    pub name: Name<'arena>,
    pub r#type: &'arena TypeAnnotation<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ImportedTypeAliasAnnotation<'arena> {
    pub span: Span,
    pub name: Name<'arena>,
    pub from: Identifier<'arena>,
    pub r#as: Option<Name<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MethodAnnotation<'arena, S, D, E> {
    pub span: Span,
    pub visibility: Option<Visibility>,
    pub r#static: bool,
    pub name: Name<'arena>,
    pub type_parameters: &'arena [TypeParameterAnnotation<'arena>],
    pub parameters: &'arena [ParameterAnnotation<'arena, S, D, E>],
    pub return_type: Option<&'arena TypeAnnotation<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[serde(tag = "type", content = "value")]
pub enum PropertyAnnotationKind {
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct PropertyAnnotation<'arena> {
    pub span: Span,
    pub kind: PropertyAnnotationKind,
    pub r#type: Option<&'arena TypeAnnotation<'arena>>,
    pub variable: DirectVariable<'arena>,
}

impl HasSpan for TypeAliasAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for ImportedTypeAliasAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<S, D, E> HasSpan for MethodAnnotation<'_, S, D, E> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for PropertyAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
