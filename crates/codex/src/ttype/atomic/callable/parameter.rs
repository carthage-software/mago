use std::cmp::Ordering;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;

use crate::misc::VariableIdentifier;
use crate::ttype::union::TUnion;

/// Represents metadata for a single parameter within a `callable` type signature.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TCallableParameter {
    /// The original parameter name, when the signature comes from a declared function-like.
    /// This is needed by parameter-dependent types such as `T[$key]` when the callable is
    /// invoked indirectly.
    name: Option<VariableIdentifier>,
    /// The type hint for the parameter, if specified within the callable signature.
    /// `None` if no specific type is given (equivalent to `mixed`).
    type_signature: Option<Arc<TUnion>>,
    /// `true` if the parameter expects an argument passed by reference (signified by `&`).
    is_by_reference: bool,
    /// `true` if this parameter is variadic (`...`).
    is_variadic: bool,
    /// `true` if this parameter is optional (signified by `=`).
    has_default: bool,
}

// Parameter names are invocation metadata, not part of a callable's structural type.
// Keeping them out of equality and ordering preserves compatibility between an anonymous
// PHPDoc callable and the equivalent declared function signature.
impl PartialEq for TCallableParameter {
    fn eq(&self, other: &Self) -> bool {
        self.type_signature == other.type_signature
            && self.is_by_reference == other.is_by_reference
            && self.is_variadic == other.is_variadic
            && self.has_default == other.has_default
    }
}

impl Eq for TCallableParameter {}

impl Hash for TCallableParameter {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.type_signature.hash(state);
        self.is_by_reference.hash(state);
        self.is_variadic.hash(state);
        self.has_default.hash(state);
    }
}

impl PartialOrd for TCallableParameter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TCallableParameter {
    fn cmp(&self, other: &Self) -> Ordering {
        self.type_signature
            .cmp(&other.type_signature)
            .then_with(|| self.is_by_reference.cmp(&other.is_by_reference))
            .then_with(|| self.is_variadic.cmp(&other.is_variadic))
            .then_with(|| self.has_default.cmp(&other.has_default))
    }
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
    #[must_use]
    pub fn new(
        type_signature: Option<Arc<TUnion>>,
        is_by_reference: bool,
        is_variadic: bool,
        has_default: bool,
    ) -> Self {
        Self { name: None, type_signature, is_by_reference, is_variadic, has_default }
    }

    /// Returns the parameter name, if the callable signature retained one.
    #[inline]
    #[must_use]
    pub const fn get_name(&self) -> Option<&VariableIdentifier> {
        self.name.as_ref()
    }

    /// Returns a new parameter with its original name retained.
    #[inline]
    #[must_use]
    pub fn with_name(mut self, name: Option<VariableIdentifier>) -> Self {
        self.name = name;
        self
    }

    /// Returns a new parameter with the given type signature.
    #[inline]
    #[must_use]
    pub fn with_type_signature(mut self, type_signature: Option<Arc<TUnion>>) -> Self {
        self.type_signature = type_signature;
        self
    }

    /// Returns a reference to the parameter's type signature (`TUnion`), if specified.
    #[inline]
    #[must_use]
    pub fn get_type_signature(&self) -> Option<&TUnion> {
        self.type_signature.as_deref()
    }

    /// Returns a mutable reference to the parameter's type signature (`TUnion`), if specified.
    pub fn get_type_signature_mut(&mut self) -> Option<&mut TUnion> {
        self.type_signature.as_mut().map(Arc::make_mut)
    }

    /// Checks if the parameter expects an argument passed by reference (`&`).
    #[inline]
    #[must_use]
    pub const fn is_by_reference(&self) -> bool {
        self.is_by_reference
    }

    /// Checks if the parameter is variadic (`...`).
    #[inline]
    #[must_use]
    pub const fn is_variadic(&self) -> bool {
        self.is_variadic
    }

    /// Checks if the parameter is has a default value (`=`).
    #[inline]
    #[must_use]
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
