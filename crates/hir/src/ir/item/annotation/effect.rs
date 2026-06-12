#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::name::Name;
use crate::ir::r#type::annotation::TypeAnnotation;
use crate::ir::variable::DirectVariable;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct ThrowsAnnotation<'arena> {
    pub span: Span,
    pub r#type: &'arena TypeAnnotation<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct AssertAnnotation<'arena> {
    pub span: Span,
    pub negated: bool,
    pub equality: bool,
    pub pattern: AssertAnnotationPattern<'arena>,
    pub target: AssertAnnotationTarget<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct AssertAnnotationPattern<'arena> {
    pub span: Span,
    pub kind: AssertAnnotationPatternKind<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum AssertAnnotationPatternKind<'arena> {
    Type(&'arena TypeAnnotation<'arena>),
    Truthy,
    Falsy,
    NonEmpty,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct AssertAnnotationTarget<'arena> {
    pub span: Span,
    pub kind: AssertAnnotationTargetKind<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum AssertAnnotationTargetKind<'arena> {
    Variable(DirectVariable<'arena>),
    Method(DirectVariable<'arena>, Name<'arena>),
    Property(DirectVariable<'arena>, Name<'arena>),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct SelfOutAnnotation<'arena> {
    pub span: Span,
    pub r#type: &'arena TypeAnnotation<'arena>,
}

impl HasSpan for ThrowsAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for AssertAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for SelfOutAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for AssertAnnotationPattern<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSpan for AssertAnnotationTarget<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl CopyInto for ThrowsAnnotation<'_> {
    type Output<'arena> = ThrowsAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        ThrowsAnnotation { span: self.span, r#type: copy_ref_into(self.r#type, arena) }
    }
}

impl CopyInto for AssertAnnotation<'_> {
    type Output<'arena> = AssertAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        AssertAnnotation {
            span: self.span,
            negated: self.negated,
            equality: self.equality,
            pattern: self.pattern.copy_into(arena),
            target: self.target.copy_into(arena),
        }
    }
}

impl CopyInto for AssertAnnotationPattern<'_> {
    type Output<'arena> = AssertAnnotationPattern<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        AssertAnnotationPattern { span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl CopyInto for AssertAnnotationPatternKind<'_> {
    type Output<'arena> = AssertAnnotationPatternKind<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            AssertAnnotationPatternKind::Type(r#type) => {
                AssertAnnotationPatternKind::Type(copy_ref_into(r#type, arena))
            }
            AssertAnnotationPatternKind::Truthy => AssertAnnotationPatternKind::Truthy,
            AssertAnnotationPatternKind::Falsy => AssertAnnotationPatternKind::Falsy,
            AssertAnnotationPatternKind::NonEmpty => AssertAnnotationPatternKind::NonEmpty,
        }
    }
}

impl CopyInto for AssertAnnotationTarget<'_> {
    type Output<'arena> = AssertAnnotationTarget<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        AssertAnnotationTarget { span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl CopyInto for AssertAnnotationTargetKind<'_> {
    type Output<'arena> = AssertAnnotationTargetKind<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            AssertAnnotationTargetKind::Variable(variable) => {
                AssertAnnotationTargetKind::Variable(variable.copy_into(arena))
            }
            AssertAnnotationTargetKind::Method(variable, name) => {
                AssertAnnotationTargetKind::Method(variable.copy_into(arena), name.copy_into(arena))
            }
            AssertAnnotationTargetKind::Property(variable, name) => {
                AssertAnnotationTargetKind::Property(variable.copy_into(arena), name.copy_into(arena))
            }
        }
    }
}

impl CopyInto for SelfOutAnnotation<'_> {
    type Output<'arena> = SelfOutAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        SelfOutAnnotation { span: self.span, r#type: copy_ref_into(self.r#type, arena) }
    }
}
