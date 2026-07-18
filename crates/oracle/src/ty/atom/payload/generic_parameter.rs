use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;

use crate::path::Path;
use crate::ty::Type;

/// A template parameter occurrence: `T` of `Foo` (or of `Foo::bar`,
/// or of function `baz`), constrained by its declared bound.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GenericParameterAtom<'arena> {
    pub name: &'arena [u8],
    pub defining_entity: DefiningEntity<'arena>,
    pub constraint: Type<'arena>,
}

/// Where a template parameter is declared.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DefiningEntity<'arena> {
    ClassLike(Path<'arena>),
    Method { class: Path<'arena>, method: &'arena [u8] },
    Function(Path<'arena>),
}

impl Display for DefiningEntity<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            DefiningEntity::ClassLike(name) => name.fmt(f),
            DefiningEntity::Method { class, method } => {
                write!(f, "{}::{}", class, String::from_utf8_lossy(method))
            }
            DefiningEntity::Function(name) => name.fmt(f),
        }
    }
}

impl Display for GenericParameterAtom<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "'{}.{} extends {}", String::from_utf8_lossy(self.name), self.defining_entity, self.constraint)
    }
}

impl CopyInto for GenericParameterAtom<'_> {
    type Output<'arena> = GenericParameterAtom<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        GenericParameterAtom {
            name: arena.alloc_slice_copy(self.name),
            defining_entity: self.defining_entity.copy_into(arena),
            constraint: self.constraint.copy_into(arena),
        }
    }
}

impl CopyInto for DefiningEntity<'_> {
    type Output<'arena> = DefiningEntity<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            DefiningEntity::ClassLike(name) => DefiningEntity::ClassLike(name.copy_into(arena)),
            DefiningEntity::Method { class, method } => {
                DefiningEntity::Method { class: class.copy_into(arena), method: arena.alloc_slice_copy(method) }
            }
            DefiningEntity::Function(name) => DefiningEntity::Function(name.copy_into(arena)),
        }
    }
}
