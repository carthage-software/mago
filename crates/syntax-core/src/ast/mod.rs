//! Shared AST primitives used across every syntax crate.

use std::slice::Iter;

use bumpalo::Bump;
use bumpalo::collections::Vec as BVec;
use bumpalo::collections::vec::IntoIter;
use serde::Serialize;

use mago_span::HasPosition;
use mago_span::HasSpan;
use mago_span::Span;

/// A sequence of AST nodes allocated in a [`bumpalo::Bump`].
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Sequence<'arena, T> {
    pub nodes: BVec<'arena, T>,
}

impl<'arena, T> Sequence<'arena, T> {
    #[inline]
    #[must_use]
    pub const fn new(inner: BVec<'arena, T>) -> Self {
        Self { nodes: inner }
    }

    #[inline]
    pub fn empty(arena: &'arena Bump) -> Self {
        Self { nodes: BVec::new_in(arena) }
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.nodes.get(index)
    }

    #[inline]
    #[must_use]
    pub fn first(&self) -> Option<&T> {
        self.nodes.first()
    }

    #[inline]
    #[must_use]
    pub fn last(&self) -> Option<&T> {
        self.nodes.last()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.nodes.iter()
    }

    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        self.nodes.as_slice()
    }
}

impl<'arena, T: HasSpan> Sequence<'arena, T> {
    #[inline]
    #[must_use]
    pub fn first_span(&self) -> Option<Span> {
        self.nodes.first().map(HasSpan::span)
    }

    #[inline]
    #[must_use]
    pub fn last_span(&self) -> Option<Span> {
        self.nodes.last().map(HasSpan::span)
    }

    /// Compute the span covering the sequence, anchored at `from`.
    ///
    /// Returns a zero-width span at `from` when the sequence is empty.
    #[inline]
    #[must_use]
    pub fn span(&self, file_id: mago_database::file::FileId, from: mago_span::Position) -> Span {
        self.last_span().map_or(Span::new(file_id, from, from), |span| Span::new(file_id, from, span.end))
    }
}

impl<'arena, T> IntoIterator for Sequence<'arena, T> {
    type Item = T;
    type IntoIter = IntoIter<'arena, Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<'a, T> IntoIterator for &'a Sequence<'_, T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A sequence of AST nodes separated by infix tokens.
///
/// The token type is generic so every syntax crate can plug in its own
/// token definition. Methods that depend on spatial relationships between
/// nodes and tokens require [`HasPosition`] on the token; that's the
/// narrowest bound that still supports `has_trailing_token` style
/// queries, and every sibling crate's token already carries a [`Position`]
/// start so the impl is trivial.
///
/// [`Position`]: mago_span::Position
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TokenSeparatedSequence<'arena, T, Tok> {
    pub nodes: BVec<'arena, T>,
    pub tokens: BVec<'arena, Tok>,
}

impl<'arena, T, Tok> TokenSeparatedSequence<'arena, T, Tok> {
    #[inline]
    #[must_use]
    pub const fn new(nodes: BVec<'arena, T>, tokens: BVec<'arena, Tok>) -> Self {
        Self { nodes, tokens }
    }

    #[inline]
    pub fn empty(arena: &'arena Bump) -> Self {
        Self { nodes: BVec::new_in(arena), tokens: BVec::new_in(arena) }
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    #[inline]
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.nodes.get(index)
    }

    #[inline]
    #[must_use]
    pub fn first(&self) -> Option<&T> {
        self.nodes.first()
    }

    #[inline]
    #[must_use]
    pub fn last(&self) -> Option<&T> {
        self.nodes.last()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.nodes.iter()
    }

    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        self.nodes.as_slice()
    }

    /// Iterate yielding `(index, node, optional trailing token)` tuples.
    ///
    /// The token is `None` only for the last element if it has no trailing
    /// separator.
    #[inline]
    pub fn iter_with_tokens(&self) -> impl Iterator<Item = (usize, &T, Option<&Tok>)> {
        self.nodes.iter().enumerate().map(move |(i, item)| (i, item, self.tokens.get(i)))
    }
}

impl<'arena, T, Tok> TokenSeparatedSequence<'arena, T, Tok>
where
    T: HasSpan,
    Tok: HasPosition,
{
    /// Whether the sequence ends with a trailing separator token.
    #[inline]
    #[must_use]
    pub fn has_trailing_token(&self) -> bool {
        self.tokens.last().is_some_and(|t| t.offset() >= self.nodes.last().map_or(0, |n| n.span().end.offset))
    }

    /// Return the trailing separator token, if any.
    #[inline]
    #[must_use]
    pub fn get_trailing_token(&self) -> Option<&Tok> {
        self.tokens.last().filter(|t| t.offset() >= self.nodes.last().map_or(0, |n| n.span().end.offset))
    }
}

impl<'arena, T, Tok> IntoIterator for TokenSeparatedSequence<'arena, T, Tok> {
    type Item = T;
    type IntoIter = IntoIter<'arena, Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<'a, T, Tok> IntoIterator for &'a TokenSeparatedSequence<'_, T, Tok> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
