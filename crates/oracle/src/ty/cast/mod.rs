//! PHP cast semantics.
//!
//! [`cast`] computes the result type of an explicit PHP cast operator -
//! `(int)`, `(float)`, `(string)`, `(bool)`, `(array)`, `(object)` -
//! applied to an arbitrary input [`Type`]. The result is a [`CastResult`]
//! pairing the post-cast type with cast flags that record whether the cast
//! lost information ([`CastFlag::Lossy`]) or could throw at runtime
//! ([`CastFlag::MayThrow`]).
//!
//! The implementation distributes over the input union and runs a per-atom
//! rule. Atom classifications combine via bitwise-or on the flags and union
//! on the result types.
//!
//! # Accuracy
//!
//! Literal preservation is precise where PHP's semantics are deterministic:
//! integer and string literals round-trip across `(int)` and `(string)`
//! losslessly, the falsy literals (`0`, `0.0`, `""`, `"0"`, `null`) collapse
//! to `false` under `(bool)`, and float literals truncate exactly under
//! `(int)`. `(array)` of a scalar or resource produces the single-element list
//! `list{T}` it actually yields, and `(string)` of an object consults the
//! symbol table for a `__toString` method - present means a lossless `string`, absent
//! means the cast may throw.
//!
//! Two cases stay at their sound limit rather than guessing: `(object)` of a
//! non-object yields `stdClass` (typing it as an anonymous `object{...}` shape
//! would drop the nominal `stdClass` identity that `instanceof` relies on),
//! and a non-literal `(int)`-of-float widens to `int` (no float range is
//! tracked to narrow against).

use mago_allocator::Arena;
use mago_flags::U8Flags;

use crate::symbol::SymbolTable;
use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::array::KnownElement;
use crate::ty::atom::payload::scalar::float::FloatAtom;
use crate::ty::atom::payload::scalar::int::IntAtom;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::builder::TypeBuilder;
use crate::ty::well_known;

/// One of PHP's six explicit cast operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CastTarget {
    Int,
    Float,
    String,
    Bool,
    Array,
    Object,
}

/// Result of [`cast`]: the post-cast type plus diagnostic flags.
#[derive(Debug, Clone, Copy)]
pub struct CastResult<'arena> {
    pub ty: Type<'arena>,
    pub flags: U8Flags<CastFlag>,
}

impl<'arena> CastResult<'arena> {
    #[inline]
    #[must_use]
    pub const fn lossless(ty: Type<'arena>) -> Self {
        Self { ty, flags: U8Flags::empty() }
    }

    #[inline]
    #[must_use]
    pub const fn lossy(ty: Type<'arena>) -> Self {
        Self { ty, flags: U8Flags::from_bits(CastFlag::Lossy as u8) }
    }

    #[inline]
    #[must_use]
    pub const fn may_throw(ty: Type<'arena>) -> Self {
        Self { ty, flags: U8Flags::from_bits(CastFlag::MayThrow as u8) }
    }
}

/// One cast diagnostic, carried as a `U8Flags<CastFlag>` on
/// [`CastResult::flags`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum CastFlag {
    /// The cast discarded information from the input (e.g. float
    /// truncation, non-numeric string to `int = 0`).
    Lossy = 1 << 0,
    /// The cast may emit an error or throw at runtime (e.g. casting an
    /// array to a string in PHP 8+, or an object lacking `__toString`).
    MayThrow = 1 << 1,
}

impl From<CastFlag> for u8 {
    fn from(flag: CastFlag) -> Self {
        flag as u8
    }
}

/// Cast `input` to `target`. Distributes over the input union: each atom is
/// cast individually, the resulting types are unioned, and the per-atom
/// flags are bit-or'd into a single flag set.
#[inline]
pub fn cast<'arena, S, A>(
    input: Type<'arena>,
    target: CastTarget,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> CastResult<'arena>
