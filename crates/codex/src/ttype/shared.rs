//! A collection of shared, static, and lazily-initialized `TAtomic` types.
//!
//! This module provides canonical, reusable instances for common PHP types.
//! Using these constants avoids repeated allocations for frequently used types.

use std::sync::LazyLock;

use mago_atom::empty_atom;

use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::array::TArray;
use crate::ttype::atomic::array::keyed::TKeyedArray;
use crate::ttype::atomic::callable::TCallable;
use crate::ttype::atomic::callable::TCallableSignature;
use crate::ttype::atomic::iterable::TIterable;
use crate::ttype::atomic::mixed::TMixed;
use crate::ttype::atomic::object::TObject;
use crate::ttype::atomic::resource::TResource;
use crate::ttype::atomic::scalar::TScalar;
use crate::ttype::atomic::scalar::int::TInteger;
use crate::ttype::atomic::scalar::string::TString;
use crate::ttype::atomic::scalar::string::TStringLiteral;
use crate::ttype::get_arraykey;
use crate::ttype::get_mixed;

/// A static `TAtomic` representing the integer literal `1`.
pub const ONE_INT_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::literal_int(1));
/// A static `TAtomic` representing the integer literal `0`.
pub const ZERO_INT_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::literal_int(0));
/// A static `TAtomic` representing the integer literal `-1`.
pub const MINUS_ONE_INT_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::literal_int(-1));
/// A static `TAtomic` representing the general `int` type.
pub const INT_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::int());
/// A static `TAtomic` representing a positive integer (`positive-int` or `int<1, max>`).
pub const POSITIVE_INT_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::Integer(TInteger::positive()));
/// A static `TAtomic` representing a non-positive integer (`non-positive-int` or `int<min, 0>`).
pub const NON_POSITIVE_INT_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::Integer(TInteger::non_positive()));
/// A static `TAtomic` representing a negative integer (`negative-int` or `int<min, -1>`).
pub const NEGATIVE_INT_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::Integer(TInteger::negative()));
/// A static `TAtomic` representing a non-negative integer (`non-negative-int` or `int<0, max>`).
pub const NON_NEGATIVE_INT_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::Integer(TInteger::non_negative()));
/// A static `TAtomic` for the general `string` type.
pub const STRING_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::String(TString::new(None, false, false, false, false)));
/// A static `TAtomic` for a `lowercase-string`.
pub const LOWERCASE_STRING_ATOMIC: &TAtomic =
    &TAtomic::Scalar(TScalar::String(TString::new(None, false, false, false, true)));
/// A static `TAtomic` for a `non-empty-string`.
pub const NON_EMPTY_STRING_ATOMIC: &TAtomic =
    &TAtomic::Scalar(TScalar::String(TString::new(None, false, false, true, false)));
/// A static `TAtomic` for a `non-empty-lowercase-string`.
pub const NON_EMPTY_LOWERCASE_STRING_ATOMIC: &TAtomic =
    &TAtomic::Scalar(TScalar::String(TString::new(None, false, false, true, true)));
/// A static `TAtomic` for a `truthy-string`.
pub const TRUTHY_STRING_ATOMIC: &TAtomic =
    &TAtomic::Scalar(TScalar::String(TString::new(None, false, true, false, false)));
/// A static `TAtomic` for a `truthy-lowercase-string`.
pub const TRUTHY_LOWERCASE_STRING_ATOMIC: &TAtomic =
    &TAtomic::Scalar(TScalar::String(TString::new(None, false, true, false, true)));
/// A static `TAtomic` for a `numeric-string`.
pub const NUMERIC_STRING_ATOMIC: &TAtomic =
    &TAtomic::Scalar(TScalar::String(TString::new(None, true, false, false, false)));
/// A static `TAtomic` for a `numeric-string` that is also `truthy`.
pub const NUMERIC_TRUTHY_STRING_ATOMIC: &TAtomic =
    &TAtomic::Scalar(TScalar::String(TString::new(None, true, true, false, false)));
