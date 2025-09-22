use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::atom;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::scalar::bool::TBool;
use crate::ttype::atomic::scalar::class_like_string::TClassLikeString;
use crate::ttype::atomic::scalar::float::TFloat;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::atomic::scalar::string::TString;

pub mod bool;
pub mod class_like_string;
pub mod float;
pub mod int;
pub mod string;

/// Represents the distinct atomic types for PHP scalars.
///
/// This includes general types (int, float, string, bool), literal types,
/// union types (num, array-key), and the top type (scalar).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum TScalar {
    /// Represents the top type `scalar`, encompassing all other scalar variants.
    Generic,
    /// Represents the union type `numeric` (`int` | `float` | `string` that is numeric).
    Numeric,
    /// Represents the union type `array-key` (`int` | `string`).
    ArrayKey,
    /// Represents boolean types (`bool`, literal `true`, literal `false`).
    Bool(TBool),
    /// Represents integer types (`int`, literal `int`).
    Integer(TInteger),
    /// Represents float types (`float`, literal `float`).
    Float(TFloat),
    /// Represents string types (`string`, literal strings, potentially with known properties).
    String(TString),
    /// Represents class-like string types (`class-string`, `interface-string`, `enum-string`, potentially with `<T>`).
    ClassLikeString(TClassLikeString),
}

impl TScalar {
    /// Creates the `scalar` (Generic) type.
    #[inline]
    pub const fn generic() -> Self {
        TScalar::Generic
    }

    /// Creates the `numeric` (`int` | `float` | `string` that is numeric) type.
    #[inline]
    pub const fn numeric() -> Self {
        TScalar::Numeric
    }

    /// Creates the `array-key` (`int` | `string`) type.
    #[inline]
    pub const fn array_key() -> Self {
        TScalar::ArrayKey
    }

    /// Creates the general `bool` type.
    #[inline]
    pub const fn bool() -> Self {
        TScalar::Bool(TBool::general())
    }

    /// Creates the literal `true` type.
    #[inline]
    pub const fn r#true() -> Self {
        TScalar::Bool(TBool::r#true())
    }

    /// Creates the literal `false` type.
    #[inline]
    pub const fn r#false() -> Self {
        TScalar::Bool(TBool::r#false())
    }

    /// Creates the general `int` type.
    #[inline]
    pub const fn int() -> Self {
        TScalar::Integer(TInteger::unspecified())
    }

    /// Creates a literal `int` type (e.g., `123`).
    #[inline]
    pub const fn literal_int(value: i64) -> Self {
        TScalar::Integer(TInteger::literal(value))
    }

    /// Creates the general `float` type.
    #[inline]
    pub const fn float() -> Self {
        TScalar::Float(TFloat::general())
    }

    /// Creates a literal `float` type (e.g., `12.3`).
    #[inline]
    pub fn literal_float(value: f64) -> Self {
        TScalar::Float(TFloat::literal(value))
    }

    /// Creates the `numeric-string` type.
    #[inline]
    pub const fn numeric_string() -> Self {
        TScalar::String(TString::general_with_props(true, false, false, true))
    }

    /// Creates the general `string` type.
    #[inline]
    pub const fn string() -> Self {
        TScalar::String(TString::general())
    }

    /// Creates the non-empty `string` type.
    #[inline]
    pub const fn non_empty_string() -> Self {
        TScalar::String(TString::non_empty())
    }

    /// Creates a literal `string` type with a known value (e.g., `"hello"`).
    #[inline]
    pub fn literal_string(value: Atom) -> Self {
        TScalar::String(TString::known_literal(value))
    }

    /// Creates a literal `class-string` type with a known value (e.g., `"MyClass"`).
    #[inline]
    pub fn literal_class_string(value: Atom) -> Self {
        TScalar::ClassLikeString(TClassLikeString::literal(value))
    }

    /// Creates a literal `string` type with an unspecified value
    #[inline]
    pub const fn unspecified_literal_string(non_empty: bool) -> Self {
        TScalar::String(TString::unspecified_literal(non_empty))
    }

