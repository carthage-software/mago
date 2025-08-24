use ordered_float::OrderedFloat;
use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;
use mago_atom::f64_atom;

use crate::ttype::TType;

/// Represents PHP float types: general `float` or a specific literal like `12.3`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TFloat {
    /// `None` for general `float`, `Some(value)` for a literal float.
    pub value: Option<OrderedFloat<f64>>,
}

impl TFloat {
    /// Creates a new FloatScalar from an optional float value.
    #[inline]
    pub fn new(value: Option<f64>) -> Self {
        Self { value: value.map(OrderedFloat::from) }
    }

    /// Creates an instance representing the general `float` type.
    #[inline]
    pub const fn general() -> Self {
        Self { value: None }
    }

    /// Creates an instance representing a literal float type (e.g., `12.3`).
    #[inline]
    pub fn literal(value: f64) -> Self {
        Self { value: Some(OrderedFloat::from(value)) }
    }

    /// Checks if this represents the general `float` type.
    #[inline]
    pub const fn is_general(&self) -> bool {
        self.value.is_none()
    }

    /// Checks if this represents a literal float type.
    #[inline]
    pub const fn is_literal(&self) -> bool {
        self.value.is_some()
    }

    /// Returns the literal float value if this represents one.
    #[inline]
    pub fn get_literal_value(&self) -> Option<f64> {
        self.value.map(|v| v.into_inner())
    }
}

impl Default for TFloat {
    /// Returns the default value, representing the general `float` type.
    fn default() -> Self {
        Self::general()
    }
}

impl From<f64> for TFloat {
    /// Creates a new FloatScalar from a float value.
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

    fn get_id(&self) -> Atom {
        match self.value {
            Some(value) => concat_atom!("float(", f64_atom(*value), ")"),
            None => atom("float"),
        }
    }
}
