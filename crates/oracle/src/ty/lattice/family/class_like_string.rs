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
//! object-family lattice (so all the world's ancestry / generic-arg
//! / variance rules apply).
//!
//! Cross-kind: a regular `String` input whose literal value is a valid
//! PHP class name is also accepted as a class-string, mirroring how
//! the runtime treats `"\\App\\Foo"` interchangeably with
//! `App\Foo::class`.

use mago_allocator::Arena;
use mago_flags::U8Flags;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::object::named::ObjectAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeKind;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringAtom;
use crate::ty::atom::payload::scalar::class_like_string::ClassLikeStringSpecifier;
use crate::ty::atom::payload::scalar::string::StringLiteral;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::name::Name;
use crate::ty::Type;
use crate::world::World;

#[inline]
pub fn refines<'arena, S, A, W>(
    input: Atom<'arena>,
    container: Atom<'arena>,
    world: &W,
    options: LatticeOptions,
    report: &mut LatticeReport<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> bool
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let Atom::ClassLikeString(container_payload) = container else {
        return false;
    };

    if matches!(container_payload.specifier, ClassLikeStringSpecifier::Any) {
        return matches_kind(input, container_payload.kind, world);
    }

    if !matches_kind(input, container_payload.kind, world) {
        return false;
    }

    let Some(container_target) = represented_type(container_payload, world, builder) else {
        return false;
    };

    let Some(input_target) = input_represented_type(input, world, builder) else {
        return false;
    };

    crate::ty::lattice::refines(input_target, container_target, world, options, report, builder)
}

/// String-literal inputs must name a syntactically valid class. When the
/// world classifies the name, an exact kind match is required; when the
/// world is silent, the name is accepted - an unknown name stays
/// permissive (open-world).
#[inline]
fn matches_kind<'arena, W>(input: Atom<'arena>, container_kind: ClassLikeKind, world: &W) -> bool
where
    W: World<'arena>,
{
    if let Atom::String(input_payload) = input {
        let StringLiteral::Value(value) = input_payload.literal else {
            return false;
        };

        if !is_valid_class_name(value.as_bytes()) {
            return false;
        }

        return match world.class_like_kind(value) {
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
fn represented_type<'arena, S, A, W>(
    payload: &ClassLikeStringAtom<'arena>,
    world: &W,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Type<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    match payload.specifier {
        ClassLikeStringSpecifier::Any => None,
        ClassLikeStringSpecifier::Literal { value } => Some(name_as_object_type(value, payload.kind, world, builder)),
        ClassLikeStringSpecifier::OfType { constraint } | ClassLikeStringSpecifier::Generic { constraint } => {
            Some(constraint)
        }
    }
}

#[inline]
fn input_represented_type<'arena, S, A, W>(
    input: Atom<'arena>,
    world: &W,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Type<'arena>>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    if let Atom::ClassLikeString(input_payload) = input {
        return represented_type(input_payload, world, builder);
    }

    let Atom::String(input_payload) = input else {
        return None;
    };

    let StringLiteral::Value(value) = input_payload.literal else {
        return None;
    };

    if !is_valid_class_name(value.as_bytes()) {
        return None;
    }

    let kind = kind_from_world(value, world);

    Some(name_as_object_type(value, kind, world, builder))
}

#[inline]
fn name_as_object_type<'arena, S, A, W>(
    name: Name<'arena>,
    kind: ClassLikeKind,
    _world: &W,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Type<'arena>
where
    S: Arena,
    A: Arena,
    W: World<'arena>,
{
    let atom = match kind {
        ClassLikeKind::Enum => builder.enum_any(name.as_bytes()),
        ClassLikeKind::Class | ClassLikeKind::Interface | ClassLikeKind::Trait => {
            builder.object(ObjectAtom { name, type_arguments: None, flags: U8Flags::empty() })
        }
    };

    builder.union_of(&[atom])
}

#[inline]
fn kind_from_world<'arena, W>(name: Name<'_>, world: &W) -> ClassLikeKind
where
    W: World<'arena>,
{
    world.class_like_kind(name).unwrap_or(ClassLikeKind::Class)
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