/// A static `TAtomic` representing the `class-string` type.
pub const CLASS_STRING_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::class_string());
/// A static `TAtomic` representing the `interface-string` type.
pub const INTERFACE_STRING_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::interface_string());
/// A static `TAtomic` representing the `enum-string` type.
pub const ENUM_STRING_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::enum_string());
/// A static `TAtomic` representing the `trait-string` type.
pub const TRAIT_STRING_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::trait_string());
/// A static `TAtomic` representing the `float` type.
pub const FLOAT_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::float());
/// A static `TAtomic` representing the `mixed` type.
pub const MIXED_ATOMIC: &TAtomic = &TAtomic::Mixed(TMixed::new());
/// A static `TAtomic` representing a `mixed` type that originates from an `isset()` check inside a loop.
pub const ISSET_FROM_LOOP_MIXED_ATOMIC: &TAtomic = &TAtomic::Mixed(TMixed::isset_from_loop());
/// A static `TAtomic` representing the `never` type, which indicates an impossible state.
pub const NEVER_ATOMIC: &TAtomic = &TAtomic::Never;
/// A static `TAtomic` representing any value that is not `null`.
pub const NON_NULL_ATOMIC: &TAtomic = &TAtomic::Mixed(TMixed::non_null());
/// A static `TAtomic` representing any "falsy" value (e.g., `false`, `0`, `""`, `[]`).
pub const FALSY_MIXED_ATOMIC: &TAtomic = &TAtomic::Mixed(TMixed::falsy());
/// A static `TAtomic` representing any "truthy" value.
pub const TRUTHY_MIXED_ATOMIC: &TAtomic = &TAtomic::Mixed(TMixed::truthy());
/// A static `TAtomic` representing the `resource` type.
pub const RESOURCE_ATOMIC: &TAtomic = &TAtomic::Resource(TResource::new(None));
/// A static `TAtomic` representing an open `resource`.
pub const OPEN_RESOURCE_ATOMIC: &TAtomic = &TAtomic::Resource(TResource::open());
/// A static `TAtomic` representing a closed `resource`.
pub const CLOSED_RESOURCE_ATOMIC: &TAtomic = &TAtomic::Resource(TResource::closed());
/// A static `TAtomic` used as a temporary placeholder during type reconciliation.
pub const PLACEHOLDER_ATOMIC: &TAtomic = &TAtomic::Placeholder;
/// A static `TAtomic` representing the `void` type.
pub const VOID_ATOMIC: &TAtomic = &TAtomic::Void;
/// A static `TAtomic` representing the `null` type.
pub const NULL_ATOMIC: &TAtomic = &TAtomic::Null;
/// A static `TAtomic` representing the `array-key` type (`int|string`).
pub const ARRAYKEY_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::ArrayKey);
/// A static `TAtomic` representing the `bool` type.
pub const BOOL_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::bool());
/// A static `TAtomic` representing the literal `false` type.
pub const FALSE_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::r#false());
/// A static `TAtomic` representing the literal `true` type.
pub const TRUE_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::r#true());
/// A static `TAtomic` representing the general `object` type.
pub const OBJECT_ATOMIC: &TAtomic = &TAtomic::Object(TObject::Any);
/// A static `TAtomic` representing the `numeric` type (`int|float|numeric-string`).
pub const NUMERIC_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::Numeric);
/// A static `TAtomic` representing an empty string literal (`""`).
pub static EMPTY_STRING_ATOMIC: LazyLock<TAtomic> = LazyLock::new(|| {
    TAtomic::Scalar(TScalar::String(TString {
        literal: Some(TStringLiteral::Value(empty_atom())),
        is_numeric: false,
        is_truthy: false,
        is_non_empty: false,
        is_lowercase: false,
    }))
});
/// A static `TAtomic` representing a `literal-string` where the value is unknown.
pub const UNSPECIFIED_LITERAL_STRING_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::unspecified_literal_string(false));
/// A static `TAtomic` representing a non-empty `literal-string` where the value is unknown.
pub const NON_EMPTY_UNSPECIFIED_LITERAL_STRING_ATOMIC: &TAtomic =
    &TAtomic::Scalar(TScalar::unspecified_literal_string(true));
