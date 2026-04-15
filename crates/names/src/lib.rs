use std::collections::HashSet;

use foldhash::HashMap;
use mago_span::Span;
use serde::Serialize;

use mago_span::HasPosition;
use mago_span::Position;

pub mod kind;
pub mod resolver;
pub mod scope;

mod internal;

/// Stores the results of a name resolution pass over a PHP program.
///
/// Maps the start byte offset of every identifier in the source to a tuple of
/// `(end offset, resolved fully qualified name, was-imported flag)`. Storing the end
/// offset alongside the start lets callers answer "what name is at this cursor offset?"
/// without re-scanning the source for identifier boundaries.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Default)]
pub struct ResolvedNames<'arena> {
    /// Internal map: start offset -> (end offset, resolved FQN, imported flag).
    names: HashMap<u32, (u32, &'arena str, bool)>,
}

impl<'arena> ResolvedNames<'arena> {
    /// Returns the total number of resolved names stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Returns `true` if no resolved names are stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    /// Checks if a resolved name exists at the given source `Position`.
    #[must_use]
    pub fn contains(&self, position: &Position) -> bool {
        self.names.contains_key(&position.offset)
    }

    /// Gets the resolved name identifier for the given source position.
    ///
    /// # Panics
    ///
    /// Panics if no resolved name is found at the specified `position`.
    /// Use `contains` first if unsure.
    pub fn get<T: HasPosition>(&self, position: &T) -> &'arena str {
        self.names.get(&position.offset()).map(|(_, name, _)| *name).expect("resolved name not found at position")
    }

    /// Attempts to resolve the name at the given source position.
    ///
    /// Returns `Some(&str)` if a resolved name exists at the position, or `None` otherwise.
    pub fn resolve<T: HasPosition>(&self, position: &T) -> Option<&'arena str> {
        self.names.get(&position.offset()).map(|(_, name, _)| *name)
    }

    /// Checks if the name resolved at the given position originated from an explicit
    /// `use` alias or construct.
    ///
    /// Returns `false` if the name was resolved relative to the namespace, is a
    /// definition, or if no name is found at the position.
    pub fn is_imported<T: HasPosition>(&self, position: &T) -> bool {
        self.names.get(&position.offset()).is_some_and(|(_, _, imported)| *imported)
    }

    /// Returns the resolved entry whose source range covers the given byte offset.
    ///
    /// Identifier ranges in `ResolvedNames` never overlap, so at most one entry can
    /// match. Returns `Some((start, end, fqn, imported))` for the covering entry, or
    /// `None` if the offset falls outside every tracked identifier.
    pub fn at_offset(&self, offset: u32) -> Option<(u32, u32, &'arena str, bool)> {
        self.names.iter().find_map(|(&start, &(end, name, imported))| {
            if start <= offset && offset < end { Some((start, end, name, imported)) } else { None }
        })
    }

    /// Iterates over every resolved entry as `(start, end, fqn, imported)`.
    pub fn iter(&self) -> impl Iterator<Item = (u32, u32, &'arena str, bool)> + '_ {
        self.names.iter().map(|(&start, &(end, name, imported))| (start, end, name, imported))
    }

    /// Inserts a resolution result into the map (intended for internal use).
    ///
    /// The full source span of the identifier is stored, so [`at_offset`](Self::at_offset)
    /// and other range-based lookups work without re-scanning the source.
    pub(crate) fn insert_at(&mut self, span: Span, name: &'arena str, imported: bool) {
        self.names.insert(span.start.offset, (span.end.offset, name, imported));
    }

    /// Returns a `HashSet` containing every resolution result as `(&start, (fqn, imported))`.
    #[deprecated(
        note = "Allocates a HashSet for no good reason. Prefer `iter()` for allocation-free iteration, or `at_offset()` for cursor lookups."
    )]
    #[must_use]
    pub fn all(&self) -> HashSet<(&u32, (&'arena str, bool))> {
        self.names.iter().map(|(k, v)| (k, (v.1, v.2))).collect()
    }
}
