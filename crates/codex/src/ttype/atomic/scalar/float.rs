use ordered_float::OrderedFloat;
use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;
use mago_atom::f64_atom;

use crate::ttype::TType;

/// Represents PHP float types: general `float`, an unspecified literal `literal-float`,
/// or a specific literal like `12.3`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord, Hash)]
pub enum TFloat {
    /// General `float` type.
    Float,
    /// A literal float where the value is unknown (`literal-float`).
    UnspecifiedLiteral,
    /// A specific literal float (e.g., `12.3`).
    Literal(OrderedFloat<f64>),
}

impl TFloat {
    /// Creates a new `FloatScalar` from an optional float value.
    #[inline]
    #[must_use]
    pub fn new(value: Option<f64>) -> Self {
        match value {
            Some(v) => Self::Literal(OrderedFloat::from(v)),
            None => Self::Float,
        }
    }

    /// Creates an instance representing the general `float` type.
    #[inline]
    #[must_use]
    pub const fn general() -> Self {
        Self::Float
    }

    /// Creates an instance representing the `literal-float` type (unspecified literal).
    #[inline]
    #[must_use]
    pub const fn unspecified_literal() -> Self {
        Self::UnspecifiedLiteral
    }

    /// Creates an instance representing a literal float type (e.g., `12.3`).
    #[inline]
    #[must_use]
    pub fn literal(value: f64) -> Self {
        Self::Literal(OrderedFloat::from(value))
    }

    /// Checks if this represents the general `float` type.
    #[inline]
    #[must_use]
    pub const fn is_general(&self) -> bool {
        matches!(self, Self::Float)
    }

    /// Checks if this represents an unspecified literal (`literal-float`).
    #[inline]
    #[must_use]
    pub const fn is_unspecified_literal(&self) -> bool {
        matches!(self, Self::UnspecifiedLiteral)
    }

    /// Checks if this represents a specific literal float type.
    #[inline]
    #[must_use]
    pub const fn is_literal(&self) -> bool {
        matches!(self, Self::Literal(_))
    }

    /// Checks if this originates from any kind of literal (specific or unspecified).
    #[inline]
    #[must_use]
    pub const fn is_literal_origin(&self) -> bool {
        matches!(self, Self::Literal(_) | Self::UnspecifiedLiteral)
    }

    /// Returns the literal float value if this represents a specific literal.
    #[inline]
    #[must_use]
    pub fn get_literal_value(&self) -> Option<f64> {
        match self {
            Self::Literal(v) => Some(v.into_inner()),
            _ => None,
        }
    }

    /// Checks if this float type is contained by another float type.
    ///
    /// Type hierarchy: Literal(v) ⊂ `UnspecifiedLiteral` ⊂ Float
    #[inline]
    #[must_use]
    pub fn contains(&self, other: TFloat) -> bool {
        match (self, other) {
            // Float contains everything
            (Self::Float, _) => true,
            // UnspecifiedLiteral contains itself and specific literals
            (Self::UnspecifiedLiteral, Self::UnspecifiedLiteral | Self::Literal(_)) => true,
            // Literal only contains itself with the same value
            (Self::Literal(v1), Self::Literal(v2)) => *v1 == v2,
            _ => false,
        }
    }
}

impl Default for TFloat {
    /// Returns the default value, representing the general `float` type.
    fn default() -> Self {
        Self::general()
    }
}

impl From<f64> for TFloat {
    /// Creates a new `FloatScalar` from a float value.
    fn from(value: f64) -> Self {
        Self::literal(value)
    }
}

impl TType for TFloat {
    fn needs_population(&self) -> bool {
        false
    }

    fn is_expandable(&self) -> bool {
        false
    }

    fn is_complex(&self) -> bool {
        false
    }

    fn get_id(&self) -> Atom {
        match self {
            Self::Float => atom("float"),
            Self::UnspecifiedLiteral => atom("literal-float"),
            Self::Literal(value) => concat_atom!("float(", f64_atom(**value), ")"),
        }
    }

    fn get_pretty_id_with_indent(&self, _indent: usize) -> Atom {
        self.get_id()
    }
}
