use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;
use mago_atom::concat_atom;

use crate::ttype::TType;
use crate::utils::str_is_numeric;

/// Represents the state of a string known to originate from a literal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum TStringLiteral {
    /// The string originates from a literal, but its specific value isn't tracked here.
    Unspecified,
    /// The string originates from a literal, and its value is known.
    Value(Atom),
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
    /// Is this string guaranteed to be lowercase (e.g., from a literal like "hello")?
    pub is_lowercase: bool,
}

impl TStringLiteral {
    /// Creates the 'Unspecified' literal state.
    #[inline]
    pub const fn unspecified() -> Self {
        TStringLiteral::Unspecified
    }

    /// Creates the 'Value' literal state with a specific string value.
    #[inline]
    pub fn value(value: String) -> Self {
        TStringLiteral::Value(atom(value.as_str()))
    }

    /// Creates the 'Value' literal state from a static string slice.
    #[inline]
    pub fn value_from_static_str(value: &'static str) -> Self {
        TStringLiteral::Value(atom(value))
    }

    /// Creates the 'Value' literal state from a string slice.
    #[inline]
    pub fn value_from_str(value: &str) -> Self {
        TStringLiteral::Value(atom(value))
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
    /// Creates a new instance of `TString` with the specified properties.
    pub const fn new(
        literal: Option<TStringLiteral>,
        is_numeric: bool,
        is_truthy: bool,
        mut is_non_empty: bool,
        mut is_lowercase: bool,
    ) -> Self {
        is_non_empty |= is_numeric || is_truthy;
        is_lowercase |= is_numeric;

        Self { literal, is_numeric, is_truthy, is_non_empty, is_lowercase }
    }

    /// Creates an instance representing the general `string` type (not known literal, no guaranteed props).
    #[inline]
    pub const fn general() -> Self {
        Self::new(None, false, false, false, false)
    }

    /// Creates a non-empty string instance with no additional properties.
    #[inline]
    pub const fn non_empty() -> Self {
        Self::new(None, false, false, true, false)
    }

    /// Creates a numeric string instance.
    #[inline]
    pub const fn numeric() -> Self {
        Self::new(None, true, false, true, false)
    }

    /// Creates a lowercase string instance.
    #[inline]
    pub const fn lowercase() -> Self {
        Self::new(None, false, false, false, true)
    }

    /// Creates a non-empty lowercase string instance.
    #[inline]
    pub const fn non_empty_lowercase() -> Self {
        Self::new(None, false, false, true, true)
    }

    /// Creates a truthy string instance.
    #[inline]
    pub const fn truthy() -> Self {
        Self::new(None, false, true, true, false)
    }

    /// Creates a general string instance with explicitly set guaranteed properties (from analysis).
    #[inline]
    pub const fn general_with_props(is_numeric: bool, is_truthy: bool, is_non_empty: bool, is_lowercase: bool) -> Self {
        Self::new(None, is_numeric, is_truthy, is_non_empty, is_lowercase)
    }

    /// Creates an instance representing an unspecified literal string (origin known, value unknown).
    /// Assumes no guaranteed properties unless specified otherwise via `_with_props`.
    #[inline]
    pub const fn unspecified_literal(non_empty: bool) -> Self {
        Self::new(Some(TStringLiteral::Unspecified), false, false, non_empty, false)
    }

    /// Creates an unspecified literal string instance with explicitly set guaranteed properties (from analysis).
    #[inline]
    pub const fn unspecified_literal_with_props(
        is_numeric: bool,
        is_truthy: bool,
        is_non_empty: bool,
        is_lowercase: bool,
    ) -> Self {
        Self::new(Some(TStringLiteral::Unspecified), is_numeric, is_truthy, is_non_empty, is_lowercase)
    }

    /// Creates an instance representing a known literal string type (e.g., `"hello"`).
    /// Properties (`is_numeric`, `is_truthy`, `is_non_empty`) are derived from the value.
    #[inline]
    pub fn known_literal(value: Atom) -> Self {
        let is_numeric = str_is_numeric(&value);
        let is_non_empty = is_numeric || !value.is_empty();
        let is_truthy = is_non_empty && value.as_str() != "0";
        let is_lowercase = value.chars().all(|c| !c.is_uppercase());

        Self::new(Some(TStringLiteral::Value(value)), is_numeric, is_truthy, is_non_empty, is_lowercase)
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

    /// Checks if the string is guaranteed to be lowercase (e.g., from a literal like "hello").
    #[inline]
    pub const fn is_lowercase(&self) -> bool {
        self.is_lowercase
    }

    /// Checks if the string is guaranteed to be boring (no interesting properties).
    #[inline]
    pub const fn is_boring(&self) -> bool {
        match &self.literal {
            Some(_) => false,
            _ => !self.is_numeric && !self.is_truthy && !self.is_non_empty && !self.is_lowercase,
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
        Self { literal: None, ..*self }
    }

    /// Returns a new instance with the same properties but with the literal value set to `Unspecified`.
    #[inline]
    pub fn with_unspecified_literal(&self) -> Self {
        Self { literal: Some(TStringLiteral::Unspecified), ..*self }
    }

    pub fn as_numeric(&self, retain_literal: bool) -> Self {
        Self {
            literal: if retain_literal { self.literal.clone() } else { None },
            is_numeric: true,
            is_truthy: self.is_truthy,
            is_non_empty: true,
            is_lowercase: true, // Numeric strings are considered lowercase
        }
    }
}

impl TType for TString {
    fn needs_population(&self) -> bool {
        false
    }

    fn is_expandable(&self) -> bool {
        false
    }

    fn get_id(&self) -> Atom {
        let s = match &self.literal {
            Some(TStringLiteral::Value(s)) => return concat_atom!("string('", s, "')"),
            Some(_) => {
                if self.is_truthy {
                    if self.is_numeric {
                        "truthy-numeric-literal-string"
                    } else if self.is_lowercase {
                        "truthy-lowercase-literal-string"
                    } else {
                        "truthy-literal-string"
                    }
                } else if self.is_numeric {
                    "numeric-literal-string"
                } else if self.is_non_empty {
                    if self.is_lowercase { "lowercase-non-empty-literal-string" } else { "non-empty-literal-string" }
                } else if self.is_lowercase {
                    "lowercase-literal-string"
                } else {
                    "literal-string"
                }
            }
            None => {
                if self.is_truthy {
                    if self.is_numeric {
                        "truthy-numeric-string"
                    } else if self.is_lowercase {
                        "truthy-lowercase-string"
                    } else {
                        "truthy-string"
                    }
                } else if self.is_numeric {
                    "numeric-string"
                } else if self.is_non_empty {
                    if self.is_lowercase { "lowercase-non-empty-string" } else { "non-empty-string" }
                } else if self.is_lowercase {
                    "lowercase-string"
                } else {
                    "string"
                }
            }
        };

        atom(s)
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

impl<T> From<T> for TString
where
    T: AsRef<str>,
{
    /// Converts any type that can be referenced as a string slice into a `known_literal` StringScalar.
    /// Derives properties from the literal value.
    fn from(value: T) -> Self {
        Self::known_literal(atom(value.as_ref()))
    }
}
