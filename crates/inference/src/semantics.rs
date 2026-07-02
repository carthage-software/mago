use mago_allocator::Arena;
use mago_allocator::vec::Vec;
use mago_oracle::ty::Atom;
use mago_oracle::ty::Type;
use mago_oracle::ty::atom::payload::array::ArrayFlag;
use mago_oracle::ty::atom::payload::array::ArrayKey;
use mago_oracle::ty::atom::payload::array::KnownItem;
use mago_oracle::ty::atom::payload::array::ListFlag;
use mago_oracle::ty::atom::payload::scalar::float::FloatAtom;
use mago_oracle::ty::atom::payload::scalar::int::IntAtom;
use mago_oracle::ty::atom::payload::scalar::string::StringLiteral;
use mago_oracle::ty::predicates::is_falsy;
use mago_oracle::ty::predicates::is_truthy;
use mago_oracle::ty::well_known::TYPE_MIXED;

/// A PHP number: arithmetic on two of these is exact.
#[derive(Clone, Copy)]
pub(crate) enum Number {
    Int(i64),
    Float(f64),
}

impl Number {
    pub(crate) fn as_f64(self) -> f64 {
        match self {
            Number::Int(value) => value as f64,
            Number::Float(value) => value,
        }
    }

    /// PHP truncates toward zero when an operand is cast to int (modulo, bitwise).
    pub(crate) fn to_int(self) -> i64 {
        match self {
            Number::Int(value) => value,
            Number::Float(value) => value as i64,
        }
    }

    pub(crate) fn is_zero(self) -> bool {
        match self {
            Number::Int(value) => value == 0,
            Number::Float(value) => value == 0.0,
        }
    }
}

/// Reads a fully-known PHP number out of a single-atom type, coercing the same
/// way PHP arithmetic does: `bool`/`null` to int, and numeric strings to their
/// numeric value (including PHP's leading-numeric prefix rule).
pub(crate) fn number_of(ty: Type<'_>) -> Option<Number> {
    let [atom] = ty.atoms else {
        return None;
    };

    Some(match atom {
        Atom::Int(IntAtom::Literal(value)) => Number::Int(*value),
        Atom::Float(FloatAtom::Literal(value)) => Number::Float(value.0.into_inner()),
        Atom::True => Number::Int(1),
        Atom::False | Atom::Null => Number::Int(0),
        Atom::String(string) => match string.literal {
            StringLiteral::Value(value) => parse_php_number(value)?,
            _ => return None,
        },
        _ => return None,
    })
}

/// Reads *every* atom of `ty` as a fully-known PHP number into `out`, coercing
/// the same way [`number_of`] does. Returns `false` (leaving `out` partly
/// filled) when `ty` is empty or any atom is not statically a single number, so
/// the type cannot be enumerated for distributed arithmetic.
pub(crate) fn numbers_of<A>(ty: Type<'_>, out: &mut Vec<'_, Number, A>) -> bool
where
    A: Arena,
{
    if ty.atoms.is_empty() {
        return false;
    }

    for atom in ty.atoms {
        let number = match atom {
            Atom::Int(IntAtom::Literal(value)) => Number::Int(*value),
            Atom::Float(FloatAtom::Literal(value)) => Number::Float(value.0.into_inner()),
            Atom::True => Number::Int(1),
            Atom::False | Atom::Null => Number::Int(0),
            Atom::String(string) => match string.literal {
                StringLiteral::Value(value) => match parse_php_number(value) {
                    Some(number) => number,
                    None => return false,
                },
                _ => return false,
            },
            _ => return false,
        };

        out.push(number);
    }

    true
}

/// The runtime truthiness of `ty`, or `None` when it cannot be decided statically.
pub(crate) fn truthiness(ty: Type<'_>) -> Option<bool> {
    if is_truthy(ty) {
        Some(true)
    } else if is_falsy(ty) {
        Some(false)
    } else {
        None
    }
}

