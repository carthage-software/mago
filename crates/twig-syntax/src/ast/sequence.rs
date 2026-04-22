use std::slice::Iter;

use bumpalo::Bump;
use bumpalo::collections::Vec as BVec;
use bumpalo::collections::vec::IntoIter;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::token::TwigToken;

/// A sequence of AST nodes allocated in a `bumpalo::Bump`.
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
}

/// A sequence of nodes separated by infix tokens (commas, semicolons, ...).
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct TokenSeparatedSequence<'arena, T> {
    pub nodes: BVec<'arena, T>,
    pub tokens: BVec<'arena, TwigToken<'arena>>,
}

impl<'arena, T> TokenSeparatedSequence<'arena, T> {
    #[inline]
    #[must_use]
    pub const fn new(nodes: BVec<'arena, T>, tokens: BVec<'arena, TwigToken<'arena>>) -> Self {
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
    pub fn iter(&self) -> Iter<'_, T> {
        self.nodes.iter()
    }

    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        self.nodes.as_slice()
    }

    /// Whether the sequence ends with a trailing separator token.
    #[inline]
    #[must_use]
    pub fn has_trailing_token(&self) -> bool
    where
        T: HasSpan,
    {
        self.tokens.last().is_some_and(|t| t.start.offset >= self.nodes.last().map_or(0, |n| n.span().end.offset))
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
