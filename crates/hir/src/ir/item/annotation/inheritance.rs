#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::r#type::annotation::NamedTypeAnnotation;
use crate::ir::r#type::annotation::TypeAnnotationKind;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_slice_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ImplementsAnnotation<'arena> {
    pub span: Span,
    pub r#type: NamedTypeAnnotation<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ExtendsAnnotation<'arena> {
    pub span: Span,
    pub r#type: NamedTypeAnnotation<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct UseAnnotation<'arena> {
    pub span: Span,
    pub r#type: NamedTypeAnnotation<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct SealedAnnotation<'arena> {
    pub span: Span,
    pub types: &'arena [NamedTypeAnnotation<'arena>],
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct RequireExtendsAnnotation<'arena> {
    pub span: Span,
    pub r#type: NamedTypeAnnotation<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct RequireImplementsAnnotation<'arena> {
    pub span: Span,
    pub r#type: NamedTypeAnnotation<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct MixinAnnotation<'arena> {
    pub span: Span,
    pub r#type: TypeAnnotationKind<'arena>,
}

impl CopyInto for ImplementsAnnotation<'_> {
    type Output<'arena> = ImplementsAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ImplementsAnnotation { span: self.span, r#type: self.r#type.copy_into(arena) }
    }
}

impl CopyInto for ExtendsAnnotation<'_> {
    type Output<'arena> = ExtendsAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ExtendsAnnotation { span: self.span, r#type: self.r#type.copy_into(arena) }
    }
}

impl CopyInto for UseAnnotation<'_> {
    type Output<'arena> = UseAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        UseAnnotation { span: self.span, r#type: self.r#type.copy_into(arena) }
    }
}

impl CopyInto for SealedAnnotation<'_> {
    type Output<'arena> = SealedAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        SealedAnnotation { span: self.span, types: copy_slice_into(self.types, arena) }
    }
}

impl CopyInto for RequireExtendsAnnotation<'_> {
    type Output<'arena> = RequireExtendsAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        RequireExtendsAnnotation { span: self.span, r#type: self.r#type.copy_into(arena) }
    }
}

impl CopyInto for RequireImplementsAnnotation<'_> {
    type Output<'arena> = RequireImplementsAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        RequireImplementsAnnotation { span: self.span, r#type: self.r#type.copy_into(arena) }
    }
}

impl CopyInto for MixinAnnotation<'_> {
    type Output<'arena> = MixinAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        MixinAnnotation { span: self.span, r#type: self.r#type.copy_into(arena) }
    }
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

impl HasSpan for UseAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