where
    S: Arena,
    A: Arena,
{
    let mut combined: Vec<Atom<'arena>> = Vec::new();
    let mut flags = U8Flags::empty();
    for atom in input.atoms {
        let outcome = cast_atom(*atom, target, symbols, builder);
        flags |= outcome.flags;
        combined.extend_from_slice(outcome.ty.atoms);
    }

    CastResult { ty: builder.union_of(&combined), flags }
}

#[inline]
fn cast_atom<'arena, S, A>(
    atom: Atom<'arena>,
    target: CastTarget,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> CastResult<'arena>
where
    S: Arena,
    A: Arena,
{
    match target {
        CastTarget::Int => cast_to_int(atom, builder),
        CastTarget::Float => cast_to_float(atom, builder),
        CastTarget::String => cast_to_string(atom, symbols, builder),
        CastTarget::Bool => cast_to_bool(atom, builder),
        CastTarget::Array => cast_to_array(atom, builder),
        CastTarget::Object => cast_to_object(atom, builder),
    }
}

#[inline]
fn cast_to_int<'arena, S, A>(atom: Atom<'arena>, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> CastResult<'arena>
where
    S: Arena,
    A: Arena,
{
    match atom {
        Atom::Int(_) => CastResult::lossless(builder.union_of(&[atom])),
        Atom::True => CastResult::lossless(builder.union_of(&[well_known::INT_ONE])),
        Atom::False | Atom::Null | Atom::Void => CastResult::lossless(builder.union_of(&[well_known::INT_ZERO])),
        Atom::Bool => CastResult::lossless(well_known::TYPE_INT),
        Atom::Float(payload) => match payload {
            FloatAtom::Literal(literal) => {
                let truncated = literal.value().trunc() as i64;

                CastResult::lossy(builder.union_of(&[Atom::int_literal(truncated)]))
            }
            FloatAtom::Unspecified | FloatAtom::UnspecifiedLiteral => CastResult::lossy(well_known::TYPE_INT),
        },
        Atom::String(payload) => match payload.literal {
            StringLiteral::Value(value) => {
                if let Some(parsed) = parse_php_int(value) {
                    CastResult::lossless(builder.union_of(&[Atom::int_literal(parsed)]))
                } else {
                    CastResult::lossy(builder.union_of(&[well_known::INT_ZERO]))
                }
            }
            StringLiteral::None | StringLiteral::Unspecified => CastResult::lossy(well_known::TYPE_INT),
        },
        Atom::Resource(_) => CastResult::lossless(well_known::TYPE_INT),
        Atom::Array(_) | Atom::List(_) => CastResult::lossy(well_known::TYPE_INT),
        Atom::Object(_)
        | Atom::Enum(_)
        | Atom::ObjectShape(_)
        | Atom::HasMethod(_)
        | Atom::HasProperty(_)
        | Atom::ObjectAny => CastResult::may_throw(well_known::TYPE_INT),
        _ => CastResult::lossy(well_known::TYPE_INT),
    }
}

#[inline]
fn cast_to_float<'arena, S, A>(atom: Atom<'arena>, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> CastResult<'arena>
where
    S: Arena,
    A: Arena,
{
    match atom {
        Atom::Float(_) => CastResult::lossless(builder.union_of(&[atom])),
        Atom::Int(payload) => match payload {
            IntAtom::Literal(value) => CastResult::lossless(builder.union_of(&[Atom::float_literal(value as f64)])),
            _ => CastResult::lossless(well_known::TYPE_FLOAT),
        },
        Atom::True => CastResult::lossless(builder.union_of(&[Atom::float_literal(1.0)])),
        Atom::False | Atom::Null | Atom::Void => CastResult::lossless(builder.union_of(&[Atom::float_literal(0.0)])),
        Atom::Bool => CastResult::lossless(well_known::TYPE_FLOAT),
        Atom::String(payload) => match payload.literal {
            StringLiteral::Value(value) => match parse_php_float(value) {
                Some(parsed) => CastResult::lossless(builder.union_of(&[Atom::float_literal(parsed)])),
                None => CastResult::lossy(builder.union_of(&[Atom::float_literal(0.0)])),
            },
            StringLiteral::None | StringLiteral::Unspecified => CastResult::lossy(well_known::TYPE_FLOAT),
        },
        Atom::Object(_)
        | Atom::Enum(_)
        | Atom::ObjectShape(_)
        | Atom::HasMethod(_)
        | Atom::HasProperty(_)
        | Atom::ObjectAny
        | Atom::Array(_)
        | Atom::List(_) => CastResult::may_throw(well_known::TYPE_FLOAT),
        _ => CastResult::lossy(well_known::TYPE_FLOAT),
    }
}

