use serde::Deserialize;
use serde::Serialize;

use mago_interner::ThreadedInterner;

use crate::ttype::TType;
use crate::utils::str_is_numeric;

/// Represents the state of a string known to originate from a literal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)] // Added as requested
pub enum TStringLiteral {
    /// The string originates from a literal, but its specific value isn't tracked here.
    Unspecified,
    /// The string originates from a literal, and its value is known.
    Value(String),
}

/// Represents a PHP string type, tracking literal origin and guaranteed properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord, Hash)]
pub struct TString {
    /// Describes the literal nature, if known. `None` means not known to be literal (general string).
    pub literal: Option<TStringLiteral>,
    /// Is this string *guaranteed* (by analysis or literal value) to be numeric according to PHP rules?
    pub is_numeric: bool,
    /// Is this string *guaranteed* (by analysis or literal value) to be truthy (non-empty and not "0")?
    pub is_truthy: bool,
    /// Is this string *guaranteed* (by analysis or literal value) to be non-empty?
    pub is_non_empty: bool,
}

impl TStringLiteral {
    /// Creates the 'Unspecified' literal state.
    #[inline]
    pub const fn unspecified() -> Self {
        TStringLiteral::Unspecified
    }

    /// Creates the 'Value' literal state with a specific string value.
    #[inline]
    pub const fn value(value: String) -> Self {
        TStringLiteral::Value(value)
    }

    /// Creates the 'Value' literal state from a string slice.
    #[inline]
    pub fn value_from_str(value: &str) -> Self {
        TStringLiteral::Value(value.to_string())
    }

    /// Checks if this represents an unspecified literal value.
    #[inline]
    pub const fn is_unspecified(&self) -> bool {
        matches!(self, TStringLiteral::Unspecified)
    }

    /// Checks if this represents a literal with a known value.
    #[inline]
    pub const fn is_value(&self) -> bool {
        matches!(self, TStringLiteral::Value(_))
    }

    /// Returns the known literal string value, if available.
    #[inline]
    pub fn get_value(&self) -> Option<&str> {
        match self {
            TStringLiteral::Value(s) => Some(s),
            TStringLiteral::Unspecified => None,
        }
    }
}

impl TString {
    /// Creates an instance representing the general `string` type (not known literal, no guaranteed props).
    #[inline]
    pub const fn general() -> Self {
        Self { literal: None, is_numeric: false, is_truthy: false, is_non_empty: false }
    }

    /// Creates a non-empty string instance with no additional properties.
    #[inline]
    pub const fn non_empty() -> Self {
        Self { literal: None, is_numeric: false, is_truthy: false, is_non_empty: true }
    }

    /// Creates a general string instance with explicitly set guaranteed properties (from analysis).
    #[inline]
    pub const fn general_with_props(is_numeric: bool, is_truthy: bool, mut is_non_empty: bool) -> Self {
        is_non_empty |= is_numeric || is_truthy;

        Self { literal: None, is_numeric, is_truthy, is_non_empty }
    }

    /// Creates an instance representing an unspecified literal string (origin known, value unknown).
    /// Assumes no guaranteed properties unless specified otherwise via `_with_props`.
    #[inline]
    pub const fn unspecified_literal() -> Self {
        Self { literal: Some(TStringLiteral::Unspecified), is_numeric: false, is_truthy: false, is_non_empty: false }
    }

    /// Creates an unspecified literal string instance with explicitly set guaranteed properties (from analysis).
    #[inline]
    pub const fn unspecified_literal_with_props(is_numeric: bool, is_truthy: bool, is_non_empty: bool) -> Self {
        Self { literal: Some(TStringLiteral::Unspecified), is_numeric, is_truthy, is_non_empty }
    }

    /// Creates an instance representing a known literal string type (e.g., `"hello"`).
    /// Properties (`is_numeric`, `is_truthy`, `is_non_empty`) are derived from the value.
    #[inline]
    pub fn known_literal(value: String) -> Self {
        let is_numeric = str_is_numeric(&value);
        let is_non_empty = is_numeric || !value.is_empty();
        let is_truthy = is_non_empty && value != "0";

        Self { literal: Some(TStringLiteral::Value(value)), is_numeric, is_truthy, is_non_empty }
    }

    /// Creates an instance representing a known literal string type from a string slice.
    #[inline]
    pub fn known_literal_from_str(value: &str) -> Self {
        Self::known_literal(value.to_string())
    }

    /// Checks if this represents a general `string` (origin not known to be literal).
    #[inline]
    pub const fn is_general(&self) -> bool {
        self.literal.is_none()
    }

