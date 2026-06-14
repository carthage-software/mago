//! Scalar synthesis: `int | string | float | bool` → `scalar`.

use mago_allocator::Arena;
use mago_allocator::vec::Vec as ScratchVec;

use crate::ty::atom::Atom;
use crate::ty::well_known::BOOL;
use crate::ty::well_known::FLOAT;
use crate::ty::well_known::INT;
use crate::ty::well_known::SCALAR;
use crate::ty::well_known::STRING;

/// When the union contains all four general primitives (`int`, `string`,
/// `float`, `bool`), collapse them to `scalar`. Refined / literal
/// forms alone don't trigger the collapse: only the general
/// unspecified forms count. Other scalar atoms (literals,
/// refinements, class-like-strings) remain independent and are left
/// to subtype absorption.
pub fn apply_scalar_synthesis<S>(atoms: &mut ScratchVec<'_, Atom<'_>, S>)
where
    S: Arena,
{
    let has_int = atoms.contains(&INT);
    let has_string = atoms.contains(&STRING);
    let has_float = atoms.contains(&FLOAT);
    let has_bool = atoms.contains(&BOOL);
    if !(has_int && has_string && has_float && has_bool) {
        return;
    }

    atoms.retain(|atom| *atom != INT && *atom != STRING && *atom != FLOAT && *atom != BOOL);
    let position = atoms.binary_search(&SCALAR).unwrap_or_else(|insertion| insertion);
    atoms.insert(position, SCALAR);
}
