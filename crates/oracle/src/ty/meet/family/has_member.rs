//! `HasMethod` and `HasProperty` family meet: compose into an
//! [`Intersected`](crate::ty::atom::kind::AtomKind::Intersected) wrapper.

use mago_allocator::Arena;

use crate::ty::atom::Atom;
use crate::ty::builder::TypeBuilder;

/// `HasMethod(m₁) ∧ HasMethod(m₂)`. When the names match, returns the
/// shared atom. Otherwise wraps both as conjuncts of an `Intersected`.
pub(in crate::ty::meet) fn has_method_meet<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    let (Atom::HasMethod(a_payload), Atom::HasMethod(b_payload)) = (a, b) else {
        return None;
    };

    if a_payload.method_name == b_payload.method_name {
        return Some(a);
    }

    Some(builder.intersected(a, &[b]))
}

/// `HasMethod(m) ∧ HasProperty(p)`: orthogonal predicates compose
/// through the [`Intersected`](crate::ty::atom::kind::AtomKind::Intersected)
/// wrapper.
pub(in crate::ty::meet) fn has_method_property_meet<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    Some(builder.intersected(a, &[b]))
}

/// `HasProperty(p₁) ∧ HasProperty(p₂)`; same structure as has-method.
pub(in crate::ty::meet) fn has_property_meet<'arena, S, A>(
    a: Atom<'arena>,
    b: Atom<'arena>,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Option<Atom<'arena>>
where
    S: Arena,
    A: Arena,
{
    let (Atom::HasProperty(a_payload), Atom::HasProperty(b_payload)) = (a, b) else {
        return None;
    };

    if a_payload.property_name == b_payload.property_name {
        return Some(a);
    }

    Some(builder.intersected(a, &[b]))
}
