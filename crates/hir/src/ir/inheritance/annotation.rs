use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::r#type::annotation::IdentifierTypeAnnotation;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ImplementsAnnotation<'arena> {
    pub span: Span,
    pub r#type: IdentifierTypeAnnotation<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ExtendsAnnotation<'arena> {
    pub span: Span,
    pub r#type: IdentifierTypeAnnotation<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct UseAnnotation<'arena> {
    pub span: Span,
    pub r#type: IdentifierTypeAnnotation<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct SealedAnnotation<'arena> {
    pub span: Span,
    pub types: &'arena [IdentifierTypeAnnotation<'arena>],
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct RequireExtendsAnnotation<'arena> {
    pub span: Span,
    pub r#type: IdentifierTypeAnnotation<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct RequireImplementsAnnotation<'arena> {
    pub span: Span,
    pub r#type: IdentifierTypeAnnotation<'arena>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct MixinAnnotation<'arena> {
    pub span: Span,
    pub r#type: IdentifierTypeAnnotation<'arena>,
}

impl HasSpan for ImplementsAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for ExtendsAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for SealedAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for RequireExtendsAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for RequireImplementsAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for MixinAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