    /// Checks if this string is known to originate from a literal (value known or unspecified).
    #[inline]
    pub const fn is_literal_origin(&self) -> bool {
        self.literal.is_some()
    }

    /// Checks if this represents an unspecified literal string (origin known, value unknown).
    #[inline]
    pub const fn is_unspecified_literal(&self) -> bool {
        matches!(self.literal, Some(TStringLiteral::Unspecified))
    }

    /// Checks if this represents a known literal string (origin known, value known).
    #[inline]
    pub const fn is_known_literal(&self) -> bool {
        matches!(self.literal, Some(TStringLiteral::Value(_)))
    }

    /// Checks if this string is guaranteed to be a specific literal value.
    ///
    /// Returns `true` if the string is a known literal and matches the provided value.
    #[inline]
    pub fn is_specific_literal(&self, value: &str) -> bool {
        match &self.literal {
            Some(TStringLiteral::Value(s)) => s == value,
            _ => false,
        }
    }

    /// Returns the known literal string value, if available.
    #[inline]
    pub fn get_known_literal_value(&self) -> Option<&str> {
        match &self.literal {
            Some(TStringLiteral::Value(s)) => Some(s),
            _ => None,
        }
    }

    /// Checks if the string is guaranteed to be numeric.
    #[inline]
    pub const fn is_known_numeric(&self) -> bool {
        self.is_numeric
    }

    /// Checks if the string is guaranteed to be truthy (non-empty and not "0").
    #[inline]
    pub const fn is_truthy(&self) -> bool {
        self.is_truthy
    }

    /// Checks if the string is guaranteed to be non-empty.
    #[inline]
    pub const fn is_non_empty(&self) -> bool {
        self.is_non_empty
    }

    /// Checks if the string is guaranteed to be boring (no interesting properties).
    #[inline]
    pub const fn is_boring(&self) -> bool {
        match &self.literal {
            Some(_) => false,
            _ => !self.is_numeric && !self.is_truthy && !self.is_non_empty,
        }
    }

    /// Returns the literal state (`Unspecified` or `Value(...)`) if the origin is literal.
    #[inline]
    pub const fn literal_state(&self) -> Option<&TStringLiteral> {
        self.literal.as_ref()
    }

    // Returns a new instance with the same properties but without the literal value.
    #[inline]
    pub fn without_literal(&self) -> Self {
        Self { literal: None, is_numeric: self.is_numeric, is_truthy: self.is_truthy, is_non_empty: self.is_non_empty }
    }

    /// Returns a new instance with the same properties but with the literal value set to `Unspecified`.
    #[inline]
    pub fn with_unspecified_literal(&self) -> Self {
        Self {
            literal: Some(TStringLiteral::Unspecified),
            is_numeric: self.is_numeric,
            is_truthy: self.is_truthy,
            is_non_empty: self.is_non_empty,
        }
    }

    pub fn as_numeric(&self, retain_literal: bool) -> Self {
        Self {
            literal: if retain_literal { self.literal.clone() } else { None },
            is_numeric: true,
            is_truthy: self.is_truthy,
            is_non_empty: true,
        }
    }
}

impl TType for TString {
    fn get_id(&self, _interner: Option<&ThreadedInterner>) -> String {
        let s = match &self.literal {
            Some(TStringLiteral::Value(s)) => return format!("string('{}')", s.replace('\'', "\\'")),
            Some(_) => {
                if self.is_truthy {
                    if self.is_numeric { "truthy-numeric-literal-string" } else { "truthy-literal-string" }
                } else if self.is_numeric {
                    "numeric-literal-string"
                } else if self.is_non_empty {
                    "non-empty-literal-string"
                } else {
                    "literal-string"
                }
            }
            None => {
                if self.is_truthy {
                    if self.is_numeric { "truthy-numeric-string" } else { "truthy-string" }
                } else if self.is_numeric {
                    "numeric-string"
                } else if self.is_non_empty {
                    "non-empty-string"
                } else {
                    "string"
                }
            }
        };

        s.to_string()
    }
}

impl Default for TStringLiteral {
    /// Defaults to `Unspecified`.
    fn default() -> Self {
        TStringLiteral::Unspecified
    }
}

impl Default for TString {
    /// Defaults to a general string with no guaranteed properties.
    fn default() -> Self {
        Self::general()
    }
}

impl From<&str> for TString {
    /// Converts a string slice into a `known_literal` StringScalar.
    /// Derives properties from the literal value.
    fn from(value: &str) -> Self {
        Self::known_literal_from_str(value)
    }
}

impl From<String> for TString {
    /// Converts a String into a `known_literal` StringScalar.
    /// Derives properties from the literal value.
    fn from(value: String) -> Self {
        Self::known_literal(value)
    }
}
