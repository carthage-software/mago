#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_span::HasSpan;
use mago_span::Span;

use crate::ir::identifier::Identifier;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_slice_into;

pub mod annotation;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Type<'arena> {
    pub span: Span,
    pub kind: TypeKind<'arena>,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", content = "value"))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum TypeKind<'arena> {
    Named(Identifier<'arena>),
    Union(&'arena [Type<'arena>]),
    Intersection(&'arena [Type<'arena>]),
    Null,
    Array,
    Callable,
    Static(Identifier<'arena>),
    Self_(Identifier<'arena>),
    Parent(Identifier<'arena>),
    Void,
    Never,
    Float,
    Bool(Option<bool>),
    Integer,
    String,
    Object,
    Mixed,
    Iterable,
}

impl HasSpan for Type<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

impl CopyInto for Type<'_> {
    type Output<'arena> = Type<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Type { span: self.span, kind: self.kind.copy_into(arena) }
    }
}

impl CopyInto for TypeKind<'_> {
    type Output<'arena> = TypeKind<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            TypeKind::Named(identifier) => TypeKind::Named(identifier.copy_into(arena)),
            TypeKind::Union(kinds) => TypeKind::Union(copy_slice_into(kinds, arena)),
            TypeKind::Intersection(kinds) => TypeKind::Intersection(copy_slice_into(kinds, arena)),
            TypeKind::Null => TypeKind::Null,
            TypeKind::Array => TypeKind::Array,
            TypeKind::Callable => TypeKind::Callable,
            TypeKind::Static(identifier) => TypeKind::Static(identifier.copy_into(arena)),
            TypeKind::Self_(identifier) => TypeKind::Self_(identifier.copy_into(arena)),
            TypeKind::Parent(identifier) => TypeKind::Parent(identifier.copy_into(arena)),
            TypeKind::Void => TypeKind::Void,
            TypeKind::Never => TypeKind::Never,
            TypeKind::Float => TypeKind::Float,
            TypeKind::Bool(value) => TypeKind::Bool(value),
            TypeKind::Integer => TypeKind::Integer,
            TypeKind::String => TypeKind::String,
            TypeKind::Object => TypeKind::Object,
            TypeKind::Mixed => TypeKind::Mixed,
            TypeKind::Iterable => TypeKind::Iterable,
        }
    }
}
