//! String-family join: axis merging and literal-count collapse.

use mago_allocator::Arena;

use crate::name::Name;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::scalar::string::StringAtom;
use crate::ty::atom::payload::scalar::string::StringCasing;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::atom::payload::scalar::string::StringRefinementFlag;
use crate::ty::builder::TypeBuilder;
use crate::ty::well_known::STRING;

/// Merge same-kind string atoms via AND-of-flags algebra. When
/// multiple strings are present, this folds them into a single
/// general/refined string plus the surviving incompatible literals.
///
/// The merge rules:
///
/// - `lower_string | upper_string` → `string` (casing collapses to
///   `Unspecified`).
/// - `non_empty_string | lit("")` → `string` (empty literal forces
///   `is_non_empty` and `is_truthy` and `is_numeric` off).
/// - `truthy_string | lit("0")` → `truthy_string`, `lit("0")` (literal
///   "0" is incompatible with truthy → kept separate).
/// - `numeric_string | lit("123")` → `numeric_string` (compatible
///   literal absorbed).
/// - `numeric_string | lit("abc")` → `numeric_string`, `lit("abc")`
///   (non-numeric literal stays separate).
pub fn apply_string_axis_merge_in_order<'arena, S, A>(
    atoms: &[Atom<'arena>],
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Vec<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    let mut other: Vec<Atom<'arena>> = Vec::with_capacity(atoms.len());
    let mut general: Option<StringAtom<'arena>> = None;
    let mut literals: Vec<Name<'arena>> = Vec::new();

    for &atom in atoms {
        let Atom::String(payload) = atom else {
            other.push(atom);
            continue;
        };
        let payload = *payload;
        if let StringLiteral::Value(value) = payload.literal {
            if let Some(ref mut existing) = general {
                let literal_value = value.as_bytes();
                let incompatible = (existing.flags.contains(StringRefinementFlag::Numeric)
                    && !is_numeric_string(literal_value))
                    || (existing.flags.contains(StringRefinementFlag::Truthy)
                        && (literal_value.is_empty() || literal_value == b"0"))
                    || (existing.flags.contains(StringRefinementFlag::NonEmpty) && literal_value.is_empty())
                    || (existing.casing == StringCasing::Lowercase && literal_value.iter().any(u8::is_ascii_uppercase))
                    || (existing.casing == StringCasing::Uppercase && literal_value.iter().any(u8::is_ascii_lowercase));
                if incompatible {
                    literals.push(value);
                } else {
                    *existing = combine_string_atoms(*existing, payload);
                }
            } else {
                literals.push(value);
            }
            continue;
        }

        match general {
            None => {
                let mut new_payload = payload;
                if new_payload.flags.contains(StringRefinementFlag::Truthy)
                    || new_payload.flags.contains(StringRefinementFlag::NonEmpty)
                    || new_payload.flags.contains(StringRefinementFlag::Numeric)
                    || new_payload.casing != StringCasing::Unspecified
                {
                    let mut kept_literals: Vec<Name<'arena>> = Vec::new();
                    for literal in &literals {
                        let value = literal.as_bytes();
                        if value.is_empty() {
                            let has_other_flags = new_payload.flags.contains(StringRefinementFlag::Truthy)
                                || new_payload.flags.contains(StringRefinementFlag::Numeric);
                            if has_other_flags {
                                kept_literals.push(*literal);
                            } else {
                                new_payload.flags.set_value(StringRefinementFlag::NonEmpty, false);
                            }
                            continue;
                        }

                        if value == b"0" {
                            if new_payload.flags.contains(StringRefinementFlag::Truthy) {
                                kept_literals.push(*literal);
                                continue;
                            }

                            new_payload.flags.set_value(StringRefinementFlag::Truthy, false);
                        }

                        if new_payload.flags.contains(StringRefinementFlag::Numeric) && !is_numeric_string(value) {
                            kept_literals.push(*literal);
                            continue;
                        }

                        let literal_casing_is_incompatible = match new_payload.casing {
                            StringCasing::Lowercase if value.iter().any(u8::is_ascii_uppercase) => true,
                            StringCasing::Uppercase if value.iter().any(u8::is_ascii_lowercase) => true,
                            _ => false,
                        };

                        if literal_casing_is_incompatible {
                            kept_literals.push(*literal);
                            continue;
                        }

                        new_payload.flags.set_value(
                            StringRefinementFlag::Numeric,
                            new_payload.flags.contains(StringRefinementFlag::Numeric) && is_numeric_string(value),
                        );
                        new_payload.casing = match new_payload.casing {
                            StringCasing::Lowercase => StringCasing::Lowercase,
                            StringCasing::Uppercase => StringCasing::Uppercase,
                            StringCasing::Unspecified => StringCasing::Unspecified,
                        };
                    }

                    literals = kept_literals;
                }

                general = Some(new_payload);
            }
            Some(ref mut existing) => {
                *existing = combine_string_atoms(*existing, payload);
            }
        }
    }

    let mut new_strings: Vec<Atom<'arena>> =
        literals.into_iter().map(|literal| builder.string_literal(literal.as_bytes())).collect();
    if let Some(payload) = general {
        new_strings.push(builder.string(payload));
    }

    other.extend(new_strings);
    other
}

