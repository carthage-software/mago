//! Two scalar-widening modes for a [`Type`].
//!
//! - [`scalars`]: replace **every** scalar narrowing - literal *and*
//!   user-declared - with the family dominator. `42` / `int<0,10>` /
//!   `positive-int` → `int`; `"foo"` / `non-empty-string` / `truthy-
//!   numeric-string` → `string`; `1.5` → `float`; `true` / `false` →
//!   `bool`. `class-string<Foo>` / `class-string<of: T>` →
//!   `class-string<Any>` (preserves the `Class`/`Interface`/
//!   `Enum`/`Trait` kind).
//!
//! - [`literals`]: replace literal scalar values with their tightest
//!   non-literal refinement; preserve user-declared narrowings.
//!   `42` / `0` / `-1` / `literal-int` → `int`. `1.5` /
//!   `literal-float` → `float`. `true` / `false` → `bool`. `"foo"` →
//!   `non-empty-truthy-lowercase-string` (every refinement bit the
//!   literal value satisfies; empty string `""` → `string`).
//!   `class-string<Foo>` → `class-string<Any>` of the same kind.
//!   Ranges, refinement flags, casing, and any non-literal form pass
//!   through unchanged.
//!
//! Both modes descend through every nested-type carrier via the
//! [`crate::transform`] walker.

use mago_allocator::Arena;
use mago_flags::U8Flags;

use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeKind;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;
use crate::ty::atom::payload::scalar::float::FloatAtom;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::atom::payload::scalar::string::StringAtom;
use crate::ty::atom::payload::scalar::string::StringCasing;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::atom::payload::scalar::string::StringRefinementFlag;
use crate::ty::builder::TypeBuilder;
use crate::ty::transform;
use crate::ty::well_known;

/// Replace every scalar narrowing with its family dominator.
#[inline]
pub fn scalars<'arena, S, A>(ty: Type<'arena>, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    transform::map(ty, widen_atom_scalar, builder)
}

/// Replace literal scalar values with their tightest non-literal
/// refinement; preserve user-declared narrowings.
#[inline]
pub fn literals<'arena, S, A>(ty: Type<'arena>, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    transform::map_with_builder(ty, widen_atom_literal, builder)
}

#[inline]
fn widen_atom_scalar(atom: Atom<'_>) -> Atom<'_> {
    match atom {
        Atom::Int(_) => well_known::INT,
        Atom::Float(_) => well_known::FLOAT,
        Atom::String(_) => well_known::STRING,
        Atom::True | Atom::False => well_known::BOOL,
        Atom::ClassLikeString(payload) => widen_class_like_string_to_any(atom, payload),
        _ => atom,
    }
}

#[inline]
fn widen_atom_literal<'arena, S, A>(atom: Atom<'arena>, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    match atom {
        Atom::Int(payload) => widen_int_literal(atom, payload),
        Atom::Float(payload) => widen_float_literal(atom, payload),
        Atom::String(payload) => widen_string_literal(atom, payload, builder),
        Atom::True | Atom::False => well_known::BOOL,
        Atom::ClassLikeString(payload) => widen_class_like_string_to_any(atom, payload),
        _ => atom,
    }
}

#[inline]
fn widen_int_literal<'arena>(atom: Atom<'arena>, payload: IntAtom<'arena>) -> Atom<'arena> {
    match payload {
        IntAtom::Literal(_) | IntAtom::UnspecifiedLiteral => well_known::INT,
        _ => atom,
    }
}

#[inline]
fn widen_float_literal(atom: Atom<'_>, payload: FloatAtom) -> Atom<'_> {
    match payload {
        FloatAtom::Literal(_) | FloatAtom::UnspecifiedLiteral => well_known::FLOAT,
        _ => atom,
    }
}

/// Build a non-literal `String` atom capturing every PHP-correct
/// refinement bit the literal value satisfies. Empty strings collapse
/// to `string` naturally (every inferred flag is false, so the built
/// shape is the well-known dominator).
#[inline]
fn widen_string_literal<'arena, S, A>(
    atom: Atom<'arena>,
    payload: &StringAtom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Atom<'arena>
where
    S: Arena,
    A: Arena,
{
    match payload.literal {
        StringLiteral::None => atom,
        StringLiteral::Unspecified => builder.string(StringAtom { literal: StringLiteral::None, ..*payload }),
        StringLiteral::Value(value) => {
            let bytes = value.as_bytes();
            let is_numeric = string_is_numeric(bytes);
            let is_non_empty = is_numeric || !bytes.is_empty();
            let is_truthy = is_non_empty && bytes != b"0";

            let mut flags = U8Flags::empty();
            flags.set_value(StringRefinementFlag::Numeric, is_numeric);
            flags.set_value(StringRefinementFlag::NonEmpty, is_non_empty);
            flags.set_value(StringRefinementFlag::Truthy, is_truthy);
            let casing = infer_casing(bytes);

            builder.string(StringAtom { literal: StringLiteral::None, casing, flags })
        }
    }
}

#[inline]
fn widen_class_like_string_to_any<'arena>(atom: Atom<'arena>, payload: &ClassLikeStringAtom<'arena>) -> Atom<'arena> {
    match payload.specifier {
        ClassLikeStringSpecifier::Any => atom,
        _ => match payload.kind {
            ClassLikeKind::Class => well_known::CLASS_STRING,
            ClassLikeKind::Interface => well_known::INTERFACE_STRING,
            ClassLikeKind::Enum => well_known::ENUM_STRING,
            ClassLikeKind::Trait => well_known::TRAIT_STRING,
        },
    }
}

/// Lowercase only when at least one lowercase letter is present and
/// no uppercase letter is. Uppercase symmetric. Strings with no
/// letters at all (digits, punctuation) get
/// [`StringCasing::Unspecified`] to avoid claiming a casing the
/// string does not exhibit.
#[inline]
fn infer_casing(bytes: &[u8]) -> StringCasing {
    let has_upper = bytes.iter().any(u8::is_ascii_uppercase);
    let has_lower = bytes.iter().any(u8::is_ascii_lowercase);
    match (has_upper, has_lower) {
        (false, true) => StringCasing::Lowercase,
        (true, false) => StringCasing::Uppercase,
        _ => StringCasing::Unspecified,
    }
}

/// Checks if a string is numeric according to PHP's definition.
///
/// Trims leading/trailing whitespace, strips a leading sign, removes
/// leading zeros, and uses `f64`'s parser for the remainder.
#[inline]
fn string_is_numeric(input: &[u8]) -> bool {
    let Ok(text) = core::str::from_utf8(input) else { return false };
    let mut maybe_numeric = text.trim();
    if maybe_numeric.is_empty() {
        return false;
    }

    if maybe_numeric.starts_with('+') || maybe_numeric.starts_with('-') {
        maybe_numeric = &maybe_numeric[1..];

        if maybe_numeric.is_empty() {
            return false;
        }
    }

    maybe_numeric = maybe_numeric.trim_start_matches('0');
    if maybe_numeric.is_empty() {
        return true;
    }

    maybe_numeric.parse::<f64>().is_ok()
}