/// A single-atom array or list type.
pub(crate) fn is_array_type(ty: Type<'_>) -> bool {
    matches!(ty.atoms, [Atom::Array(_) | Atom::List(_)])
}

/// Any value of `ty` could be an array at runtime (a concrete array/list, or a
/// `mixed` that might hold one).
pub(crate) fn could_be_array(ty: Type<'_>) -> bool {
    ty.atoms.iter().any(|atom| matches!(atom, Atom::Array(_) | Atom::List(_) | Atom::Mixed(_)))
}

/// Reads the fully-known entries of a sealed array/list shape into `out`, in
/// order. Returns `false` (and leaves `out` partly filled) when `ty` is not a
/// single closed shape — an open rest type means the contents are not fully
/// known.
pub(crate) fn collect_closed_array<'arena, A>(ty: Type<'arena>, out: &mut Vec<'_, KnownItem<'arena>, A>) -> bool
where
    A: Arena,
{
    let [atom] = ty.atoms else {
        return false;
    };

    match atom {
        Atom::List(list) => {
            if !list.element_type.is_never() {
                return false;
            }

            if let Some(known) = list.known_elements {
                for element in known {
                    out.push(KnownItem {
                        key: ArrayKey::Int(i64::from(element.index)),
                        value: element.value,
                        optional: element.optional,
                    });
                }
            }

            true
        }
        Atom::Array(array) => {
            if !array.is_sealed() {
                return false;
            }

            if let Some(items) = array.known_items {
                out.extend_from_slice(items);
            }

            true
        }
        _ => false,
    }
}

/// Collects the value-atom contributions of an array/list type into `out`,
/// returning whether the type is guaranteed non-empty. A non-array type
/// contributes `mixed` (an unknown array could hold anything).
pub(crate) fn collect_array_values<'arena, A>(ty: Type<'arena>, out: &mut Vec<'_, Atom<'arena>, A>) -> bool
where
    A: Arena,
{
    let [atom] = ty.atoms else {
        out.extend_from_slice(TYPE_MIXED.atoms);
        return false;
    };

    match atom {
        Atom::List(list) => {
            let mut non_empty = list.flags.contains(ListFlag::NonEmpty);
            if let Some(known) = list.known_elements {
                for element in known {
                    out.extend_from_slice(element.value.atoms);
                    non_empty |= !element.optional;
                }
            }
            if !list.element_type.is_never() {
                out.extend_from_slice(list.element_type.atoms);
            }

            non_empty
        }
        Atom::Array(array) => {
            let mut non_empty = array.flags.contains(ArrayFlag::NonEmpty);
            if let Some(items) = array.known_items {
                for item in items {
                    out.extend_from_slice(item.value.atoms);
                    non_empty |= !item.optional;
                }
            }
            if let Some(value) = array.value_param {
                out.extend_from_slice(value.atoms);
            }

            non_empty
        }
        _ => {
            out.extend_from_slice(TYPE_MIXED.atoms);
            false
        }
    }
}

pub(crate) fn literal_string_bytes(ty: Type<'_>) -> Option<&[u8]> {
    let [Atom::String(string)] = ty.atoms else {
        return None;
    };

    match string.literal {
        StringLiteral::Value(value) => Some(value),
        _ => None,
    }
}

/// Appends `ty`'s PHP string form to `bytes`, returning `false` when the value
/// cannot be folded to an exact string (non-literal, or a float whose exact
/// PHP formatting we don't replicate).
pub(crate) fn append_string<A>(bytes: &mut Vec<'_, u8, A>, ty: Type<'_>) -> bool
where
    A: Arena,
{
    let [atom] = ty.atoms else {
        return false;
    };

    match atom {
        Atom::String(string) => match string.literal {
            StringLiteral::Value(value) => bytes.extend_from_slice(value),
            _ => return false,
        },
        Atom::Int(IntAtom::Literal(value)) => {
            let mut buffer = itoa::Buffer::new();
            bytes.extend_from_slice(buffer.format(*value).as_bytes());
        }
        Atom::True => bytes.push(b'1'),
        Atom::False | Atom::Null => {}
        Atom::Array(_) | Atom::List(_) => bytes.extend_from_slice(b"Array"),
        _ => return false,
    }

    true
}