    /// Creates the general `class-string` type (no constraint `<T>`).
    #[inline]
    pub const fn class_string() -> Self {
        TScalar::ClassLikeString(TClassLikeString::class_string())
    }

    /// Creates the general `interface-string` type (no constraint `<T>`).
    #[inline]
    pub const fn interface_string() -> Self {
        TScalar::ClassLikeString(TClassLikeString::interface_string())
    }

    /// Creates the general `enum-string` type (no constraint `<T>`).
    #[inline]
    pub const fn enum_string() -> Self {
        TScalar::ClassLikeString(TClassLikeString::enum_string())
    }

    /// Creates the general `trait-string` type (no constraint `<T>`).
    #[inline]
    pub const fn trait_string() -> Self {
        TScalar::ClassLikeString(TClassLikeString::trait_string())
    }

    /// Creates the `class-string<T>` type.
    #[inline]
    pub fn class_string_of_type(atomic_type: TAtomic) -> Self {
        TScalar::ClassLikeString(TClassLikeString::class_string_of_type(atomic_type))
    }

    /// Creates the `interface-string<T>` type.
    #[inline]
    pub fn interface_string_of_type(atomic_type: TAtomic) -> Self {
        TScalar::ClassLikeString(TClassLikeString::interface_string_of_type(atomic_type))
    }

    /// Creates the `enum-string<T>` type.
    #[inline]
    pub fn enum_string_of_type(atomic_type: TAtomic) -> Self {
        TScalar::ClassLikeString(TClassLikeString::enum_string_of_type(atomic_type))
    }

    /// Checks if this is the top type `scalar`.
    #[inline]
    pub const fn is_generic(&self) -> bool {
        matches!(self, TScalar::Generic)
    }

    /// Checks if this is the `array-key` type (`int` | `string`).
    #[inline]
    pub const fn is_array_key(&self) -> bool {
        matches!(self, TScalar::ArrayKey)
    }

    /// Checks if this is any kind of boolean (`bool`, `true`, `false`).
    #[inline]
    pub const fn is_bool(&self) -> bool {
        matches!(self, TScalar::Bool(_))
    }

    /// Checks if this is the general `bool` type.
    #[inline]
    pub const fn is_general_bool(&self) -> bool {
        matches!(self, TScalar::Bool(b) if b.is_general())
    }

    /// Checks if this is the literal `true` type.
    #[inline]
    pub const fn is_true(&self) -> bool {
        matches!(self, TScalar::Bool(b) if b.is_true())
    }

    /// Checks if this is the literal `false` type.
    #[inline]
    pub const fn is_false(&self) -> bool {
        matches!(self, TScalar::Bool(b) if b.is_false())
    }

    /// Checks if this is any kind of integer (`int`, literal `int`).
    #[inline]
    pub const fn is_int(&self) -> bool {
        matches!(self, TScalar::Integer(_))
    }

    /// Checks if this is the general `int` type.
    #[inline]
    pub const fn is_general_int(&self) -> bool {
        matches!(self, TScalar::Integer(i) if i.is_unspecified())
    }

    /// Checks if this is a literal `int` type.
    #[inline]
    pub const fn is_literal_int(&self) -> bool {
        matches!(self, TScalar::Integer(i) if i.is_literal())
    }

    /// Checks if this is any kind of numeric type (`int`, `float`, `num`, or a string that is numeric).
    #[inline]
    pub const fn is_literal_value(&self) -> bool {
        match self {
            TScalar::String(str) => str.is_known_literal(),
            TScalar::Integer(int) => int.is_literal(),
            TScalar::Float(float) => float.is_literal(),
            TScalar::Bool(bool) => !bool.is_general(),
            _ => false,
        }
    }

    /// Checks if this is any kind of numeric type (`int`, `float`, `num`, or a string that is numeric).
    #[inline]
    pub const fn is_numeric(&self) -> bool {
        match self {
            TScalar::Numeric => true,
            TScalar::Integer(_) | TScalar::Float(_) => true,
            TScalar::String(str) => str.is_numeric,
            _ => false,
        }
    }

