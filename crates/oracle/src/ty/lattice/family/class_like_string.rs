//! Class-like-string family.
//!
//! `class-string`, `interface-string`, `enum-string`, `trait-string`,
//! plus refined forms (`class-string<Foo>`, `class-string<T>`,
//! `class-string<T of B>`, the literal `"App\\Foo"` typed as a class-string).
//!
//! Distinct kinds are disjoint: `class-string` is not a subtype of
//! `interface-string`, etc. Within a kind, the rule is "input fits
//! container iff the class the input names refines the class the
//! container expects". A literal class-string and a refined
//! `class-string<C>` therefore both reduce to "compare the named
//! object against the constraint", which routes through the regular
//! object-family lattice (so all the symbol table's ancestry / generic-arg
//! / variance rules apply).
//!
//! Cross-kind: a regular `String` input whose literal value is a valid
//! PHP class name is also accepted as a class-string, mirroring how
//! the runtime treats `"\\App\\Foo"` interchangeably with
//! `App\Foo::class`.

use mago_allocator::Arena;
use mago_flags::U8Flags;

use crate::id::SymbolId;
use crate::path::Path;
use crate::symbol::SymbolTable;
use crate::symbol::class_like::ClassLikeKind;
use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::object::named::ObjectAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;

#[inline]
pub fn refines<'arena, S, A>(
    input: Atom<'arena>,
    container: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
{
    let Atom::ClassLikeString(container_payload) = container else {
        return false;
    };

    if matches!(container_payload.specifier, ClassLikeStringSpecifier::Any) {
        return matches_kind(input, container_payload.kind, symbols);
    }

    if !matches_kind(input, container_payload.kind, symbols) {
        return false;
    }

    let Some(container_target) = represented_type(container_payload, symbols, builder) else {
        return false;
    };

    let Some(input_target) = input_represented_type(input, symbols, builder) else {
        return false;
    };

    crate::ty::lattice::refines(input_target, container_target, symbols, options, report, builder)
}

/// String-literal inputs must name a syntactically valid class. When the
/// symbol table classifies the name, an exact kind match is required; when the
/// symbol table is silent, the name is accepted - an unknown name stays
/// permissive (the hierarchy is open).
#[inline]
fn matches_kind<'arena, A>(input: Atom<'arena>, container_kind: ClassLikeKind, symbols: &SymbolTable<'arena, A>) -> bool
where
    A: Arena,
{
    if let Atom::String(input_payload) = input {
        let StringLiteral::Value(value) = input_payload.literal else {
            return false;
        };

        if !is_valid_class_name(value) {
            return false;
        }

        return match symbols.class_like_kind(SymbolId::class_like(value)) {
            Some(kind) => kind == container_kind,
            None => true,
        };
    }

    let Atom::ClassLikeString(input_payload) = input else {
        return false;
    };

    input_payload.kind == container_kind
}

#[inline]
fn represented_type<'arena, S, A>(
    payload: &ClassLikeStringAtom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Type<'arena>>
where
    S: Arena,
    A: Arena,
{
    match payload.specifier {
        ClassLikeStringSpecifier::Any => None,
        ClassLikeStringSpecifier::Literal { value } => Some(name_as_object_type(value, payload.kind, symbols, builder)),
        ClassLikeStringSpecifier::OfType { constraint } | ClassLikeStringSpecifier::Generic { constraint } => {
            Some(constraint)
        }
    }
}

#[inline]
fn input_represented_type<'arena, S, A>(
    input: Atom<'arena>,
    symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Type<'arena>>
where
    S: Arena,
    A: Arena,
{
    if let Atom::ClassLikeString(input_payload) = input {
        return represented_type(input_payload, symbols, builder);
    }

    let Atom::String(input_payload) = input else {
        return None;
    };

    let StringLiteral::Value(value) = input_payload.literal else {
        return None;
    };

    if !is_valid_class_name(value) {
        return None;
    }

    let name = builder.intern_class_like_path(value);
    let kind = kind_from_symbols(name, symbols);

    Some(name_as_object_type(name, kind, symbols, builder))
}

#[inline]
fn name_as_object_type<'arena, S, A>(
    name: Path<'arena>,
    kind: ClassLikeKind,
    _symbols: &SymbolTable<'arena, A>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
{
    let atom = match kind {
        ClassLikeKind::Enum => builder.enum_atom(name.as_bytes()),
        ClassLikeKind::Class | ClassLikeKind::Interface | ClassLikeKind::Trait => {
            builder.object(ObjectAtom { name, type_arguments: None, flags: U8Flags::empty() })
        }
    };

    builder.union_of(&[atom])
}

#[inline]
fn kind_from_symbols<A>(name: Path<'_>, symbols: &SymbolTable<'_, A>) -> ClassLikeKind
where
    A: Arena,
{
    symbols.class_like_kind(name.id).unwrap_or(ClassLikeKind::Class)
}

/// Validate that `bytes` is a syntactically well-formed PHP class name
/// (`Foo`, `\Foo`, `Foo\Bar`, `App\Service\Logger`, …). Used to reject
/// arbitrary string literals that don't look like class names before
/// treating them as class-strings.
#[inline]
fn is_valid_class_name(bytes: &[u8]) -> bool {
    let length = bytes.len();
    if length == 0 || bytes[length - 1] == b'\\' {
        return false;
    }

    let mut index = usize::from(bytes[0] == b'\\');
    if index >= length {
        return false;
    }

    let mut part_start = true;
    while index < length {
        let byte = bytes[index];
        if byte == b'\\' {
            if part_start {
                return false;
            }

            part_start = true;
        } else if part_start {
            if !(byte.is_ascii_alphabetic() || byte == b'_') {
                return false;
            }

            part_start = false;
        } else if !(byte.is_ascii_alphanumeric() || byte == b'_' || byte >= 0x80) {
            return false;
        }

        index += 1;
    }

    !part_start
}
