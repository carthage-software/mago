use std::cmp::Ordering;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::id::SymbolId;
use crate::path::Path;
use crate::ty::Type;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct InheritedType<'arena> {
    pub span: Span,
    pub target: Path<'arena>,
    pub provenance: Provenance<'arena>,
    pub arguments: &'arena [GenericArgument<'arena>],
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum Provenance<'arena> {
    Direct,
    Inherited { via: Path<'arena> },
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct GenericArgument<'arena> {
    pub parameter: &'arena [u8],
    pub ty: Type<'arena>,
}

impl<'arena> InheritedType<'arena> {
    #[inline]
    #[must_use]
    pub const fn new(
        span: Span,
        target: Path<'arena>,
        provenance: Provenance<'arena>,
        arguments: &'arena [GenericArgument<'arena>],
    ) -> Self {
        Self { span, target, provenance, arguments }
    }
}

impl HasSpan for InheritedType<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

/// A flattened inheritance relationship (extends / implements / uses): the
/// edges plus an offset index sorted by `target.id` for O(log n) lookup by
/// ancestor.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub struct InheritedTypeList<'arena> {
    /// Flattened: direct + inherited edges (provenance distinguishes them).
    pub edges: &'arena [InheritedType<'arena>],
    /// `target.id`-sorted offsets into `edges`; O(log n) lookup by ancestor.
    pub index: &'arena [u32],
}

impl<'arena> InheritedTypeList<'arena> {
    /// The edges in source order.
    #[inline]
    #[must_use]
    pub const fn edges(&self) -> &'arena [InheritedType<'arena>] {
        self.edges
    }

    /// The number of edges in the list.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.edges.len()
    }

    /// Whether the list has no edges.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// The edge whose target is `ancestor`, found through the sorted index.
    #[must_use]
    pub fn get(&self, ancestor: SymbolId) -> Option<&'arena InheritedType<'arena>> {
        let edges = self.edges;
        let slot = self
            .index
            .binary_search_by(|&offset| {
                edges.get(offset as usize).map_or(Ordering::Greater, |edge| edge.target.id.cmp(&ancestor))
            })
            .ok()?;

        edges.get(*self.index.get(slot)? as usize)
    }

    /// Whether `ancestor` appears as a target in this list.
    #[inline]
    #[must_use]
    pub fn contains(&self, ancestor: SymbolId) -> bool {
        self.get(ancestor).is_some()
    }

    /// An iterator over the edges in source order.
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'arena, InheritedType<'arena>> {
        self.edges.iter()
    }
}

impl<'arena> IntoIterator for &InheritedTypeList<'arena> {
    type Item = &'arena InheritedType<'arena>;
    type IntoIter = std::slice::Iter<'arena, InheritedType<'arena>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
