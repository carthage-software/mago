use std::cmp::Ordering;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::hash::Hash;
use std::hash::Hasher;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_slice_into;

use crate::id::SymbolId;

/// One segment of a [`Path`]. The variant fixes how the segment folds when its
/// [`SymbolId`] is computed, which mirrors PHP's per-entity case rules.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum PathSegment<'arena> {
    /// A namespaced name whose leaf is case-sensitive: the namespace folds, the
    /// short name is kept verbatim (a global constant).
    QualifiedSensative(&'arena [u8]),
    /// A fully case-insensitive name (class-like, function, method, hook).
    QualifiedInsensative(&'arena [u8]),
    /// A case-sensitive member name without a leading sigil (class constant,
    /// enum case, property).
    Name(&'arena [u8]),
    /// A case-sensitive variable name, sigil (`$`) included.
    Variable(&'arena [u8]),
}

impl<'arena> PathSegment<'arena> {
    #[inline]
    #[must_use]
    pub const fn as_bytes(self) -> &'arena [u8] {
        match self {
            PathSegment::QualifiedSensative(bytes)
            | PathSegment::QualifiedInsensative(bytes)
            | PathSegment::Name(bytes)
            | PathSegment::Variable(bytes) => bytes,
        }
    }
}

impl CopyInto for PathSegment<'_> {
    type Output<'arena> = PathSegment<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        match *self {
            PathSegment::QualifiedSensative(bytes) => PathSegment::QualifiedSensative(arena.alloc_slice_copy(bytes)),
            PathSegment::QualifiedInsensative(bytes) => {
                PathSegment::QualifiedInsensative(arena.alloc_slice_copy(bytes))
            }
            PathSegment::Name(bytes) => PathSegment::Name(arena.alloc_slice_copy(bytes)),
            PathSegment::Variable(bytes) => PathSegment::Variable(arena.alloc_slice_copy(bytes)),
        }
    }
}

/// The identity of a symbol or member: an ordered list of [`PathPart`]s plus a
/// precomputed [`SymbolId`]. `App\Model\User::find` is three parts; a bare
/// class is one.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct Path<'arena> {
    pub id: SymbolId,
    pub segments: &'arena [PathSegment<'arena>],
}

