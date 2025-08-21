use serde::Serialize;
use strum::Display;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::keyword::Keyword;
use crate::ast::sequence::Sequence;

/// Represents a modifier statement.
///
/// # Examples
///
/// ```php
/// final class Foo {}
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
#[repr(u8)]
pub enum Modifier<'arena> {
    Static(Keyword<'arena>),
    Final(Keyword<'arena>),
    Abstract(Keyword<'arena>),
    Readonly(Keyword<'arena>),
    Public(Keyword<'arena>),
    PublicSet(Keyword<'arena>),
    Protected(Keyword<'arena>),
    ProtectedSet(Keyword<'arena>),
    Private(Keyword<'arena>),
    PrivateSet(Keyword<'arena>),
}

impl<'arena> Modifier<'arena> {
    pub fn get_keyword(&self) -> &Keyword<'arena> {
        match self {
            Modifier::Static(k) => k,
            Modifier::Final(k) => k,
            Modifier::Abstract(k) => k,
            Modifier::Readonly(k) => k,
            Modifier::Public(k) => k,
            Modifier::PublicSet(k) => k,
            Modifier::Protected(k) => k,
            Modifier::ProtectedSet(k) => k,
            Modifier::Private(k) => k,
            Modifier::PrivateSet(k) => k,
        }
    }

    /// Returns `true` if the modifier is a visibility modifier.
    pub fn is_visibility(&self) -> bool {
        matches!(
            self,
            Modifier::Public(..)
                | Modifier::Protected(..)
                | Modifier::Private(..)
                | Modifier::PrivateSet(..)
                | Modifier::ProtectedSet(..)
                | Modifier::PublicSet(..)
        )
    }

    /// Returns `true` if the modifier is a read visibility modifier.
    pub fn is_read_visibility(&self) -> bool {
        matches!(self, Modifier::Public(..) | Modifier::Protected(..) | Modifier::Private(..))
    }

    /// Returns `true` if the modifier is a write visibility modifier.
    pub fn is_write_visibility(&self) -> bool {
        matches!(self, Modifier::PrivateSet(..) | Modifier::ProtectedSet(..) | Modifier::PublicSet(..))
    }

    #[inline]
    pub const fn is_static(&self) -> bool {
        matches!(self, Modifier::Static(..))
    }

    #[inline]
    pub const fn is_final(&self) -> bool {
        matches!(self, Modifier::Final(..))
    }

    #[inline]
    pub const fn is_abstract(&self) -> bool {
        matches!(self, Modifier::Abstract(..))
    }

    #[inline]
    pub const fn is_readonly(&self) -> bool {
        matches!(self, Modifier::Readonly(..))
    }

    #[inline]
    pub const fn is_public(&self) -> bool {
        matches!(self, Modifier::Public(..))
    }

    #[inline]
    pub const fn is_protected(&self) -> bool {
        matches!(self, Modifier::Protected(..))
    }

    #[inline]
    pub const fn is_private(&self) -> bool {
        matches!(self, Modifier::Private(..))
    }

    #[inline]
    pub const fn is_public_set(&self) -> bool {
        matches!(self, Modifier::PublicSet(..))
    }

    #[inline]
    pub const fn is_protected_set(&self) -> bool {
        matches!(self, Modifier::ProtectedSet(..))
    }

    #[inline]
    pub const fn is_private_set(&self) -> bool {
        matches!(self, Modifier::PrivateSet(..))
    }
}

impl HasSpan for Modifier<'_> {
    fn span(&self) -> Span {
        match self {
            Modifier::Static(value)
            | Modifier::Final(value)
            | Modifier::Abstract(value)
            | Modifier::Readonly(value)
            | Modifier::Public(value)
            | Modifier::Protected(value)
            | Modifier::Private(value)
            | Modifier::PrivateSet(value)
            | Modifier::ProtectedSet(value)
            | Modifier::PublicSet(value) => value.span(),
        }
    }
}

impl<'arena> Sequence<'arena, Modifier<'arena>> {
    /// Returns the first abstract modifier in the sequence, if any.
    pub fn get_static(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Static(..)))
    }

    /// Returns `true` if the sequence contains a static modifier.
    pub fn contains_static(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Static(..)))
    }

    /// Return the first final modifier in the sequence, if any.
    pub fn get_final(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Final(_)))
    }

    /// Returns `true` if the sequence contains a final modifier.
    pub fn contains_final(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Final(..)))
    }

    /// Returns the first abstract modifier in the sequence, if any.
    pub fn get_abstract(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Abstract(..)))
    }

    /// Returns `true` if the sequence contains an abstract modifier.
    pub fn contains_abstract(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Abstract(..)))
    }

    /// Returns the first abstract modifier in the sequence, if any.
    pub fn get_readonly(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Readonly(..)))
    }

    /// Returns `true` if the sequence contains a readonly modifier.
    pub fn contains_readonly(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Readonly(..)))
    }

    pub fn get_first_visibility(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| {
            matches!(
                modifier,
                Modifier::Public(..)
                    | Modifier::Protected(..)
                    | Modifier::Private(..)
                    | Modifier::PrivateSet(..)
                    | Modifier::ProtectedSet(..)
                    | Modifier::PublicSet(..)
            )
        })
    }

    pub fn get_first_read_visibility(&self) -> Option<&Modifier<'arena>> {
        self.iter()
            .find(|modifier| matches!(modifier, Modifier::Public(..) | Modifier::Protected(..) | Modifier::Private(..)))
    }

    pub fn get_first_write_visibility(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| {
            matches!(modifier, Modifier::PrivateSet(..) | Modifier::ProtectedSet(..) | Modifier::PublicSet(..))
        })
    }

    /// Returns `true` if the sequence contains a visibility modifier for reading or writing.
    pub fn contains_visibility(&self) -> bool {
        self.iter().any(Modifier::is_visibility)
    }

    pub fn get_public(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Public(..)))
    }

    /// Returns `true` if the sequence contains a public visibility modifier.
    pub fn contains_public(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Public(..)))
    }

    pub fn get_protected(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Protected(..)))
    }

    /// Returns `true` if the sequence contains a protected visibility modifier.
    pub fn contains_protected(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Protected(..)))
    }

    pub fn get_private(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Private(..)))
    }

    /// Returns `true` if the sequence contains a private visibility modifier.
    pub fn contains_private(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Private(..)))
    }

    pub fn get_private_set(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::PrivateSet(..)))
    }

    pub fn contains_private_set(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::PrivateSet(..)))
    }

    pub fn get_protected_set(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::ProtectedSet(..)))
    }

    pub fn contains_protected_set(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::ProtectedSet(..)))
    }

    pub fn get_public_set(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::PublicSet(..)))
    }

    pub fn contains_public_set(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::PublicSet(..)))
    }
}
