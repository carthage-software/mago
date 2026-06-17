//! Compatibility relations: can two types refer to the same value?
//!
//! Two questions, two functions:
//!
//! - [`statically_compatible`]: at the type-system level, does there exist
//!   a runtime value the type system admits as inhabiting both types?
//!   Identical to [`crate::ty::lattice::overlaps`]; re-exported here so callers
//!   can find both relations in one place.
//! - [`runtime_compatible`]: at the PHP runtime level, could a single value
//!   be a member of both types after PHP erases the information the type
//!   system tracks but the runtime does not (object generic arguments,
//!   intersection conjuncts beyond the head class, etc.)? More permissive
//!   than [`statically_compatible`].
//!
//! Concrete differences:
//!
//! - `Cell<int>` vs `Cell<string>` - statically disjoint under invariance,
//!   runtime-compatible because PHP cannot tell two `Cell` instances apart
//!   by their generic argument.
//! - `Foo&Bar` vs `Foo` - runtime-compatible: an instance of `Foo&Bar` is
//!   also an instance of `Foo` for `instanceof`.
//! - `int` vs `string` - incompatible under both relations.
//!
//! Both functions write into a [`LatticeReport`] for parity with the rest
//! of the lattice surface, although the runtime variant currently records
//! nothing.

use mago_allocator::Arena;

use crate::path::Path;
use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::kind::AtomKind;
use crate::ty::builder::TypeBuilder;
use crate::ty::lattice;
use crate::ty::lattice::LatticeOptions;
use crate::ty::lattice::LatticeReport;
use crate::ty::well_known;
use crate::world::World;

/// Static compatibility: a value the type system admits in `a` is also
/// admitted in `b`. Equivalent to [`lattice::overlaps`].
#[inline]
pub fn statically_compatible<'arena, S, A, W>(
    a: Type<'arena>,
    b: Type<'arena>,
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
    lattice::overlaps(a, b, world, options, report, builder)
}

/// Runtime compatibility: a single PHP runtime value could inhabit both
/// `a` and `b` once the runtime has erased the information the type system
/// tracks but the PHP engine does not.
///
/// More permissive than [`statically_compatible`]: same-class objects with
/// disjoint generic arguments are compatible, and intersection conjuncts
/// beyond the head class are ignored.
#[inline]
pub fn runtime_compatible<'arena, S, A, W>(
    a: Type<'arena>,
    b: Type<'arena>,
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
    a.atoms
        .iter()
        .any(|left| b.atoms.iter().any(|right| atom_runtime_compatible(*left, *right, world, options, report, builder)))
}

#[inline]
fn atom_runtime_compatible<'arena, S, A, W>(
    a: Atom<'arena>,
    b: Atom<'arena>,
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
    if a == well_known::NEVER || b == well_known::NEVER {
        return false;
    }

    if a == b {
        return true;
    }

    if a == well_known::MIXED || b == well_known::MIXED || a == well_known::PLACEHOLDER || b == well_known::PLACEHOLDER
    {
        return true;
    }

    let a_object = is_object_family_atom(a);
    let b_object = is_object_family_atom(b);
    if a_object && b_object {
        return objects_runtime_compatible(a, b, world);
    }

    if a_object != b_object {
        return false;
    }

    let a_type = builder.union_of(&[a]);
    let b_type = builder.union_of(&[b]);

    lattice::overlaps(a_type, b_type, world, options, report, builder)
}

/// `true` iff some nominal class on one side is related (either direction
/// of `descends_from`) to some nominal class on the other. An empty
/// nominal set on a side means "any class", which is compatible with
/// anything in the family.
#[inline]
fn objects_runtime_compatible<'arena, W>(a: Atom<'arena>, b: Atom<'arena>, world: &W) -> bool
where
    W: World<'arena>,
{
    let a_classes = nominal_classes(a);
    let b_classes = nominal_classes(b);

    if a_classes.is_empty() || b_classes.is_empty() {
        return true;
    }

    a_classes.iter().any(|a_class| {
        b_classes
            .iter()
            .any(|b_class| world.descends_from(a_class.id, b_class.id) || world.descends_from(b_class.id, a_class.id))
    })
}

/// Collect the nominal class names an object-family atom identifies at
/// runtime. Empty when the atom is purely structural (`object`,
/// `object{...}`, `has-method`, `has-property`) and therefore matches any
/// class.
#[inline]
fn nominal_classes(atom: Atom<'_>) -> Vec<Path<'_>> {
    match atom {
        Atom::Object(payload) => vec![payload.name],
        Atom::Enum(payload) => vec![payload.name],
        Atom::Intersected(payload) => {
            let mut collected = nominal_classes(*payload.head);
            for conjunct in payload.conjuncts {
                collected.extend(nominal_classes(*conjunct));
            }

            collected
        }
        _ => Vec::new(),
    }
}

#[inline]
const fn is_object_family(kind: AtomKind) -> bool {
    matches!(
        kind,
        AtomKind::Object
            | AtomKind::Enum
            | AtomKind::ObjectShape
            | AtomKind::HasMethod
            | AtomKind::HasProperty
            | AtomKind::ObjectAny
    )
}

#[inline]
fn is_object_family_atom(atom: Atom<'_>) -> bool {
    if is_object_family(atom.kind()) {
        return true;
    }

    if let Atom::Intersected(payload) = atom {
        return is_object_family(payload.head.kind());
    }

    false
}