    /// Checks if this is any kind of number type (`int`, `float`, or `num`).
    #[inline]
    pub const fn is_int_or_float(&self) -> bool {
        matches!(self, TScalar::Integer(_) | TScalar::Float(_))
    }

    /// Gets the value if this is a literal `int`.
    #[inline]
    pub const fn get_literal_int_value(&self) -> Option<i64> {
        match self {
            TScalar::Integer(i) => i.get_literal_value(),
            _ => None,
        }
    }

    /// Gets the maximum value if this is an integer of a specific size.
    #[inline]
    pub const fn get_maximum_int_value(&self) -> Option<i64> {
        match self {
            TScalar::Integer(i) => i.get_maximum_value(),
            _ => None,
        }
    }

    /// Gets the minimum value if this is an integer of a specific size.
    #[inline]
    pub const fn get_minimum_int_value(&self) -> Option<i64> {
        match self {
            TScalar::Integer(i) => i.get_minimum_value(),
            _ => None,
        }
    }

    /// Checks if this is any kind of float (`float`, literal `float`).
    #[inline]
    pub const fn is_float(&self) -> bool {
        matches!(self, TScalar::Float(_))
    }

    /// Checks if this is the general `float` type.
    #[inline]
    pub const fn is_general_float(&self) -> bool {
        matches!(self, TScalar::Float(f) if f.is_general())
    }

    /// Checks if this is a literal `float` type.
    #[inline]
    pub const fn is_literal_float(&self) -> bool {
        matches!(self, TScalar::Float(f) if f.is_literal())
    }

    /// Gets the value if this is a literal `float`.
    #[inline]
    pub fn get_literal_float_value(&self) -> Option<f64> {
        match self {
            TScalar::Float(f) => f.get_literal_value(),
            _ => None,
        }
    }

    /// Checks if this is any kind of string represented by `StringScalar`.
    #[inline]
    pub const fn is_string(&self) -> bool {
        matches!(self, TScalar::String(_))
    }

    /// Checks if this is any kind of string represented by `StringScalar` or `ClassLikeStringScalar`.
    #[inline]
    pub const fn is_any_string(&self) -> bool {
        matches!(self, TScalar::String(_) | TScalar::ClassLikeString(_))
    }

    /// Checks if this is a non-boring string (not general).
    #[inline]
    pub const fn is_non_boring_string(&self) -> bool {
        match self {
            TScalar::String(s) => !s.is_boring(),
            TScalar::ClassLikeString(_) => true,
            _ => false,
        }
    }

    /// Checks if this is the general `string` type (not known literal origin).
    #[inline]
    pub const fn is_general_string(&self) -> bool {
        matches!(self, TScalar::String(s) if s.is_general())
    }

    /// Checks if this is a literal `string` with a *known* value.
    #[inline]
    pub const fn is_known_literal_string(&self) -> bool {
        matches!(self, TScalar::String(s) if s.is_known_literal())
    }

    /// Checks if this is a `string` that is known to be non-empty.
    #[inline]
    pub const fn is_non_empty_string(&self) -> bool {
        matches!(self, TScalar::String(s) if s.is_non_empty())
    }

    /// Checks if this is a string known to be literal, but with an *unspecified* value.
    #[inline]
    pub const fn is_unspecified_literal_string(&self) -> bool {
        matches!(self, TScalar::String(s) if s.is_unspecified_literal())
    }

    /// Checks if this is any string known to originate from a literal (known or unspecified value).
    #[inline]
    pub const fn is_literal_origin_string(&self) -> bool {
        matches!(self, TScalar::String(s) if s.is_literal_origin())
    }

    /// Gets the value if this is a literal `string` with a known value.
    #[inline]
    pub fn get_known_literal_string_value(&self) -> Option<&str> {
        match self {
            TScalar::String(s) => s.get_known_literal_value(),
            _ => None,
        }
    }

    #[inline]
    pub const fn is_literal_class_string(&self) -> bool {
        match self {
            TScalar::ClassLikeString(s) => s.is_literal(),
            _ => false,
        }
    }

