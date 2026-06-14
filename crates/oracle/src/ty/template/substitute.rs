//! Capture-free substitution of template parameters in a [`Type`].
//!
//! Replaces every free occurrence of a template parameter inside `ty`
//! with the type the caller's closure supplies. The structural walk is
//! delegated to [`crate::transform::flat_map`]; this module only owns
//! the per-atom decision.
//!
//! The closure is parameterised on [`GenericParameterAtom`] rather
//! than on `(name, defining_entity)` pairs so the caller can inspect
//! the constraint or the qualifier when deciding what to substitute.
//! Returning `None` from the closure leaves the parameter in place;
//! returning `Some(replacement)` performs the rewrite.

use mago_allocator::Arena;

use crate::ty::Type;
use crate::ty::atom::Atom;
use crate::ty::atom::payload::generic_parameter::GenericParameterAtom;
use crate::ty::builder::TypeBuilder;
use crate::ty::transform;

/// Apply a substitution closure to every free template parameter in
/// `ty`.
///
/// `resolver` is consulted for each
/// [`GenericParameter`](Atom::GenericParameter) atom encountered during
/// the structural walk:
///
/// - `Some(replacement)`: the parameter is replaced; the
///   replacement's atoms flow into the surrounding union.
/// - `None`: the parameter is left in place. The closure may still
///   substitute inside the parameter's constraint by recursing if it
///   wants to; this module does not do so automatically.
///
/// Returns the same [`Type`] when nothing changed.
#[inline]
pub fn substitute<'arena, F, S, A>(
    ty: Type<'arena>,
    resolver: &F,
    builder: &mut TypeBuilder<'_, 'arena, S, A>,
) -> Type<'arena>
where
    F: Fn(&GenericParameterAtom<'arena>) -> Option<Type<'arena>>,
    S: Arena,
    A: Arena,
{
    transform::flat_map(ty, |atom| substitute_atom(atom, resolver), builder)
}

#[inline]
fn substitute_atom<'arena, F>(atom: Atom<'arena>, resolver: &F) -> Vec<Atom<'arena>>
where
    F: Fn(&GenericParameterAtom<'arena>) -> Option<Type<'arena>>,
{
    let Atom::GenericParameter(payload) = atom else {
        return vec![atom];
    };

    match resolver(payload) {
        Some(replacement) => replacement.atoms.to_vec(),
        None => vec![atom],
    }
}
