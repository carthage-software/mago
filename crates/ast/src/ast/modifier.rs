use serde::Deserialize;
use serde::Serialize;
use strum::Display;

use fennec_span::HasSpan;
use fennec_span::Span;

use crate::ast::keyword::Keyword;
use crate::sequence::Sequence;

/// Represents a modifier statement.
///
/// # Examples
///
/// ```php
/// final class Foo {}
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord, Display)]
#[serde(tag = "type", content = "value")]
pub enum Modifier {
    Static(Keyword),
    Final(Keyword),
    Abstract(Keyword),
    Readonly(Keyword),
    Public(Keyword),
    Protected(Keyword),
    Private(Keyword),
}

impl Modifier {
    pub fn keyword(&self) -> &Keyword {
        match &self {
            Modifier::Static(keyword) => keyword,
            Modifier::Final(keyword) => keyword,
            Modifier::Abstract(keyword) => keyword,
            Modifier::Readonly(keyword) => keyword,
            Modifier::Public(keyword) => keyword,
            Modifier::Protected(keyword) => keyword,
            Modifier::Private(keyword) => keyword,
        }
    }

    /// Returns `true` if the modifier is a visibility modifier.
    pub fn is_visibility(&self) -> bool {
        match self {
            Modifier::Public { .. } | Modifier::Protected { .. } | Modifier::Private { .. } => true,
            _ => false,
        }
    }
}

impl HasSpan for Modifier {
    fn span(&self) -> Span {
        match self {
            Modifier::Static(value)
            | Modifier::Final(value)
            | Modifier::Abstract(value)
            | Modifier::Readonly(value)
            | Modifier::Public(value)
            | Modifier::Protected(value)
            | Modifier::Private(value) => value.span(),
        }
    }
}

impl Sequence<Modifier> {
    /// Returns the first abstract modifier in the sequence, if any.
    pub fn get_static(&self) -> Option<&Modifier> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Static(..)))
    }

    /// Returns `true` if the sequence contains a static modifier.
    pub fn contains_static(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Static(..)))
    }

    /// Return the first final modifier in the sequence, if any.
    pub fn get_final(&self) -> Option<&Modifier> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Final(_)))
    }

    /// Returns `true` if the sequence contains a final modifier.
    pub fn contains_final(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Final(..)))
    }

    /// Returns the first abstract modifier in the sequence, if any.
    pub fn get_abstract(&self) -> Option<&Modifier> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Abstract(..)))
    }

    /// Returns `true` if the sequence contains an abstract modifier.
    pub fn contains_abstract(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Abstract(..)))
    }

    /// Returns the first abstract modifier in the sequence, if any.
    pub fn get_readonly(&self) -> Option<&Modifier> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Readonly(..)))
    }

    /// Returns `true` if the sequence contains a readonly modifier.
    pub fn contains_readonly(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Readonly(..)))
    }

    pub fn get_first_visibility(&self) -> Option<&Modifier> {
        self.iter()
            .find(|modifier| matches!(modifier, Modifier::Public(..) | Modifier::Protected(..) | Modifier::Private(..)))
    }

    /// Returns `true` if the sequence contains a visibility modifier.
    pub fn contains_visibility(&self) -> bool {
        self.iter().any(Modifier::is_visibility)
    }

    pub fn get_public(&self) -> Option<&Modifier> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Public(..)))
    }

    /// Returns `true` if the sequence contains a public visibility modifier.
    pub fn contains_public(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Public(..)))
    }

    pub fn get_protected(&self) -> Option<&Modifier> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Protected(..)))
    }

    /// Returns `true` if the sequence contains a protected visibility modifier.
    pub fn contains_protected(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Protected(..)))
    }

    pub fn get_private(&self) -> Option<&Modifier> {
        self.iter().find(|modifier| matches!(modifier, Modifier::Private(..)))
    }

    /// Returns `true` if the sequence contains a private visibility modifier.
    pub fn contains_private(&self) -> bool {
        self.iter().any(|modifier| matches!(modifier, Modifier::Private(..)))
    }
}
