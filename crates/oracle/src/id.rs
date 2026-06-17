use std::hash::BuildHasher;
use std::hash::Hasher;

use foldhash::fast::FixedState;
#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct SymbolId(u64);

const TAG_CONSTANT: u8 = 0;
const TAG_CLASS_LIKE: u8 = 1;
const TAG_FUNCTION_LIKE: u8 = 2;
const TAG_CLASS_LIKE_CONSTANT: u8 = 3;
const TAG_ENUM_CASE: u8 = 4;
const TAG_PROPERTY: u8 = 5;
const TAG_PROPERTY_HOOK: u8 = 6;
const TAG_METHOD: u8 = 7;
const TAG_FUNCTION_LIKE_PARAMETER: u8 = 8;
const TAG_METHOD_PARAMETER: u8 = 9;
const TAG_PROPERTY_HOOK_PARAMETER: u8 = 10;
const TAG_NAMESPACE: u8 = 11;
const TAG_TYPE_ALIAS: u8 = 12;

impl SymbolId {
    /// Identifies a class-like type alias (`@type`). The class name is
    /// case-insensitive, the alias name case-sensitive.
    #[must_use]
    pub fn type_alias(class_name: &[u8], alias_name: &[u8]) -> Self {
        let mut hasher = FixedState::default().build_hasher();
        hasher.write_u8(TAG_TYPE_ALIAS);
        write_folded(&mut hasher, strip_leading_separator(class_name));
        write_verbatim(&mut hasher, alias_name);

        SymbolId(hasher.finish())
    }

    /// Identifies a namespace. Fully case-insensitive, mirroring PHP's
    /// case-insensitive namespace resolution.
    #[must_use]
    pub fn namespace(name: &[u8]) -> Self {
        let mut hasher = FixedState::default().build_hasher();
        hasher.write_u8(TAG_NAMESPACE);
        write_folded(&mut hasher, strip_leading_separator(name));

        SymbolId(hasher.finish())
    }

    /// Identifies a global constant. The namespace is case-insensitive, the
    /// short name case-sensitive.
    #[must_use]
    pub fn constant(name: &[u8]) -> Self {
        let mut hasher = FixedState::default().build_hasher();
        hasher.write_u8(TAG_CONSTANT);
        write_namespace_folded(&mut hasher, strip_leading_separator(name));

        SymbolId(hasher.finish())
    }

    /// Identifies a class-like entity (class, interface, trait, or enum). Fully
    /// case-insensitive.
    #[must_use]
    pub fn class_like(name: &[u8]) -> Self {
        let mut hasher = FixedState::default().build_hasher();
        hasher.write_u8(TAG_CLASS_LIKE);
        write_folded(&mut hasher, strip_leading_separator(name));

        SymbolId(hasher.finish())
    }

    /// Identifies a function-like entity (function, closure, or arrow function).
    /// Fully case-insensitive.
    #[must_use]
    pub fn function_like(name: &[u8]) -> Self {
        let mut hasher = FixedState::default().build_hasher();
        hasher.write_u8(TAG_FUNCTION_LIKE);
        write_folded(&mut hasher, strip_leading_separator(name));

        SymbolId(hasher.finish())
    }

    /// Identifies a class-like constant. The class name is case-insensitive, the
    /// constant name case-sensitive.
    #[must_use]
    pub fn class_like_constant(class_name: &[u8], constant_name: &[u8]) -> Self {
        let mut hasher = FixedState::default().build_hasher();
        hasher.write_u8(TAG_CLASS_LIKE_CONSTANT);
        write_folded(&mut hasher, strip_leading_separator(class_name));
        write_verbatim(&mut hasher, constant_name);

        SymbolId(hasher.finish())
    }

    /// Identifies an enum case. The enum name is case-insensitive, the case name
    /// case-sensitive.
    #[must_use]
    pub fn enum_case(enum_name: &[u8], case_name: &[u8]) -> Self {
        let mut hasher = FixedState::default().build_hasher();
        hasher.write_u8(TAG_ENUM_CASE);
        write_folded(&mut hasher, strip_leading_separator(enum_name));
        write_verbatim(&mut hasher, case_name);

        SymbolId(hasher.finish())
    }

