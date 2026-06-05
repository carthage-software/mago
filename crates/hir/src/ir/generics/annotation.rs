use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::generics::TypeParameterDefiningEntity;
use crate::ir::generics::Variance;
use crate::ir::name::Name;
use crate::ir::r#type::annotation::TypeAnnotation;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TypeParameterAnnotation<'arena> {
    pub span: Span,
    pub variance: Variance,
    pub name: Name<'arena>,
    pub bound: Option<&'arena TypeAnnotation<'arena>>,
    pub default: Option<&'arena TypeAnnotation<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct InheritedTemplateAnnotation<'arena> {
    pub defining_entity: TypeParameterDefiningEntity<'arena>,
    pub name: Name<'arena>,
    pub bound: Option<&'arena TypeAnnotation<'arena>>,
    pub default: Option<&'arena TypeAnnotation<'arena>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct WhereConstraintAnnotation<'arena> {
    pub span: Span,
    pub type_parameter: Name<'arena>,
    pub constraint: &'arena TypeAnnotation<'arena>,
}

impl HasSpan for TypeParameterAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for WhereConstraintAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