    #[inline]
    pub fn get_literal_class_string_value(&self) -> Option<Atom> {
        match self {
            TScalar::ClassLikeString(s) => s.literal_value(),
            _ => None,
        }
    }

    /// Checks if this is any kind of class-like string type.
    #[inline]
    pub const fn is_class_string_type(&self) -> bool {
        matches!(self, TScalar::ClassLikeString(_))
    }

    /// Checks if this is a `class-string` (any constraint).
    #[inline]
    pub const fn is_class_string_kind(&self) -> bool {
        matches!(self, TScalar::ClassLikeString(cls) if cls.is_class_kind())
    }

    /// Checks if this is an `interface-string` (any constraint).
    #[inline]
    pub const fn is_interface_string_kind(&self) -> bool {
        matches!(self, TScalar::ClassLikeString(cls) if cls.is_interface_kind())
    }

    /// Checks if this is an `enum-string` (any constraint).
    #[inline]
    pub const fn is_enum_string_kind(&self) -> bool {
        matches!(self, TScalar::ClassLikeString(cls) if cls.is_enum_kind())
    }

    /// Check if the scalar is truthy (i.e., will resolve to `true` in a boolean context).
    #[inline]
    pub fn is_truthy(&self) -> bool {
        match &self {
            TScalar::Bool(b) => b.is_true(),
            TScalar::Integer(i) => match i.get_literal_value() {
                Some(v) => v != 0,
                None => match i.get_minimum_value() {
                    Some(v) => v != 0,
                    None => match i.get_maximum_value() {
                        Some(v) => v != 0,
                        None => false,
                    },
                },
            },
            TScalar::Float(f) => f.get_literal_value().is_some_and(|v| v != 0.0),
            TScalar::String(s) => s.is_truthy,
            TScalar::ClassLikeString(_) => true,
            _ => false,
        }
    }

    /// Check if the scalar is falsy (i.e., will resolve to `false` in a boolean context).
    #[inline]
    pub fn is_falsy(&self) -> bool {
        match &self {
            TScalar::Bool(b) => b.is_false(),
            TScalar::Integer(i) => match i.get_literal_value() {
                Some(v) => v == 0,
                None => false,
            },
            TScalar::Float(f) => f.get_literal_value().is_some_and(|v| v == 0.0),
            TScalar::String(s) => s.get_known_literal_value().is_some_and(|v| v.is_empty()),
            TScalar::ClassLikeString(_) => false,
            _ => false,
        }
    }

    #[inline]
    pub const fn is_boring(&self) -> bool {
        match self {
            TScalar::Numeric => true,
            TScalar::ArrayKey => true,
            TScalar::Bool(bool_scalar) => bool_scalar.is_general(),
            TScalar::Integer(int_scalar) => int_scalar.is_unspecified(),
            TScalar::Float(float_scalar) => float_scalar.is_general(),
            TScalar::String(string_scalar) => string_scalar.is_boring(),
            TScalar::ClassLikeString(class_like_string_scalar) => !class_like_string_scalar.has_constraint(),
            _ => false,
        }
    }

    /// Returns the inner `TBool` struct if this is a `Scalar::Bool`.
    #[inline]
    pub const fn as_bool(&self) -> Option<&TBool> {
        match self {
            TScalar::Bool(b) => Some(b),
            _ => None,
        }
    }

    /// Returns the inner `IntScalar` struct if this is a `Scalar::Int`.
    #[inline]
    pub const fn as_int(&self) -> Option<&TInteger> {
        match self {
            TScalar::Integer(i) => Some(i),
            _ => None,
        }
    }

    /// Returns the inner `FloatScalar` struct if this is a `Scalar::Float`.
    #[inline]
    pub const fn as_float(&self) -> Option<&TFloat> {
        match self {
            TScalar::Float(f) => Some(f),
            _ => None,
        }
    }