/// A static `TAtomic` representing the `scalar` type (`int|float|string|bool`).
pub const SCALAR_ATOMIC: &TAtomic = &TAtomic::Scalar(TScalar::Generic);

/// A lazily-initialized static `TAtomic` for a mixed iterable (`iterable<mixed, mixed>`).
pub static MIXED_ITERABLE_ATOMIC: LazyLock<TAtomic> = LazyLock::new(|| {
    TAtomic::Iterable(TIterable {
        key_type: Box::new(get_mixed()),
        value_type: Box::new(get_mixed()),
        intersection_types: None,
    })
});

/// A lazily-initialized static `TAtomic` for an empty array (`array<never, never>`).
pub static EMPTY_KEYED_ARRAY_ATOMIC: LazyLock<TAtomic> =
    LazyLock::new(|| TAtomic::Array(TArray::Keyed(TKeyedArray::new())));
/// A lazily-initialized static `TAtomic` for a mixed array (`array<array-key, mixed>`).
pub static MIXED_KEYED_ARRAY_ATOMIC: LazyLock<TAtomic> = LazyLock::new(|| {
    TAtomic::Array(TArray::Keyed(TKeyedArray::new_with_parameters(Box::new(get_arraykey()), Box::new(get_mixed()))))
});
/// A lazily-initialized static `TAtomic` for a mixed callable (`callable`).
pub static MIXED_CALLABLE_ATOMIC: LazyLock<TAtomic> =
    LazyLock::new(|| TAtomic::Callable(TCallable::Signature(TCallableSignature::mixed(false))));
/// A lazily-initialized static `TAtomic` for a mixed closure (`Closure`).
pub static MIXED_CLOSURE_ATOMIC: LazyLock<TAtomic> =
    LazyLock::new(|| TAtomic::Callable(TCallable::Signature(TCallableSignature::mixed(true))));

/// A static slice of atomics representing the union type `int|float`.
pub const INT_FLOAT_ATOMIC_SLICE: &[TAtomic] = &[TAtomic::Scalar(TScalar::int()), TAtomic::Scalar(TScalar::float())];
/// A static slice of atomics representing the union type `int|string`.
pub const INT_STRING_ATOMIC_SLICE: &[TAtomic] = &[TAtomic::Scalar(TScalar::int()), TAtomic::Scalar(TScalar::string())];
/// A static slice of atomics representing the union type `null|scalar`.
pub const NULL_SCALAR_ATOMIC_SLICE: &[TAtomic] = &[TAtomic::Null, TAtomic::Scalar(TScalar::Generic)];
/// A static slice of atomics representing the union type `null|string`.
pub const NULL_STRING_ATOMIC_SLICE: &[TAtomic] = &[TAtomic::Null, TAtomic::Scalar(TScalar::string())];
/// A static slice of atomics representing the union type `null|int`.
pub const NULL_INT_ATOMIC_SLICE: &[TAtomic] = &[TAtomic::Null, TAtomic::Scalar(TScalar::int())];
/// A static slice of atomics representing the union type `null|float`.
pub const NULL_FLOAT_ATOMIC_SLICE: &[TAtomic] = &[TAtomic::Null, TAtomic::Scalar(TScalar::float())];
/// A static slice of atomics representing the union type `null|object`.
pub const NULL_OBJECT_ATOMIC_SLICE: &[TAtomic] = &[TAtomic::Null, TAtomic::Object(TObject::Any)];

/// A static slice of atomics representing the union type `-1|0|1`, commonly
/// returned by comparison operations like the spaceship operator (`<=>`).
pub const SIGNUM_RESULT_SLICE: &[TAtomic] = &[
    TAtomic::Scalar(TScalar::literal_int(-1)),
    TAtomic::Scalar(TScalar::literal_int(0)),
    TAtomic::Scalar(TScalar::literal_int(1)),
];
