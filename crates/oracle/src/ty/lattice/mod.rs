//! The type lattice and its three relations.
//!
//! The type system forms a partially-ordered lattice. This module exposes
//! three operations on it:
//!
//! - [`refines`] - `a <: b` (every value of `a` is a value of `b`).
//! - [`generalizes`] - `a :> b` (every value of `b` is a value of `a`),
//!   the reverse of [`refines`].
//! - [`overlaps`] - `a ∩ b ≠ ∅` (there exists a value in both `a` and
//!   `b`). The boolean overlap question; the type-returning meet (greatest
//!   lower bound) lives in [`crate::ty::meet`].
//!
//! Each takes a [`World`](crate::world::World) (class hierarchy lookups,
//! member existence checks, template metadata), a [`LatticeOptions`] value
//! (caller-set knobs like `ignore_null`), a `&mut LatticeReport`
//! (diagnostics: the [`CoercionCause`] set and optional replacements), and
//! a [`TypeBuilder`](crate::ty::builder::TypeBuilder) for the intermediate and
//! replacement types the rules construct.
//!
//! Per-family rules live in [`family`]; each
//! [`AtomKind`](crate::ty::atom::kind::AtomKind) family owns its refinement and
//! overlap logic in a dedicated submodule.

pub mod family;

pub(crate) mod overlaps;
pub(crate) mod sealed;

mod options;
mod refines;
mod report;

pub use crate::ty::lattice::options::LatticeOptions;
pub use crate::ty::lattice::overlaps::is_uninhabited;
pub use crate::ty::lattice::overlaps::overlaps;
pub use crate::ty::lattice::refines::generalizes;
pub use crate::ty::lattice::refines::refines;
pub use crate::ty::lattice::report::CoercionCause;
pub use crate::ty::lattice::report::LatticeReport;

pub(crate) use crate::ty::lattice::refines::atom_admits_empty_container;
pub(crate) use crate::ty::lattice::refines::atom_is_empty_container;
pub(crate) use crate::ty::lattice::refines::atom_refines;