    /// Returns the inner `StringScalar` struct if this is a `Scalar::String`.
    #[inline]
    pub const fn as_string(&self) -> Option<&TString> {
        match self {
            TScalar::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the inner `ClassLikeStringScalar` struct if this is a `Scalar::ClassLikeString`.
    #[inline]
    pub const fn as_class_string(&self) -> Option<&TClassLikeString> {
        match self {
            TScalar::ClassLikeString(cs) => Some(cs),
            _ => None,
        }
    }
}

impl TType for TScalar {
    fn get_child_nodes<'a>(&'a self) -> Vec<TypeRef<'a>> {
        match self {
            TScalar::Bool(ttype) => ttype.get_child_nodes(),
            TScalar::Integer(ttype) => ttype.get_child_nodes(),
            TScalar::Float(ttype) => ttype.get_child_nodes(),
            TScalar::String(ttype) => ttype.get_child_nodes(),
            TScalar::ClassLikeString(ttype) => ttype.get_child_nodes(),
            _ => vec![],
        }
    }

    fn needs_population(&self) -> bool {
        match self {
            TScalar::Bool(ttype) => ttype.needs_population(),
            TScalar::Integer(ttype) => ttype.needs_population(),
            TScalar::Float(ttype) => ttype.needs_population(),
            TScalar::String(ttype) => ttype.needs_population(),
            TScalar::ClassLikeString(ttype) => ttype.needs_population(),
            _ => false,
        }
    }

    fn is_expandable(&self) -> bool {
        match self {
            TScalar::Bool(ttype) => ttype.is_expandable(),
            TScalar::Integer(ttype) => ttype.is_expandable(),
            TScalar::Float(ttype) => ttype.is_expandable(),
            TScalar::String(ttype) => ttype.is_expandable(),
            TScalar::ClassLikeString(ttype) => ttype.is_expandable(),
            _ => false,
        }
    }

    fn is_complex(&self) -> bool {
        match self {
            TScalar::Bool(ttype) => ttype.is_complex(),
            TScalar::Integer(ttype) => ttype.is_complex(),
            TScalar::Float(ttype) => ttype.is_complex(),
            TScalar::String(ttype) => ttype.is_complex(),
            TScalar::ClassLikeString(ttype) => ttype.is_complex(),
            _ => false,
        }
    }

    fn get_id(&self) -> Atom {
        match self {
            TScalar::Bool(t) => t.get_id(),
            TScalar::Float(t) => t.get_id(),
            TScalar::String(t) => t.get_id(),
            TScalar::ClassLikeString(t) => t.get_id(),
            TScalar::Integer(t) => t.get_id(),
            TScalar::Generic => atom("scalar"),
            TScalar::ArrayKey => atom("array-key"),
            TScalar::Numeric => atom("numeric"),
        }
    }

    fn get_pretty_id_with_indent(&self, indent: usize) -> Atom {
        match self {
            TScalar::Bool(t) => t.get_pretty_id_with_indent(indent),
            TScalar::Float(t) => t.get_pretty_id_with_indent(indent),
            TScalar::String(t) => t.get_pretty_id_with_indent(indent),
            TScalar::ClassLikeString(t) => t.get_pretty_id_with_indent(indent),
            TScalar::Integer(t) => t.get_pretty_id_with_indent(indent),
            TScalar::Generic => atom("scalar"),
            TScalar::ArrayKey => atom("array-key"),
            TScalar::Numeric => atom("numeric"),
        }
    }
}

impl From<TBool> for TScalar {
    fn from(t: TBool) -> Self {
        TScalar::Bool(t)
    }
}

impl From<TInteger> for TScalar {
    fn from(t: TInteger) -> Self {
        TScalar::Integer(t)
    }
}

impl From<TFloat> for TScalar {
    fn from(t: TFloat) -> Self {
        TScalar::Float(t)
    }
}

impl From<TString> for TScalar {
    fn from(t: TString) -> Self {
        TScalar::String(t)
    }
}

impl From<TClassLikeString> for TScalar {
    fn from(t: TClassLikeString) -> Self {
        TScalar::ClassLikeString(t)
    }
}
