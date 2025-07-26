use serde::Deserialize;
use serde::Serialize;

use crate::ttype::union::TUnion;

/// Represents metadata for a single parameter within a `callable` type signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct TCallableParameter {
    /// The type hint for the parameter, if specified within the callable signature.
    /// `None` if no specific type is given (equivalent to `mixed`).
    type_signature: Option<Box<TUnion>>,
    /// `true` if the parameter expects an argument passed by reference (signified by `&`).
    is_by_reference: bool,
    /// `true` if this parameter is variadic (`...`).
    is_variadic: bool,
    /// `true` if this parameter is optional (signified by `=`).
    has_default: bool,
}

impl TCallableParameter {
    /// Creates a new `CallableParameter` specifying all properties directly.
    ///
    /// # Arguments
    ///
    /// * `type_signature`: The optional type hint for the parameter (`None` for `mixed`).
    /// * `is_by_reference`: Whether the parameter expects pass-by-reference (`&`).
    /// * `is_variadic`: Whether the parameter is variadic (`...`).
    /// * `has_default`: Whether the parameter is optional (`=`).
    #[inline]
    pub const fn new(
        type_signature: Option<Box<TUnion>>,
        is_by_reference: bool,
        is_variadic: bool,
        has_default: bool,
    ) -> Self {
        Self { type_signature, is_by_reference, is_variadic, has_default }
    }

    /// Returns a reference to the parameter's type signature (`TUnion`), if specified.
    #[inline]
    pub fn get_type_signature(&self) -> Option<&TUnion> {
        self.type_signature.as_deref()
    }

    /// Returns a mutable reference to the parameter's type signature (`TUnion`), if specified.
    pub fn get_type_signature_mut(&mut self) -> Option<&mut TUnion> {
        self.type_signature.as_deref_mut()
    }

    /// Checks if the parameter expects an argument passed by reference (`&`).
    #[inline]
    pub const fn is_by_reference(&self) -> bool {
        self.is_by_reference
    }

    /// Checks if the parameter is variadic (`...`).
    #[inline]
    pub const fn is_variadic(&self) -> bool {
        self.is_variadic
    }

    /// Checks if the parameter is has a default value (`=`).
    #[inline]
    pub const fn has_default(&self) -> bool {
        self.has_default
    }
}

/// Provides a default `CallableParameter` representing a non-optional, non-variadic,
/// non-reference parameter with no specific type (effectively `mixed`).
impl Default for TCallableParameter {
    #[inline]
    fn default() -> Self {
        Self::new(None, false, false, false)
    }
}
