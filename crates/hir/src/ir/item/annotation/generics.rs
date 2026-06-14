#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_phpdoc_syntax::cst::tag::TemplateTagValueVariance;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::identifier::Identifier;
use crate::ir::name::Name;
use crate::ir::r#type::annotation::TypeAnnotation;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_ref_into;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum TypeParameterDefiningEntity<'arena> {
    ClassLike(Identifier<'arena>),
    Function(Identifier<'arena>),
    Method(Identifier<'arena>, Name<'arena>),
    Closure(Span),
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum Variance {
    Invariant,
    Covariant,
    Contravariant,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct TypeParameterAnnotation<'arena> {
    pub span: Span,
    pub variance: Variance,
    pub name: Name<'arena>,
    pub bound: Option<&'arena TypeAnnotation<'arena>>,
    pub default: Option<&'arena TypeAnnotation<'arena>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct InheritedTypeParameterAnnotation<'arena> {
    pub span: Span,
    pub defining_entity: TypeParameterDefiningEntity<'arena>,
    pub name: Name<'arena>,
    pub bound: Option<&'arena TypeAnnotation<'arena>>,
    pub default: Option<&'arena TypeAnnotation<'arena>>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct WhereConstraintAnnotation<'arena> {
    pub span: Span,
    pub type_parameter: Name<'arena>,
    pub constraint: &'arena TypeAnnotation<'arena>,
}

impl CopyInto for Variance {
    type Output<'arena> = Variance;

    fn copy_into<'arena, A>(&self, _arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        *self
    }
}

impl CopyInto for TypeParameterDefiningEntity<'_> {
    type Output<'arena> = TypeParameterDefiningEntity<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            TypeParameterDefiningEntity::ClassLike(identifier) => {
                TypeParameterDefiningEntity::ClassLike(identifier.copy_into(arena))
            }
            TypeParameterDefiningEntity::Function(identifier) => {
                TypeParameterDefiningEntity::Function(identifier.copy_into(arena))
            }
            TypeParameterDefiningEntity::Method(identifier, name) => {
                TypeParameterDefiningEntity::Method(identifier.copy_into(arena), name.copy_into(arena))
            }
            TypeParameterDefiningEntity::Closure(span) => TypeParameterDefiningEntity::Closure(span),
        }
    }
}

impl CopyInto for TypeParameterAnnotation<'_> {
    type Output<'arena> = TypeParameterAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        TypeParameterAnnotation {
            span: self.span,
            variance: self.variance,
            name: self.name.copy_into(arena),
            bound: self.bound.map(|bound| copy_ref_into(bound, arena)),
            default: self.default.map(|default| copy_ref_into(default, arena)),
        }
    }
}

impl CopyInto for InheritedTypeParameterAnnotation<'_> {
    type Output<'arena> = InheritedTypeParameterAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        InheritedTypeParameterAnnotation {
            span: self.span,
            defining_entity: self.defining_entity.copy_into(arena),
            name: self.name.copy_into(arena),
            bound: self.bound.map(|bound| copy_ref_into(bound, arena)),
            default: self.default.map(|default| copy_ref_into(default, arena)),
        }
    }
}

impl CopyInto for WhereConstraintAnnotation<'_> {
    type Output<'arena> = WhereConstraintAnnotation<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        WhereConstraintAnnotation {
            span: self.span,
            type_parameter: self.type_parameter.copy_into(arena),
            constraint: copy_ref_into(self.constraint, arena),
        }
    }
}

impl From<TemplateTagValueVariance> for Variance {
    fn from(variance: TemplateTagValueVariance) -> Self {
        match variance {
            TemplateTagValueVariance::Invariant => Variance::Invariant,
            TemplateTagValueVariance::Covariant => Variance::Covariant,
            TemplateTagValueVariance::Contravariant => Variance::Contravariant,
        }
    }
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

impl HasSpan for InheritedTypeParameterAnnotation<'_> {
    fn span(&self) -> Span {
        self.span
    }
}
