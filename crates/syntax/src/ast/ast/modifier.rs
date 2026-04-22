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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn is_read_visibility(&self) -> bool {
        matches!(self, Modifier::Public(..) | Modifier::Protected(..) | Modifier::Private(..))
    }

    /// Returns `true` if the modifier is a write visibility modifier.
    #[must_use]
    pub fn is_write_visibility(&self) -> bool {
        matches!(self, Modifier::PrivateSet(..) | Modifier::ProtectedSet(..) | Modifier::PublicSet(..))
    }

    #[inline]
    #[must_use]
    pub const fn is_static(&self) -> bool {
        matches!(self, Modifier::Static(..))
    }

    #[inline]
    #[must_use]
    pub const fn is_final(&self) -> bool {
        matches!(self, Modifier::Final(..))
    }

    #[inline]
    #[must_use]
    pub const fn is_abstract(&self) -> bool {
        matches!(self, Modifier::Abstract(..))
    }

    #[inline]
    #[must_use]
    pub const fn is_readonly(&self) -> bool {
        matches!(self, Modifier::Readonly(..))
    }

    #[inline]
    #[must_use]
    pub const fn is_public(&self) -> bool {
        matches!(self, Modifier::Public(..))
    }

    #[inline]
    #[must_use]
    pub const fn is_protected(&self) -> bool {
        matches!(self, Modifier::Protected(..))
    }

    #[inline]
    #[must_use]
    pub const fn is_private(&self) -> bool {
        matches!(self, Modifier::Private(..))
    }

    #[inline]
    #[must_use]
    pub const fn is_public_set(&self) -> bool {
        matches!(self, Modifier::PublicSet(..))
    }

    #[inline]
    #[must_use]
    pub const fn is_protected_set(&self) -> bool {
        matches!(self, Modifier::ProtectedSet(..))
    }

    #[inline]
    #[must_use]
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

/// Accessors layered over a modifier [`Sequence`] so that callers can
/// query visibility, staticness, etc. without pattern-matching every
/// variant. Imported automatically by `use crate::ast::*`.
pub trait ModifierSequenceExt<'arena> {
    fn get_static(&self) -> Option<&Modifier<'arena>>;
    fn contains_static(&self) -> bool;
    fn get_final(&self) -> Option<&Modifier<'arena>>;
    fn contains_final(&self) -> bool;
    fn get_abstract(&self) -> Option<&Modifier<'arena>>;
    fn contains_abstract(&self) -> bool;
    fn get_readonly(&self) -> Option<&Modifier<'arena>>;
    fn contains_readonly(&self) -> bool;
    fn get_first_visibility(&self) -> Option<&Modifier<'arena>>;
    fn get_first_read_visibility(&self) -> Option<&Modifier<'arena>>;
    fn get_first_write_visibility(&self) -> Option<&Modifier<'arena>>;
    fn contains_visibility(&self) -> bool;
    fn get_public(&self) -> Option<&Modifier<'arena>>;
    fn contains_public(&self) -> bool;
    fn get_protected(&self) -> Option<&Modifier<'arena>>;
    fn contains_protected(&self) -> bool;
    fn get_private(&self) -> Option<&Modifier<'arena>>;
    fn contains_private(&self) -> bool;
    fn get_private_set(&self) -> Option<&Modifier<'arena>>;
    fn contains_private_set(&self) -> bool;
    fn get_protected_set(&self) -> Option<&Modifier<'arena>>;
    fn contains_protected_set(&self) -> bool;
    fn get_public_set(&self) -> Option<&Modifier<'arena>>;
    fn contains_public_set(&self) -> bool;
}

impl<'arena> ModifierSequenceExt<'arena> for Sequence<'arena, Modifier<'arena>> {
    #[inline]
    fn get_static(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Static(..)))
    }

    #[inline]
    fn contains_static(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Static(..)))
    }

    #[inline]
    fn get_final(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Final(_)))
    }

    #[inline]
    fn contains_final(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Final(..)))
    }

    #[inline]
    fn get_abstract(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Abstract(..)))
    }

    #[inline]
    fn contains_abstract(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Abstract(..)))
    }

    #[inline]
    fn get_readonly(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Readonly(..)))
    }

    #[inline]
    fn contains_readonly(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Readonly(..)))
    }

    #[inline]
    fn get_first_visibility(&self) -> Option<&Modifier<'arena>> {
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

    #[inline]
    fn get_first_read_visibility(&self) -> Option<&Modifier<'arena>> {
        self.iter()
            .find(|modifier| matches!(modifier, Modifier::Public(..) | Modifier::Protected(..) | Modifier::Private(..)))
    }

    #[inline]
    fn get_first_write_visibility(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| {
            matches!(modifier, Modifier::PrivateSet(..) | Modifier::ProtectedSet(..) | Modifier::PublicSet(..))
        })
    }

    #[inline]
    fn contains_visibility(&self) -> bool {
        self.iter().any(Modifier::is_visibility)
    }

    #[inline]
    fn get_public(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Public(..)))
    }

    #[inline]
    fn contains_public(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Public(..)))
    }

    #[inline]
    fn get_protected(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Protected(..)))
    }

    #[inline]
    fn contains_protected(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Protected(..)))
    }

    #[inline]
    fn get_private(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Private(..)))
    }

    #[inline]
    fn contains_private(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Private(..)))
    }

    #[inline]
    fn get_private_set(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::PrivateSet(..)))
    }

    #[inline]
    fn contains_private_set(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::PrivateSet(..)))
    }

    #[inline]
    fn get_protected_set(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::ProtectedSet(..)))
    }

    #[inline]
    fn contains_protected_set(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::ProtectedSet(..)))
    }

    #[inline]
    fn get_public_set(&self) -> Option<&Modifier<'arena>> {
        self.iter().find(|modifier| matches!(modifier, Modifier::PublicSet(..)))
    }

    #[inline]
    fn contains_public_set(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::PublicSet(..)))
    }
}
