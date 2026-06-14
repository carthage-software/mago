//! Float-family join: literal-count collapse.

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;

use crate::ty::atom::Atom;
use crate::ty::atom::payload::scalar::float::FloatAtom;
use crate::ty::well_known::FLOAT;

/// Drop float literals and add the broad `float` form when the
/// distinct-literal count exceeds `threshold`.
pub fn apply_float_literal_collapse<S>(atoms: &mut ScratchVec<'_, Atom<'_>, S>, threshold: u16)
where
    S: Arena,
{
    if atoms.contains(&FLOAT) {
        return;
    }

    let count = atoms.iter().filter(|atom| matches!(atom, Atom::Float(FloatAtom::Literal(_)))).count();

    if count <= usize::from(threshold) {
        return;
    }

    atoms.retain(|atom| !matches!(atom, Atom::Float(FloatAtom::Literal(_))));
    let position = atoms.binary_search(&FLOAT).unwrap_or_else(|insertion| insertion);
    atoms.insert(position, FLOAT);
}
