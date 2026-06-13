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
mod options;
pub(crate) mod overlaps;
mod refines;
mod report;
pub(crate) mod sealed;

pub use self::options::LatticeOptions;
pub use self::overlaps::is_uninhabited;
pub use self::overlaps::overlaps;
pub(crate) use self::refines::atom_admits_empty_container;
pub(crate) use self::refines::atom_is_empty_container;
pub(crate) use self::refines::atom_refines;
pub use self::refines::generalizes;
pub use self::refines::refines;
pub use self::report::CoercionCause;
pub use self::report::LatticeReport;
