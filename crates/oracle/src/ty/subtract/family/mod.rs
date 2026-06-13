//! Per-family subtract rules. Each submodule owns the difference
//! algebra for one atom kind (or one cross-kind pair). The entry
//! `family_atom_minus` in the parent dispatches on the input kinds
//! and delegates here.

pub(super) mod array;
pub(super) mod callable;
pub(super) mod dominator;
pub(super) mod generic;
pub(super) mod has_member;
pub(super) mod int;
pub(super) mod iterable;
pub(super) mod list;
pub(super) mod object;
pub(super) mod string;