/// `===`: type-and-value identity, so numeric strings are *not* coerced. Two
/// concrete literals of different kinds (e.g. `1 === 1.0`) are never identical.
pub(crate) fn fold_identical(left: Type<'_>, right: Type<'_>) -> Option<bool> {
    let ([left], [right]) = (left.atoms, right.atoms) else {
        return None;
    };

    match (left, right) {
        (Atom::Int(IntAtom::Literal(left)), Atom::Int(IntAtom::Literal(right))) => Some(left == right),
        (Atom::Float(FloatAtom::Literal(left)), Atom::Float(FloatAtom::Literal(right))) => Some(left == right),
        (Atom::String(left), Atom::String(right)) => match (left.literal, right.literal) {
            (StringLiteral::Value(left), StringLiteral::Value(right)) => Some(left == right),
            _ => None,
        },
        (Atom::True, Atom::True) | (Atom::False, Atom::False) | (Atom::Null, Atom::Null) => Some(true),
        _ if is_concrete_literal(left) && is_concrete_literal(right) => Some(false),
        _ => None,
    }
}

fn is_concrete_literal(atom: &Atom<'_>) -> bool {
    matches!(
        atom,
        Atom::Int(IntAtom::Literal(_)) | Atom::Float(FloatAtom::Literal(_)) | Atom::True | Atom::False | Atom::Null
    ) || matches!(atom, Atom::String(string) if matches!(string.literal, StringLiteral::Value(_)))
}

/// Parses a PHP numeric string: optional surrounding ASCII whitespace, optional
/// sign, integer/float syntax, and PHP's "leading-numeric" prefix rule (e.g.
/// `"123abc"` is `123`, `"0x1A"` is `0`). Returns `None` when no numeric prefix
/// exists.
pub(crate) fn parse_php_number(bytes: &[u8]) -> Option<Number> {
    let trimmed = bytes.trim_ascii();
    if trimmed.is_empty() {
        return None;
    }

    let mut index = 0;
    let mut is_float = false;

    if matches!(trimmed.first(), Some(b'+' | b'-')) {
        index += 1;
    }

    let integer_start = index;
    while matches!(trimmed.get(index), Some(byte) if byte.is_ascii_digit()) {
        index += 1;
    }
    let has_integer_digits = index > integer_start;

    let mut has_fraction_digits = false;
    if matches!(trimmed.get(index), Some(b'.')) {
        is_float = true;
        index += 1;
        let fraction_start = index;
        while matches!(trimmed.get(index), Some(byte) if byte.is_ascii_digit()) {
            index += 1;
        }
        has_fraction_digits = index > fraction_start;
    }

    if !has_integer_digits && !has_fraction_digits {
        return None;
    }

    if matches!(trimmed.get(index), Some(b'e' | b'E')) {
        let mut exponent = index + 1;
        if matches!(trimmed.get(exponent), Some(b'+' | b'-')) {
            exponent += 1;
        }
        let exponent_start = exponent;
        while matches!(trimmed.get(exponent), Some(byte) if byte.is_ascii_digit()) {
            exponent += 1;
        }
        if exponent > exponent_start {
            is_float = true;
            index = exponent;
        }
    }

    let prefix = std::str::from_utf8(&trimmed[..index]).ok()?;

    if is_float {
        return prefix.parse::<f64>().ok().map(Number::Float);
    }

    match prefix.parse::<i64>() {
        Ok(value) => Some(Number::Int(value)),
        Err(_) => prefix.parse::<f64>().ok().map(Number::Float),
    }
}
