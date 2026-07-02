//! The PHP type system: representation, comparison, and lattice primitives
//! for static analysis tools.
//!
//! Every type is arena-allocated and lifetime-explicit: a [`Type<'arena>`] is
//! a canonical union of [`Atom<'arena>`]s whose payloads live behind plain
//! references in the arena that built them. There is no global state; the
//! [`TypeBuilder`] owns construction and hash-conses
//! payloads so that, within one arena, structural equality coincides with
//! pointer equality.
//!
//! Lifetimes encode ownership layering: a long-lived shared arena holds
//! signature types, shorter-lived per-file arenas hold inference results, and
//! covariance lets file-local types embed `&'shared` atoms while the borrow
//! checker rejects the reverse direction.
//!
//! The forward direction - shared atoms embedding into file types - is plain
//! covariance:
//!
//! ```
//! use mago_allocator::LocalArena;
//! use mago_oracle::ty::TypeBuilder;
//! use mago_oracle::ty::well_known;
//!
//! let shared_arena = LocalArena::new();
//! let shared_scratch = LocalArena::new();
//! let mut shared_builder = TypeBuilder::new(&shared_arena, &shared_scratch);
//! let collection = shared_builder.object_named(b"Collection");
//!
//! let file_arena = LocalArena::new();
//! let file_scratch = LocalArena::new();
//! let mut file_builder = TypeBuilder::new(&file_arena, &file_scratch);
//! let nullable = file_builder.union_of(&[well_known::NULL, collection]);
//! assert_eq!(nullable.to_string(), "Collection|null");
//! ```
//!
//! The reverse - a shared type capturing file-arena data that then outlives
//! the file - is a compile error, not a runtime hazard:
//!
//! ```compile_fail
//! use mago_allocator::LocalArena;
//! use mago_oracle::ty::Type;
//! use mago_oracle::ty::TypeBuilder;
//!
//! let shared_arena = LocalArena::new();
//! let shared_scratch = LocalArena::new();
//! let mut shared_builder = TypeBuilder::new(&shared_arena, &shared_scratch);
//!
//! let escaped: Type<'_> = {
//!     let file_arena = LocalArena::new();
//!     let file_scratch = LocalArena::new();
//!     let mut file_builder = TypeBuilder::new(&file_arena, &file_scratch);
//!     let file_local = file_builder.object_named(b"FileLocal");
//!     let file_type = file_builder.union_of(&[file_local]);
//!
//!     shared_builder.union_of(file_type.atoms)
//! };
//!
//! let still_alive = shared_builder.union_of(&[]);
//! assert_eq!(escaped.atoms.len(), 1);
//! ```

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

#[cfg(feature = "serde")]
use serde::Serialize;

use mago_allocator::Arena;
use mago_allocator::copy::CopyInto;
use mago_allocator::copy::copy_slice_into;
use mago_flags::U16Flags;

pub use crate::ty::atom::Atom;
pub use crate::ty::atom::kind::AtomKind;
pub use crate::ty::atom::set::AtomKindSet;
pub use crate::ty::builder::TypeBuilder;
pub use crate::ty::builder::union_buffer::UnionBuffer;
pub use crate::ty::flags::FlowFlag;

pub mod atom;
pub mod builder;
pub mod cast;
pub mod compatibility;
pub mod expand;
pub mod flags;
pub mod hierarchy;
pub mod inspect;
pub mod join;
pub mod lattice;
pub mod meet;
pub mod predicates;
pub mod serialize;
pub mod subtract;
pub mod template;
pub mod transform;
pub mod well_known;
pub mod widen;

mod layout;

/// A union of one or more [`Atom`]s.
///
/// `atoms` is sorted in canonical (kind-first structural) order and
/// deduplicated; the [`TypeBuilder`](crate::ty::builder::TypeBuilder) is the only
/// sanctioned constructor outside of `'static` well-known values. `kinds` is
/// the precomputed set of atom kinds present in the union.
///
/// An empty union is `never`. Equality is structural with two fast paths:
/// differing kind masks reject in one comparison, and a shared atom-slice
/// allocation (the within-builder consing guarantee) accepts without
/// walking the atoms.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialOrd, Ord)]
pub struct Type<'arena> {
    pub atoms: &'arena [Atom<'arena>],
    pub kinds: AtomKindSet,
}

impl PartialEq for Type<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if self.kinds != other.kinds {
            return false;
        }

        self.ptr_eq(other) || self.atoms == other.atoms
    }
}

