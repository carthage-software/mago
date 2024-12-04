use std::slice::Iter;
use std::vec::IntoIter;

use serde::Deserialize;
use serde::Serialize;

use fennec_span::HasSpan;
use fennec_span::Position;
use fennec_span::Span;
use fennec_token::Token;

/// Represents a sequence of nodes.
///
/// An example of this is modifiers in a method declaration.
///
/// i.e. `public` and `static` in `public static function foo() {}`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Sequence<T> {
    pub(crate) inner: Vec<T>,
}

/// Represents a sequence of nodes separated by a token.
///
/// An example of this is arguments in a function call, where the tokens are commas.
///
/// i.e. `1`, `2` and `3` in `foo(1, 2, 3)`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TokenSeparatedSequence<T> {
    pub(crate) inner: Vec<T>,
    pub tokens: Vec<Token>,
}

impl<T: HasSpan> Sequence<T> {
    pub fn new(inner: Vec<T>) -> Self {
        Self { inner }
    }

    pub fn empty() -> Self {
        Self { inner: vec![] }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    pub fn first(&self) -> Option<&T> {
        self.inner.first()
    }

    pub fn first_span(&self) -> Option<Span> {
        self.inner.first().map(|node| node.span())
    }

    pub fn last(&self) -> Option<&T> {
        self.inner.last()
    }

    pub fn last_span(&self) -> Option<Span> {
        self.inner.last().map(|node| node.span())
    }

    pub fn span(&self, from: Position) -> Span {
        self.last_span().map_or(Span::new(from, from), |span| Span::new(from, span.end))
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.inner.iter()
    }

    pub fn as_slice(&self) -> &[T] {
        self.inner.as_slice()
    }
}

impl<T: HasSpan> TokenSeparatedSequence<T> {
    pub fn new(inner: Vec<T>, tokens: Vec<Token>) -> Self {
        Self { inner, tokens }
    }

    pub fn empty() -> Self {
        Self { inner: vec![], tokens: vec![] }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.inner.get(index)
    }

    #[inline]
    #[must_use]
    pub fn first(&self) -> Option<&T> {
        self.inner.first()
    }

    pub fn first_span(&self) -> Option<Span> {
        match (self.tokens.first(), self.inner.first()) {
            (Some(token), Some(node)) => {
                // check if the token comes before the node
                if token.span.end <= node.span().start {
                    Some(token.span)
                } else {
                    Some(node.span())
                }
            }
            (Some(token), None) => Some(token.span),
            (None, Some(node)) => Some(node.span()),
            (None, None) => None,
        }
    }

    pub fn last(&self) -> Option<&T> {
        self.inner.last()
    }

    pub fn last_span(&self) -> Option<Span> {
        match (self.tokens.last(), self.inner.last()) {
            (Some(token), Some(node)) => {
                // check if the token comes after the node
                if token.span.start >= node.span().end {
                    Some(token.span)
                } else {
                    Some(node.span())
                }
            }
            (Some(token), None) => Some(token.span),
            (None, Some(node)) => Some(node.span()),
            (None, None) => None,
        }
    }

    pub fn span(&self, from: Position) -> Span {
        self.last_span().map_or(Span::new(from, from), |span| Span::new(from, span.end))
    }

    pub fn has_trailing_token(&self) -> bool {
        self.tokens.last().map_or(false, |token| token.span.start >= self.last_span().unwrap().end)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.inner.iter()
    }

    /// Returns an iterator over the sequence, where each item includes
    /// the index of the element, the element and the token following it.
    /// The token is `None` only for the last element if it has no trailing token.
    pub fn iter_with_tokens(&self) -> impl Iterator<Item = (usize, &T, Option<&Token>)> {
        self.inner.iter().enumerate().map(move |(i, item)| {
            let token = self.tokens.get(i);

            (i, item, token)
        })
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.inner.as_slice()
    }
}

impl<T: HasSpan> FromIterator<T> for Sequence<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self { inner: iter.into_iter().collect() }
    }
}

impl<T: HasSpan> IntoIterator for Sequence<T> {
    type Item = T;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<T: HasSpan> IntoIterator for TokenSeparatedSequence<T> {
    type Item = T;
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<T: HasSpan> std::default::Default for Sequence<T> {
    fn default() -> Self {
        Sequence::new(Default::default())
    }
}

impl<T: HasSpan> std::default::Default for TokenSeparatedSequence<T> {
    fn default() -> Self {
        TokenSeparatedSequence::new(Default::default(), Default::default())
    }
}
