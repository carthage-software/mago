//! Per-family meet rules. Each submodule owns the intersection
//! algebra for one atom kind (or one cross-kind pair). The entry
//! `family_atom_meet` in the parent module dispatches on the input
//! kinds and delegates here.

pub(super) mod array;
pub(super) mod callable;
pub(super) mod generic;
pub(super) mod has_member;
pub(super) mod int;
pub(super) mod iterable;
pub(super) mod object;
pub(crate) mod string;