    /// Identifies a property. The class name is case-insensitive, the property
    /// name case-sensitive.
    #[must_use]
    pub fn property(class_name: &[u8], property_name: &[u8]) -> Self {
        let mut hasher = FixedState::default().build_hasher();
        hasher.write_u8(TAG_PROPERTY);
        write_folded(&mut hasher, strip_leading_separator(class_name));
        write_verbatim(&mut hasher, property_name);

        SymbolId(hasher.finish())
    }

    /// Identifies a property hook. The class name and hook name are
    /// case-insensitive, the property name case-sensitive.
    #[must_use]
    pub fn property_hook(class_name: &[u8], property_name: &[u8], hook_name: &[u8]) -> Self {
        let mut hasher = FixedState::default().build_hasher();
        hasher.write_u8(TAG_PROPERTY_HOOK);
        write_folded(&mut hasher, strip_leading_separator(class_name));
        write_verbatim(&mut hasher, property_name);
        write_folded(&mut hasher, hook_name);

        SymbolId(hasher.finish())
    }

    /// Identifies a method. Both the class name and the method name are
    /// case-insensitive.
    #[must_use]
    pub fn method(class_name: &[u8], method_name: &[u8]) -> Self {
        let mut hasher = FixedState::default().build_hasher();
        hasher.write_u8(TAG_METHOD);
        write_folded(&mut hasher, strip_leading_separator(class_name));
        write_folded(&mut hasher, method_name);

        SymbolId(hasher.finish())
    }

    /// Identifies a parameter of a function-like. The function name is
    /// case-insensitive, the parameter name (sigil included) case-sensitive.
    #[must_use]
    pub fn function_like_parameter(function_like: &[u8], parameter: &[u8]) -> Self {
        let mut hasher = FixedState::default().build_hasher();
        hasher.write_u8(TAG_FUNCTION_LIKE_PARAMETER);
        write_folded(&mut hasher, strip_leading_separator(function_like));
        write_verbatim(&mut hasher, parameter);

        SymbolId(hasher.finish())
    }

    /// Identifies a parameter of a method. The class and method names are
    /// case-insensitive, the parameter name case-sensitive.
    #[must_use]
    pub fn method_parameter(class_name: &[u8], method_name: &[u8], parameter: &[u8]) -> Self {
        let mut hasher = FixedState::default().build_hasher();
        hasher.write_u8(TAG_METHOD_PARAMETER);
        write_folded(&mut hasher, strip_leading_separator(class_name));
        write_folded(&mut hasher, method_name);
        write_verbatim(&mut hasher, parameter);

        SymbolId(hasher.finish())
    }

    /// Identifies a parameter of a property hook (the `set` hook's value). The
    /// class and hook names are case-insensitive, the property and parameter
    /// names case-sensitive.
    #[must_use]
    pub fn property_hook_parameter(
        class_name: &[u8],
        property_name: &[u8],
        hook_name: &[u8],
        parameter: &[u8],
    ) -> Self {
        let mut hasher = FixedState::default().build_hasher();
        hasher.write_u8(TAG_PROPERTY_HOOK_PARAMETER);
        write_folded(&mut hasher, strip_leading_separator(class_name));
        write_verbatim(&mut hasher, property_name);
        write_folded(&mut hasher, hook_name);
        write_verbatim(&mut hasher, parameter);

        SymbolId(hasher.finish())
    }
}

fn strip_leading_separator(name: &[u8]) -> &[u8] {
    name.strip_prefix(b"\\").unwrap_or(name)
}

fn write_folded(hasher: &mut impl Hasher, bytes: &[u8]) {
    hasher.write_usize(bytes.len());
    for &byte in bytes {
        hasher.write_u8(byte.to_ascii_lowercase());
    }
}

fn write_verbatim(hasher: &mut impl Hasher, bytes: &[u8]) {
    hasher.write_usize(bytes.len());
    hasher.write(bytes);
}

fn write_namespace_folded(hasher: &mut impl Hasher, bytes: &[u8]) {
    let boundary = bytes.iter().rposition(|&byte| byte == b'\\').map_or(0, |position| position + 1);
    hasher.write_usize(bytes.len());
    for (index, &byte) in bytes.iter().enumerate() {
        hasher.write_u8(if index < boundary { byte.to_ascii_lowercase() } else { byte });
    }
}