#[inline]
fn combine_string_atoms<'arena>(left: StringAtom<'arena>, right: StringAtom<'arena>) -> StringAtom<'arena> {
    let literal = match (left.literal, right.literal) {
        (StringLiteral::Value(left_value), StringLiteral::Value(right_value)) => {
            if left_value == right_value {
                StringLiteral::Value(right_value)
            } else {
                StringLiteral::Unspecified
            }
        }
        (StringLiteral::Unspecified, _) | (_, StringLiteral::Unspecified) => StringLiteral::Unspecified,
        _ => StringLiteral::None,
    };

    let left_casing_neutral = matches!(
        left.literal,
        StringLiteral::Value(value) if value.as_bytes().iter().all(|byte| !byte.is_ascii_alphabetic())
    );
    let right_casing_neutral = matches!(
        right.literal,
        StringLiteral::Value(value) if value.as_bytes().iter().all(|byte| !byte.is_ascii_alphabetic())
    );
    let casing = match (left.casing, right.casing) {
        (StringCasing::Lowercase, StringCasing::Lowercase) => StringCasing::Lowercase,
        (StringCasing::Uppercase, StringCasing::Uppercase) => StringCasing::Uppercase,
        (casing, StringCasing::Unspecified) if right_casing_neutral => casing,
        (StringCasing::Unspecified, casing) if left_casing_neutral => casing,
        _ => StringCasing::Unspecified,
    };

    StringAtom { literal, casing, flags: left.flags.intersection(right.flags) }
}

#[inline]
fn is_numeric_string(input: &[u8]) -> bool {
    core::str::from_utf8(input).is_ok_and(|text| text.parse::<i64>().is_ok() || text.parse::<f64>().is_ok())
}

/// Drop string literals and add the broad `string` form when the
/// distinct-literal count exceeds `threshold`.
pub fn apply_string_literal_collapse(atoms: &mut Vec<Atom<'_>>, threshold: u16) {
    if atoms.contains(&STRING) {
        return;
    }

    let count = atoms
        .iter()
        .filter(|atom| matches!(atom, Atom::String(payload) if matches!(payload.literal, StringLiteral::Value(_))))
        .count();

    if count <= usize::from(threshold) {
        return;
    }

    atoms.retain(|atom| !matches!(atom, Atom::String(payload) if matches!(payload.literal, StringLiteral::Value(_))));
    let position = atoms.binary_search(&STRING).unwrap_or_else(|insertion| insertion);
    atoms.insert(position, STRING);
}
