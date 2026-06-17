//! `String` family meet: union-of-constraints algebra plus the
//! `numeric ∧ string` cross-kind crossing.

use mago_allocator::Arena;
use mago_flags::U8Flags;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::scalar::string::StringAtom;
use crate::ty::atom::payload::scalar::string::StringCasing;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::atom::payload::scalar::string::StringRefinementFlag;
use crate::ty::builder::TypeBuilder;

/// Intersect two `String` atoms. The result has every flag present in
/// either side (OR-merge), the casing constraint of either when the
/// other is unspecified (AND-merge), and a literal value when only one
/// side pins one. Opposite fixed casings collapse to `lit("")` when no
/// literal value is pinned (the only string satisfying both; `None`
/// when a non-empty flag rules even that out) or constrain a pinned
/// literal to carry no ASCII letters at all; literal-vs-flag and
/// literal-vs-casing incompatibilities collapse to `None` (disjoint).
#[inline]
pub(crate) fn string_meet<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    let (Atom::String(a_payload), Atom::String(b_payload)) = (a, b) else {
        return None;
    };

    let opposite_casings = matches!(
        (a_payload.casing, b_payload.casing),
        (StringCasing::Lowercase, StringCasing::Uppercase) | (StringCasing::Uppercase, StringCasing::Lowercase)
    );

    let casing = match (a_payload.casing, b_payload.casing) {
        (StringCasing::Lowercase, StringCasing::Lowercase) => StringCasing::Lowercase,
        (StringCasing::Uppercase, StringCasing::Uppercase) => StringCasing::Uppercase,
        (StringCasing::Unspecified, casing) | (casing, StringCasing::Unspecified) => casing,
        _ => StringCasing::Unspecified,
    };

    let flags = a_payload.flags.union(b_payload.flags);

    let literal = match (a_payload.literal, b_payload.literal) {
        (StringLiteral::Value(a_value), StringLiteral::Value(b_value)) => {
            if a_value == b_value {
                StringLiteral::Value(a_value)
            } else {
                return None;
            }
        }
        (StringLiteral::Value(value), StringLiteral::Unspecified)
        | (StringLiteral::Unspecified, StringLiteral::Value(value)) => StringLiteral::Value(value),
        (StringLiteral::Value(value), StringLiteral::None) | (StringLiteral::None, StringLiteral::Value(value)) => {
            StringLiteral::Value(value)
        }
        (StringLiteral::Unspecified, _) | (_, StringLiteral::Unspecified) => StringLiteral::Unspecified,
        (StringLiteral::None, StringLiteral::None) => StringLiteral::None,
    };

    if opposite_casings && matches!(literal, StringLiteral::None | StringLiteral::Unspecified) {
        let empty_string_admitted = !flags.contains(StringRefinementFlag::NonEmpty)
            && !flags.contains(StringRefinementFlag::Truthy)
            && !flags.contains(StringRefinementFlag::Numeric);
        if empty_string_admitted {
            return Some(builder.string_literal(b""));
        }

        return None;
    }

    let merged = StringAtom { literal, casing, flags };
    if !literal_satisfies_flags(merged.literal, merged.flags) {
        return None;
    }

    if opposite_casings
        && let StringLiteral::Value(value) = merged.literal
        && value.iter().any(u8::is_ascii_alphabetic)
    {
        return None;
    }

    if !literal_satisfies_casing(merged.literal, merged.casing) {
        return None;
    }

    Some(builder.string(merged))
}

/// `numeric ∧ string` is the set of strings whose value parses as a
/// number; i.e. the `numeric-string` refinement, preserving any
/// casing / literal / flags already on the string side.
pub(in crate::ty::meet) fn numeric_string_meet<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    let ((Atom::String(string_payload), _) | (_, Atom::String(string_payload))) = (a, b) else {
        return None;
    };

    let mut flags = string_payload.flags;
    flags.set(StringRefinementFlag::Numeric);

    let merged = StringAtom { literal: string_payload.literal, casing: string_payload.casing, flags };
    if !literal_satisfies_flags(merged.literal, merged.flags) {
        return None;
    }

    Some(builder.string(merged))
}

#[inline]
fn literal_satisfies_flags(literal: StringLiteral<'_>, flags: U8Flags<StringRefinementFlag>) -> bool {
    let StringLiteral::Value(value) = literal else {
        return true;
    };

    let bytes = value;
    if flags.contains(StringRefinementFlag::NonEmpty) && bytes.is_empty() {
        return false;
    }

    if flags.contains(StringRefinementFlag::Truthy) && (bytes.is_empty() || bytes == b"0") {
        return false;
    }

    if flags.contains(StringRefinementFlag::Numeric)
        && !core::str::from_utf8(bytes).is_ok_and(|text| text.parse::<i64>().is_ok() || text.parse::<f64>().is_ok())
    {
        return false;
    }

    true
}

#[inline]
fn literal_satisfies_casing(literal: StringLiteral<'_>, casing: StringCasing) -> bool {
    let StringLiteral::Value(value) = literal else {
        return true;
    };

    let bytes = value;
    match casing {
        StringCasing::Unspecified => true,
        StringCasing::Lowercase => !bytes.iter().any(u8::is_ascii_uppercase),
        StringCasing::Uppercase => !bytes.iter().any(u8::is_ascii_lowercase),
    }
}