/// A [`Type`] paired with its flow state: provenance flags and one byte of
/// consumer-defined metadata that the type system itself never inspects.
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Typed<'arena> {
    pub ty: Type<'arena>,
    pub flags: U16Flags<FlowFlag>,
    pub meta: u8,
}

/// The kind set of a canonical atom slice. `const` so `'static` well-known
/// types can precompute their masks at compile time.
#[must_use]
pub const fn kind_set_of(atoms: &[Atom<'_>]) -> AtomKindSet {
    let mut kinds = AtomKindSet::EMPTY;
    let mut index = 0;
    while index < atoms.len() {
        kinds = kinds.with(atoms[index].kind());
        index += 1;
    }

    kinds
}

impl<'arena> Type<'arena> {
    /// Wrap an already-canonical atom slice: sorted, deduplicated, well-known
    /// singletons normalized. The [`TypeBuilder`](crate::ty::builder::TypeBuilder)
    /// upholds this; direct use is reserved for `'static` constants.
    #[inline]
    #[must_use]
    pub const fn from_canonical_atoms(atoms: &'arena [Atom<'arena>]) -> Self {
        Self { atoms, kinds: kind_set_of(atoms) }
    }

    /// `true` for the canonical `never` (the single-atom `[Never]` union the
    /// builder produces for empty input) and for a raw empty atom slice.
    #[inline]
    #[must_use]
    pub const fn is_never(&self) -> bool {
        match self.atoms {
            [] => true,
            [atom] => matches!(atom, Atom::Never),
            _ => false,
        }
    }

    #[inline]
    #[must_use]
    pub const fn is_single(&self) -> bool {
        self.atoms.len() == 1
    }

    #[inline]
    #[must_use]
    pub const fn is_union(&self) -> bool {
        self.atoms.len() > 1
    }

    #[inline]
    #[must_use]
    pub const fn contains_kind(&self, kind: AtomKind) -> bool {
        self.kinds.contains(kind)
    }

    /// `true` iff `self` and `other` share the same atom slice allocation.
    ///
    /// Within one builder this is content equality (the builder
    /// hash-conses); across arenas fall back to `==`.
    #[inline]
    #[must_use]
    pub fn ptr_eq(&self, other: &Type<'_>) -> bool {
        std::ptr::eq(self.atoms.as_ptr().cast::<u8>(), other.atoms.as_ptr().cast::<u8>())
            && self.atoms.len() == other.atoms.len()
    }

    /// Debug-build invariant check: the atom slice is strictly sorted (no
    /// duplicates) and the kind mask matches the atoms. The
    /// [`TypeBuilder`](crate::ty::builder::TypeBuilder) asserts this at every
    /// exit; direct [`from_canonical_atoms`](Self::from_canonical_atoms)
    /// users can call it themselves.
    #[inline]
    pub fn assert_canonical(&self) {
        debug_assert!(self.atoms.is_sorted_by(|left, right| left < right), "atoms must be strictly sorted");
        debug_assert!(self.kinds == kind_set_of(self.atoms), "kind mask must match the atoms");
    }
}

impl Display for Type<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let count = self.atoms.len();
        if count == 0 {
            return f.write_str("never");
        }

        if count == 1 {
            return Display::fmt(&self.atoms[0], f);
        }

        let mut rendered: Vec<String> = self
            .atoms
            .iter()
            .map(|atom| {
                let text = atom.to_string();
                if atom.kind() == AtomKind::GenericParameter || (atom.has_intersection_types() && count > 1) {
                    format!("({text})")
                } else {
                    text
                }
            })
            .collect();
        rendered.sort_unstable();

        let mut first = true;
        for text in &rendered {
            if !first {
                f.write_str("|")?;
            }

            first = false;
            f.write_str(text)?;
        }

        Ok(())
    }
}

impl Display for Typed<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.ty, f)
    }
}

impl CopyInto for Type<'_> {
    type Output<'arena> = Type<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Type { atoms: copy_slice_into(self.atoms, arena), kinds: self.kinds }
    }
}

impl CopyInto for Typed<'_> {
    type Output<'arena> = Typed<'arena>;

    fn copy_into<'arena, A>(&self, arena: &'arena A) -> Self::Output<'arena>
    where
        A: Arena,
    {
        Typed { ty: self.ty.copy_into(arena), flags: self.flags, meta: self.meta }
    }
}