#[inline]
fn cast_to_string<'arena, S, A>(
    atom: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> CastResult<'arena>
where
    S: Arena,
    A: Arena,
{
    match atom {
        Atom::String(_) => CastResult::lossless(builder.union_of(&[atom])),
        Atom::Int(payload) => match payload {
            IntAtom::Literal(value) => {
                let literal = builder.string_literal(value.to_string().as_bytes());

                CastResult::lossless(builder.union_of(&[literal]))
            }
            _ => CastResult::lossless(well_known::TYPE_STRING),
        },
        Atom::Float(payload) => match payload {
            FloatAtom::Literal(literal) => {
                let rendered = builder.string_literal(format_php_float(literal.value()).as_bytes());

                CastResult::lossless(builder.union_of(&[rendered]))
            }
            _ => CastResult::lossless(well_known::TYPE_STRING),
        },
        Atom::True => {
            let literal = builder.string_literal(b"1");

            CastResult::lossless(builder.union_of(&[literal]))
        }
        Atom::False | Atom::Null | Atom::Void => CastResult::lossless(builder.union_of(&[well_known::EMPTY_STRING])),
        Atom::Bool | Atom::Resource(_) => CastResult::lossless(well_known::TYPE_STRING),
        Atom::Object(payload) if symbols.class_has_method(payload.name.id, b"__toString") => {
            CastResult::lossless(well_known::TYPE_STRING)
        }
        Atom::HasMethod(payload) if payload.method_name == b"__toString" => {
            CastResult::lossless(well_known::TYPE_STRING)
        }
        Atom::Object(_)
        | Atom::Enum(_)
        | Atom::ObjectShape(_)
        | Atom::HasMethod(_)
        | Atom::HasProperty(_)
        | Atom::ObjectAny
        | Atom::Array(_)
        | Atom::List(_) => CastResult::may_throw(well_known::TYPE_STRING),
        _ => CastResult::lossy(well_known::TYPE_STRING),
    }
}