impl<'arena> Path<'arena> {
    #[inline]
    fn new(id: SymbolId, segments: &'arena [PathSegment<'arena>]) -> Self {
        Path { id, segments }
    }

    #[must_use]
    pub fn constant<A>(arena: &'arena A, name: &'arena [u8]) -> Self
    where
        A: Arena,
    {
        Self::new(
            SymbolId::constant(name),
            arena.alloc_slice_copy(&[PathSegment::QualifiedSensative(strip_leading_separator(name))]),
        )
    }

    #[must_use]
    pub fn class_like<A>(arena: &'arena A, name: &'arena [u8]) -> Self
    where
        A: Arena,
    {
        Self::new(
            SymbolId::class_like(name),
            arena.alloc_slice_copy(&[PathSegment::QualifiedInsensative(strip_leading_separator(name))]),
        )
    }

    #[must_use]
    pub fn function_like<A>(arena: &'arena A, name: &'arena [u8]) -> Self
    where
        A: Arena,
    {
        Self::new(
            SymbolId::function_like(name),
            arena.alloc_slice_copy(&[PathSegment::QualifiedInsensative(strip_leading_separator(name))]),
        )
    }

    #[must_use]
    pub fn class_like_constant<A>(arena: &'arena A, class_name: &'arena [u8], name: &'arena [u8]) -> Self
    where
        A: Arena,
    {
        Self::new(
            SymbolId::class_like_constant(class_name, name),
            arena.alloc_slice_copy(&[
                PathSegment::QualifiedInsensative(strip_leading_separator(class_name)),
                PathSegment::Name(name),
            ]),
        )
    }

    #[must_use]
    pub fn type_alias<A>(arena: &'arena A, class_name: &'arena [u8], name: &'arena [u8]) -> Self
    where
        A: Arena,
    {
        Self::new(
            SymbolId::type_alias(class_name, name),
            arena.alloc_slice_copy(&[
                PathSegment::QualifiedInsensative(strip_leading_separator(class_name)),
                PathSegment::Name(name),
            ]),
        )
    }

    #[must_use]
    pub fn enum_case<A>(arena: &'arena A, enum_name: &'arena [u8], name: &'arena [u8]) -> Self
    where
        A: Arena,
    {
        Self::new(
            SymbolId::enum_case(enum_name, name),
            arena.alloc_slice_copy(&[
                PathSegment::QualifiedInsensative(strip_leading_separator(enum_name)),
                PathSegment::Name(name),
            ]),
        )
    }

    #[must_use]
    pub fn property<A>(arena: &'arena A, class_name: &'arena [u8], name: &'arena [u8]) -> Self
    where
        A: Arena,
    {
        Self::new(
            SymbolId::property(class_name, name),
            arena.alloc_slice_copy(&[
                PathSegment::QualifiedInsensative(strip_leading_separator(class_name)),
                PathSegment::Name(name),
            ]),
        )
    }

    #[must_use]
    pub fn property_hook<A>(
        arena: &'arena A,
        class_name: &'arena [u8],
        property_name: &'arena [u8],
        name: &'arena [u8],
    ) -> Self
    where
        A: Arena,
    {
        Self::new(
            SymbolId::property_hook(class_name, property_name, name),
            arena.alloc_slice_copy(&[
                PathSegment::QualifiedInsensative(strip_leading_separator(class_name)),
                PathSegment::Name(property_name),
                PathSegment::QualifiedInsensative(name),
            ]),
        )
    }

    #[must_use]
    pub fn method<A>(arena: &'arena A, class_name: &'arena [u8], name: &'arena [u8]) -> Self
    where
        A: Arena,
    {
        Self::new(
            SymbolId::method(class_name, name),
            arena.alloc_slice_copy(&[
                PathSegment::QualifiedInsensative(strip_leading_separator(class_name)),
                PathSegment::QualifiedInsensative(name),
            ]),
        )
    }

    #[must_use]
    pub fn function_like_parameter<A>(arena: &'arena A, function_like: &'arena [u8], parameter: &'arena [u8]) -> Self
    where
        A: Arena,
    {
        Self::new(
            SymbolId::function_like_parameter(function_like, parameter),
            arena.alloc_slice_copy(&[
                PathSegment::QualifiedInsensative(strip_leading_separator(function_like)),
                PathSegment::Variable(parameter),
            ]),
        )
    }

    #[must_use]
    pub fn method_parameter<A>(
        arena: &'arena A,
        class_name: &'arena [u8],
        method: &'arena [u8],
        parameter: &'arena [u8],
    ) -> Self
    where
        A: Arena,
    {
        Self::new(
            SymbolId::method_parameter(class_name, method, parameter),
            arena.alloc_slice_copy(&[
                PathSegment::QualifiedInsensative(strip_leading_separator(class_name)),
                PathSegment::QualifiedInsensative(method),
                PathSegment::Variable(parameter),
            ]),
        )
    }

    #[must_use]
    pub fn property_hook_parameter<A>(
        arena: &'arena A,
        class_name: &'arena [u8],
        property_name: &'arena [u8],
        hook: &'arena [u8],
        parameter: &'arena [u8],
    ) -> Self
    where
        A: Arena,
    {
        Self::new(
            SymbolId::property_hook_parameter(class_name, property_name, hook, parameter),
            arena.alloc_slice_copy(&[
                PathSegment::QualifiedInsensative(strip_leading_separator(class_name)),
                PathSegment::Name(property_name),
                PathSegment::QualifiedInsensative(hook),
                PathSegment::Variable(parameter),
            ]),
        )
    }

    /// The bytes of the leaf segment (the most specific name). For a bare class
    /// this is the whole name; for `User::find` it is `find`.
    #[inline]
    #[must_use]
    pub fn as_bytes(self) -> &'arena [u8] {
        self.segments.last().map_or(b"", |part| part.as_bytes())
    }
}

impl PartialEq for Path<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.segments == other.segments
    }
}

impl Eq for Path<'_> {}

impl Ord for Path<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.segments.cmp(other.segments)
    }
}

impl PartialOrd for Path<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Path<'_> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.segments.hash(state);
    }
}

impl Display for Path<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        for (index, part) in self.segments.iter().enumerate() {
            if index > 0 {
                f.write_str("::")?;
            }

            f.write_str(&String::from_utf8_lossy(part.as_bytes()))?;
        }

        Ok(())
    }
}

impl CopyInto for Path<'_> {
    type Output<'arena> = Path<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Path { id: self.id, segments: copy_slice_into(self.segments, arena) }
    }
}

fn strip_leading_separator(name: &[u8]) -> &[u8] {
    name.strip_prefix(b"\\").unwrap_or(name)
}
