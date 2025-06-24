use serde::Deserialize;
use serde::Serialize;

use mago_interner::ThreadedInterner;

use crate::ttype::TType;

/// Represents PHP's boolean type system, including the general `bool` type
/// and the literal `true` and `false` types.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct TBool {
    pub value: Option<bool>,
}

impl TBool {
    /// Creates a new Bool instance from an optional boolean value.
    pub const fn new(value: Option<bool>) -> Self {
        Self { value }
    }

    /// Creates an instance representing the literal `true` type.
    #[inline]
    pub const fn r#true() -> Self {
        Self { value: Some(true) }
    }

    /// Creates an instance representing the literal `false` type.
    #[inline]
    pub const fn r#false() -> Self {
        Self { value: Some(false) }
    }

    /// Creates an instance representing the general `bool` type.
    #[inline]
    pub const fn general() -> Self {
        Self { value: None }
    }

    /// Checks if this instance represents the literal `true` type.
    #[inline]
    pub const fn is_true(&self) -> bool {
        matches!(self.value, Some(true))
    }

    /// Checks if this instance represents the literal `false` type.
    #[inline]
    pub const fn is_false(&self) -> bool {
        matches!(self.value, Some(false))
    }

    /// Checks if this instance represents the general `bool` type (neither specifically true nor false).
    #[inline]
    pub const fn is_general(&self) -> bool {
        self.value.is_none()
    }
}

impl TType for TBool {
    fn get_id(&self, _interner: Option<&ThreadedInterner>) -> String {
        match self.value {
            Some(true) => "true".to_string(),
            Some(false) => "false".to_string(),
            None => "bool".to_string(),
        }
    }
}

impl Default for TBool {
    fn default() -> Self {
        Self::general()
    }
}

impl From<bool> for TBool {
    fn from(value: bool) -> Self {
        Self::new(Some(value))
    }
}

impl From<Option<bool>> for TBool {
    fn from(value: Option<bool>) -> Self {
        Self::new(value)
    }
}
