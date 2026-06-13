//! `String \ String` axis-narrowing rules.

use mago_allocator::Arena;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::scalar::string::StringCasing;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::atom::payload::scalar::string::StringRefinementFlag;
use crate::ty::builder::TypeBuilder;

/// `String \ String` for axis-narrowing cases.
///
/// - Two distinct string literals: subtract is identity (the literal
///   sets are disjoint, but our `overlaps` returns `true` due to the
///   broader `String` family rules; we keep the input unchanged here so
///   the distributive fold still terminates correctly). A specific
///   literal removes only one value and has no canonical complement
///   form, so subtract stays identity for it.
/// - Equal literals: collapse to bottom.
/// - General string `\` broad non-empty / truthy string: only the empty
///   string `""` survives.
/// - General string `\` broad numeric string: keeps the non-numeric
///   strings - any string that doesn't parse as a number, plus the
///   empty string (numeric requires non-empty in PHP) - via a
///   `Negated` conjunct.
pub(in crate::ty::subtract) fn string_minus<'arena, S, A>(
    input: Atom<'arena>,
    removed: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Vec<Atom<'arena>>>
where
    S: Arena,
    A: Arena,
{
    let (Atom::String(input_payload), Atom::String(removed_payload)) = (input, removed) else {
        return None;
    };

    if let StringLiteral::Value(input_value) = input_payload.literal
        && let StringLiteral::Value(removed_value) = removed_payload.literal
        && input_value == removed_value
    {
        return Some(Vec::new());
    }

    let input_is_general = matches!(input_payload.literal, StringLiteral::None | StringLiteral::Unspecified)
        && input_payload.flags.is_empty()
        && matches!(input_payload.casing, StringCasing::Unspecified);

    let removed_is_broad = matches!(removed_payload.literal, StringLiteral::None | StringLiteral::Unspecified);
    let removed_requires_non_empty = removed_payload.flags.contains(StringRefinementFlag::NonEmpty)
        || removed_payload.flags.contains(StringRefinementFlag::Truthy);
    if input_is_general && removed_is_broad && removed_requires_non_empty {
        return Some(vec![builder.string_literal(b"")]);
    }

    if input_is_general && removed_is_broad && removed_payload.flags.contains(StringRefinementFlag::Numeric) {
        let removed_type = builder.union_of(&[removed]);
        let negated = builder.negated(removed_type);
        return Some(vec![builder.intersected(input, &[negated])]);
    }

    None
}
