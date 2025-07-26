use serde::Deserialize;
use serde::Serialize;

use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::union::TUnion;

/// Represents a key used in PHP arrays, which can be either an integer (`int`) or a string (`string`).
///
/// PHP automatically casts other scalar types (float, bool, null) and resources to int or string
/// when used as array keys. Objects used as keys usually result in errors or use spl_object_hash.
/// This enum focuses on the valid resulting key types after potential casting.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ArrayKey {
    /// An integer array key.
    Integer(i64),
    /// A string array key.
    String(String),
}

impl ArrayKey {
    /// If this key is an `Integer`, returns `Some(i64)`, otherwise `None`.
    #[inline]
    pub const fn get_integer(&self) -> Option<i64> {
        match self {
            ArrayKey::Integer(i) => Some(*i),
            ArrayKey::String(_) => None,
        }
    }

    /// If this key is a `String`, returns `Some(&String)`, otherwise `None`.
    #[inline]
    // Not const because it returns a reference derived from a match on a reference.
    // While theoretically possible in future Rust, currently references from matches prevent const.
    pub fn get_string(&self) -> Option<&String> {
        match self {
            ArrayKey::Integer(_) => None,
            ArrayKey::String(s) => Some(s),
        }
    }

    /// Checks if this array key is an integer (`ArrayKey::Integer`).
    #[inline]
    pub const fn is_integer(&self) -> bool {
        matches!(self, ArrayKey::Integer(_))
    }

    /// Checks if this array key is a string (`ArrayKey::String`).
    #[inline]
    pub const fn is_string(&self) -> bool {
        matches!(self, ArrayKey::String(_))
    }

    /// Converts the array key into a specific literal atomic type representing the key *value*.
    /// Preserves the literal value (e.g., `10`, `"abc"`).
    ///
    /// Note: Clones the string for `ArrayKey::String`.
    #[inline]
    pub fn to_atomic(&self) -> TAtomic {
        match &self {
            ArrayKey::Integer(i) => TAtomic::Scalar(TScalar::Integer(TInteger::literal(*i))),
            ArrayKey::String(s) => TAtomic::Scalar(TScalar::String(s.clone().into())),
        }
    }

    /// Converts the array key into a `TUnion` containing its specific literal atomic type.
    /// Equivalent to `TUnion::new(vec![self.to_atomic()])`.
    #[inline]
    pub fn to_union(&self) -> TUnion {
        TUnion::new(vec![self.to_atomic()])
    }

    /// Converts the array key into a general atomic type representing the key *type* (`int` or `string`).
    /// Does not preserve the specific literal value.
    #[inline]
    pub const fn to_general_atomic(&self) -> TAtomic {
        match self {
            ArrayKey::Integer(_) => TAtomic::Scalar(TScalar::int()),
            ArrayKey::String(_) => TAtomic::Scalar(TScalar::string()),
        }
    }

    /// Converts the array key into a `TUnion` containing its general atomic type (`int` or `string`).
    ///
    /// Equivalent to `TUnion::new(vec![self.to_general_atomic()])`.
    #[inline]
    pub fn to_general_union(&self) -> TUnion {
        TUnion::new(vec![self.to_general_atomic()])
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
        }
    }
}

impl From<i64> for ArrayKey {
    /// Converts an `i64` to an `ArrayKey::Integer`.
    #[inline]
    fn from(i: i64) -> Self {
        ArrayKey::Integer(i)
    }
}

impl From<String> for ArrayKey {
    /// Converts a `String` to an `ArrayKey::String`.
    #[inline]
    fn from(s: String) -> Self {
        ArrayKey::String(s)
    }
}

impl From<&str> for ArrayKey {
    /// Converts a `&str` to an `ArrayKey::String`.
    #[inline]
    fn from(s: &str) -> Self {
        ArrayKey::String(s.to_string())
    }
}
