use std::borrow::Cow;

use mago_atom::Atom;
use mago_atom::concat_atom;
use mago_atom::i64_atom;
use serde::Deserialize;
use serde::Serialize;

use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::atomic::scalar::string::TString;
use crate::ttype::get_arraykey;
use crate::ttype::get_int;
use crate::ttype::get_string;
use crate::ttype::shared::ARRAYKEY_ATOMIC;
use crate::ttype::shared::INT_ATOMIC;
use crate::ttype::shared::STRING_ATOMIC;
use crate::ttype::union::TUnion;

/// Represents a key used in PHP arrays, which can be either an integer (`int`) or a string (`string`).
///
/// PHP automatically casts other scalar types (float, bool, null) and resources to int or string
/// when used as array keys. Objects used as keys usually result in errors or use `spl_object_hash`.
/// This enum focuses on the valid resulting key types after potential casting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ArrayKey {
    /// An integer array key.
    Integer(i64),
    /// A string array key.
    String(Atom),
    /// A class-like constant or enum case key, not yet resolved to its concrete value.
    ///
    /// This is used when a docblock specifies `array{Foo::BAR: string}` where
    /// `Foo::BAR` is a class constant or enum case. The key will be resolved
    /// to its concrete `Integer` or `String` value during type expansion.
    ClassLikeConstant { class_like_name: Atom, constant_name: Atom },
}

impl ArrayKey {
    /// If this key is an `Integer`, returns `Some(i64)`, otherwise `None`.
    #[inline]
    #[must_use]
    pub const fn get_integer(&self) -> Option<i64> {
        match self {
            ArrayKey::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// If this key is a `String`, returns `Some(&String)`, otherwise `None`.
    #[inline]
    // Not const because it returns a reference derived from a match on a reference.
    // While theoretically possible in future Rust, currently references from matches prevent const.
    #[must_use]
    pub fn get_string(&self) -> Option<&str> {
        match self {
            ArrayKey::String(s) => Some(s),
            _ => None,
        }
    }

    /// Checks if this array key is an integer (`ArrayKey::Integer`).
    #[inline]
    #[must_use]
    pub const fn is_integer(&self) -> bool {
        matches!(self, ArrayKey::Integer(_))
    }

    /// Checks if this array key is a string (`ArrayKey::String`).
    #[inline]
    #[must_use]
    pub const fn is_string(&self) -> bool {
        matches!(self, ArrayKey::String(_))
    }

    /// Checks if this array key is an unresolved class-like constant.
    #[inline]
    #[must_use]
    pub const fn is_class_like_constant(&self) -> bool {
        matches!(self, ArrayKey::ClassLikeConstant { .. })
    }

    /// Converts the array key into an `Atom` representing the key *value*.
    /// Preserves the literal value (e.g., `10`, `"abc"`).
    #[inline]
    #[must_use]
    pub fn to_atom(&self) -> Atom {
        match self {
            ArrayKey::Integer(i) => i64_atom(*i),
            ArrayKey::String(s) => *s,
            ArrayKey::ClassLikeConstant { class_like_name, constant_name } => {
                concat_atom!(class_like_name, "::", constant_name)
            }
        }
    }

    /// Converts the array key into a specific literal atomic type representing the key *value*.
    /// Preserves the literal value (e.g., `10`, `"abc"`).
    #[inline]
    #[must_use]
    pub fn to_atomic(&self) -> TAtomic {
        match &self {
            ArrayKey::Integer(i) => TAtomic::Scalar(TScalar::Integer(TInteger::literal(*i))),
            ArrayKey::String(s) => TAtomic::Scalar(TScalar::String(TString::known_literal(*s))),
            ArrayKey::ClassLikeConstant { .. } => TAtomic::Scalar(TScalar::ArrayKey),
        }
    }

    /// Converts the array key into a `TUnion` containing its specific literal atomic type.
    #[inline]
    #[must_use]
    pub fn to_union(&self) -> TUnion {
        TUnion::from_single(Cow::Owned(self.to_atomic()))
    }

    /// Converts the array key into a general atomic type representing the key *type* (`int` or `string`).
    /// Does not preserve the specific literal value.
    #[inline]
    #[must_use]
    pub const fn to_general_atomic(&self) -> &'static TAtomic {
        match self {
            ArrayKey::Integer(_) => INT_ATOMIC,
            ArrayKey::String(_) => STRING_ATOMIC,
            ArrayKey::ClassLikeConstant { .. } => ARRAYKEY_ATOMIC,
        }
    }

    /// Converts the array key into a `TUnion` containing its general atomic type (`int` or `string`).
    #[inline]
    #[must_use]
    pub fn to_general_union(&self) -> TUnion {
        match self {
            ArrayKey::Integer(_) => get_int(),
            ArrayKey::String(_) => get_string(),
            ArrayKey::ClassLikeConstant { .. } => get_arraykey(),
        }
    }
}

impl std::fmt::Display for ArrayKey {
    /// Converts the array key to a `String` for display purposes.
    /// String keys are enclosed in single quotes.
    ///
    /// Example: `ArrayKey::Integer(10)` becomes `"10"`.
    /// Example: `ArrayKey::String("a".to_string())` becomes `"'a'"`.
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArrayKey::Integer(i) => write!(f, "{i}"),
            ArrayKey::String(k) => write!(f, "'{k}'"),
            ArrayKey::ClassLikeConstant { class_like_name, constant_name } => {
                write!(f, "{class_like_name}::{constant_name}")
            }
        }
    }
}

impl<T> From<T> for ArrayKey
where
    T: AsRef<str>,
{
    /// Converts any type that can be referenced as a `str` to an `ArrayKey::String`.
    /// The string is cloned into a `Atom`.
    #[inline]
    fn from(s: T) -> Self {
        ArrayKey::String(Atom::from(s.as_ref()))
    }
}
