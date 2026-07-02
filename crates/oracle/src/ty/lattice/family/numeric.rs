//! `numeric` container: `int | float | numeric-string | numeric literals`.
//!
//! A general `string` is NOT numeric (only `numeric-string` and string
//! literals that parse as numbers are).

use crate::ty::atom::Atom;
use crate::ty::lattice::family::string;

#[inline]
#[must_use]
pub fn refines(input: Atom<'_>, _container: Atom<'_>) -> bool {
    match input {
        Atom::Int(_) | Atom::Float(_) => true,
        Atom::String(payload) => string::input_is_numeric(*payload),
        _ => false,
    }
}