#[inline]
fn cast_to_bool<'arena, S, A>(atom: Atom<'arena>, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> CastResult<'arena>
where
    S: Arena,
    A: Arena,
{
    match atom {
        Atom::True => CastResult::lossless(builder.union_of(&[well_known::TRUE])),
        Atom::False | Atom::Null | Atom::Void => CastResult::lossless(builder.union_of(&[well_known::FALSE])),
        Atom::Bool => CastResult::lossless(builder.union_of(&[atom])),
        Atom::Int(payload) => match payload {
            IntAtom::Literal(0) => CastResult::lossless(builder.union_of(&[well_known::FALSE])),
            IntAtom::Literal(_) => CastResult::lossless(builder.union_of(&[well_known::TRUE])),
            _ => CastResult::lossless(well_known::TYPE_BOOL),
        },
        Atom::Float(payload) => match payload {
            FloatAtom::Literal(literal) if literal.value() == 0.0 => {
                CastResult::lossless(builder.union_of(&[well_known::FALSE]))
            }
            FloatAtom::Literal(_) => CastResult::lossless(builder.union_of(&[well_known::TRUE])),
            _ => CastResult::lossless(well_known::TYPE_BOOL),
        },
        Atom::String(payload) => match payload.literal {
            StringLiteral::Value(value) => {
                let bytes = value;
                if bytes.is_empty() || bytes == b"0" {
                    CastResult::lossless(builder.union_of(&[well_known::FALSE]))
                } else {
                    CastResult::lossless(builder.union_of(&[well_known::TRUE]))
                }
            }
            StringLiteral::None | StringLiteral::Unspecified => CastResult::lossless(well_known::TYPE_BOOL),
        },
        Atom::Array(_) | Atom::List(_) => CastResult::lossless(well_known::TYPE_BOOL),
        Atom::Object(_)
        | Atom::Enum(_)
        | Atom::ObjectShape(_)
        | Atom::HasMethod(_)
        | Atom::HasProperty(_)
        | Atom::ObjectAny
        | Atom::Resource(_)
        | Atom::Callable(_) => CastResult::lossless(builder.union_of(&[well_known::TRUE])),
        _ => CastResult::lossless(well_known::TYPE_BOOL),
    }
}

#[inline]
fn cast_to_array<'arena, S, A>(atom: Atom<'arena>, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> CastResult<'arena>
where
    S: Arena,
    A: Arena,
{
    match atom {
        Atom::Array(_) | Atom::List(_) => CastResult::lossless(builder.union_of(&[atom])),
        Atom::Null | Atom::Void => CastResult::lossless(builder.union_of(&[well_known::EMPTY_ARRAY])),
        Atom::Int(_) | Atom::Float(_) | Atom::String(_) | Atom::True | Atom::False | Atom::Bool | Atom::Resource(_) => {
            let element = builder.union_of(&[atom]);
            let list = builder.sealed_list(&[KnownElement { index: 0, value: element, optional: false }], true);

            CastResult::lossless(builder.union_of(&[list]))
        }
        _ => CastResult::lossy(builder.union_of(&[well_known::ARRAY_KEY_MIXED])),
    }
}

#[inline]
fn cast_to_object<'arena, S, A>(atom: Atom<'arena>, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> CastResult<'arena>
where
    S: Arena,
    A: Arena,
{
    match atom {
        Atom::Object(_)
        | Atom::Enum(_)
        | Atom::ObjectShape(_)
        | Atom::HasMethod(_)
        | Atom::HasProperty(_)
        | Atom::ObjectAny => CastResult::lossless(builder.union_of(&[atom])),
        _ => {
            let std_class = builder.object_named(b"stdClass");

            CastResult::lossy(builder.union_of(&[std_class]))
        }
    }
}

/// Parse a string the way PHP's `(int)` does: leading whitespace + optional
/// sign + decimal digits, stopping at the first non-digit. Returns `None`
/// when no leading-decimal prefix exists; `Some(0)` is reserved for the
/// explicit literal "0".
#[inline]
fn parse_php_int(bytes: &[u8]) -> Option<i64> {
    let trimmed = bytes.trim_ascii_start();
    let mut index = 0;
    if matches!(trimmed.first(), Some(b'-' | b'+')) {
        index = 1;
    }

    let digits_start = index;
    while index < trimmed.len() && trimmed[index].is_ascii_digit() {
        index += 1;
    }

    if index == digits_start {
        return None;
    }

    core::str::from_utf8(&trimmed[..index]).ok()?.parse::<i64>().ok()
}

#[inline]
fn parse_php_float(bytes: &[u8]) -> Option<f64> {
    let trimmed = bytes.trim_ascii();
    if trimmed.is_empty() {
        return None;
    }

    core::str::from_utf8(trimmed).ok()?.parse::<f64>().ok()
}

#[inline]
fn format_php_float(value: f64) -> String {
    if value == value.trunc() && value.is_finite() { format!("{}", value as i64) } else { format!("{value}") }
}
