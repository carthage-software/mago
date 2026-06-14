//! Build-then-finalize scratch buffer for [`Typed`] mutation.
//!
//! Every mutation through the [`TypeBuilder`] round-trips the result through
//! interning: sort, dedup, hash, table lookup. For consumers that perform
//! many mutations on the same type before observing the result
//! (assertion-handling loops, per-statement type evolution, switch-arm
//! merging) the per-step interning cost dominates.
//!
//! [`UnionBuffer`] solves this by holding the atoms and flow flags in an
//! owned scratch buffer. Mutations are direct `Vec` operations;
//! [`build`](UnionBuffer::build) sorts, deduplicates, and interns exactly
//! once at the end, and [`build_canonical`](UnionBuffer::build_canonical)
//! additionally applies the join's lattice-canonical collapses.
//!
//! # Origin short-circuit
//!
//! When constructed with [`from_typed`](UnionBuffer::from_typed), the buffer
//! remembers the originating value. If `build` is reached with the buffer in
//! the same shape (no mutation, no flag flip), it returns the original value
//! directly - no canonicalization, no intern lookup. A buffer that diverges
//! from the origin and then returns to the origin shape is still considered
//! "changed" and rebuilt; tracking the actual diff would defeat the point.
//!
//! # Querying mid-sequence
//!
//! The buffer does **not** expose `refines` / `overlaps` / `meet` against the
//! in-progress state. Those operations need a [`Type`](crate::ty::Type).
//! Call [`build`](UnionBuffer::build) to finalize, query, then open a fresh
//! buffer if more mutations follow.

use mago_allocator::Arena;
use mago_flags::U16Flags;

use crate::ty::Typed;
use crate::ty::atom::Atom;
use crate::ty::builder::TypeBuilder;
use crate::ty::flags::FlowFlag;

/// Mutable scratch buffer for accumulating changes to a type before
/// committing the result through a [`TypeBuilder`].
///
/// See the [module documentation](self) for the rationale and short-circuit
/// semantics.
#[derive(Debug, Clone)]
pub struct UnionBuffer<'arena> {
    atoms: Vec<Atom<'arena>>,
    flags: U16Flags<FlowFlag>,
    origin: Option<Typed<'arena>>,
    dirty: bool,
}

impl<'arena> UnionBuffer<'arena> {
    /// Construct an empty buffer. [`build`](Self::build) will collapse to
    /// [`well_known::TYPE_NEVER`](crate::ty::well_known::TYPE_NEVER), matching
    /// the [`TypeBuilder::union_of`] empty-input convention.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { atoms: Vec::new(), flags: U16Flags::empty(), origin: None, dirty: false }
    }

    /// Open a buffer backed by `typed`'s atoms and flags. The origin value is
    /// remembered so an unmodified `build()` returns it without re-interning.
    #[inline]
    #[must_use]
    pub fn from_typed(typed: Typed<'arena>) -> Self {
        Self { atoms: typed.ty.atoms.to_vec(), flags: typed.flags, origin: Some(typed), dirty: false }
    }

    /// Current atom buffer, in mutation order (not yet sorted, deduplicated,
    /// or canonicalized). Cheap.
    #[inline]
    #[must_use]
    pub fn atoms(&self) -> &[Atom<'arena>] {
        &self.atoms
    }

    /// Current flow flags.
    #[inline]
    #[must_use]
    pub const fn flags(&self) -> U16Flags<FlowFlag> {
        self.flags
    }

    /// `true` iff the buffer contains no atoms yet.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.atoms.is_empty()
    }

    /// `true` iff the buffer contains at least one occurrence of `atom`.
    /// O(n) on the buffer length; intended for predicate dispatch in the
    /// same loop that mutates.
    #[inline]
    #[must_use]
    pub fn contains(&self, atom: Atom<'arena>) -> bool {
        self.atoms.contains(&atom)
    }

    /// Number of atoms currently in the buffer.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.atoms.len()
    }

    /// Append `atom` to the buffer. Order is preserved during mutation;
    /// `build()` sorts before interning.
    #[inline]
    pub fn push(&mut self, atom: Atom<'arena>) -> &mut Self {
        self.atoms.push(atom);
        self.dirty = true;
        self
    }

    /// Append every atom from `iter`.
    #[inline]
    pub fn extend<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator<Item = Atom<'arena>>,
    {
        let before = self.atoms.len();
        self.atoms.extend(iter);
        if self.atoms.len() != before {
            self.dirty = true;
        }

        self
    }

    /// Remove the first occurrence of `atom`. No-op when absent.
    #[inline]
    pub fn remove(&mut self, atom: Atom<'arena>) -> &mut Self {
        if let Some(index) = self.atoms.iter().position(|existing| *existing == atom) {
            self.atoms.remove(index);
            self.dirty = true;
        }

        self
    }

    /// Remove every occurrence of `atom`.
    #[inline]
    pub fn remove_all(&mut self, atom: Atom<'arena>) -> &mut Self {
        let before = self.atoms.len();
        self.atoms.retain(|existing| *existing != atom);
        if self.atoms.len() != before {
            self.dirty = true;
        }

        self
    }

    /// Keep only atoms for which `predicate` returns `true`.
    #[inline]
    pub fn retain<F>(&mut self, mut predicate: F) -> &mut Self
    where
        F: FnMut(&Atom<'arena>) -> bool,
    {
        let before = self.atoms.len();
        self.atoms.retain(|atom| predicate(atom));
        if self.atoms.len() != before {
            self.dirty = true;
        }

        self
    }

    /// Replace the first occurrence of `old` with `new`. No-op when `old` is
    /// absent.
    #[inline]
    pub fn replace(&mut self, old: Atom<'arena>, new: Atom<'arena>) -> &mut Self {
        if let Some(index) = self.atoms.iter().position(|existing| *existing == old)
            && self.atoms[index] != new
        {
            self.atoms[index] = new;
            self.dirty = true;
        }

        self
    }

    /// Apply `f` to every atom, replacing each in place.
    #[inline]
    pub fn map<F>(&mut self, mut f: F) -> &mut Self
    where
        F: FnMut(Atom<'arena>) -> Atom<'arena>,
    {
        for slot in &mut self.atoms {
            let new = f(*slot);
            if new != *slot {
                *slot = new;
                self.dirty = true;
            }
        }

        self
    }

    /// Apply `f` to every atom, expanding each to zero or more atoms. Useful
    /// for narrowing patterns where one atom decomposes into a union (e.g.
    /// an integer range split).
    #[inline]
    pub fn flat_map<I, F>(&mut self, mut f: F) -> &mut Self
    where
        I: IntoIterator<Item = Atom<'arena>>,
        F: FnMut(Atom<'arena>) -> I,
    {
        let original = core::mem::take(&mut self.atoms);
        let mut rebuilt = Vec::with_capacity(original.len());
        let mut changed = false;
        for atom in original {
            let mut iter = f(atom).into_iter();
            match (iter.next(), iter.next()) {
                (Some(only), None) => {
                    if only != atom {
                        changed = true;
                    }

                    rebuilt.push(only);
                }
                (Some(first), Some(second)) => {
                    changed = true;
                    rebuilt.push(first);
                    rebuilt.push(second);
                    rebuilt.extend(iter);
                }
                (None, _) => {
                    changed = true;
                }
            }
        }

        self.atoms = rebuilt;
        if changed {
            self.dirty = true;
        }

        self
    }

    /// Replace the entire flow-flag set.
    #[inline]
    pub fn set_flags(&mut self, flags: U16Flags<FlowFlag>) -> &mut Self {
        if flags != self.flags {
            self.flags = flags;
            self.dirty = true;
        }

        self
    }

    /// Apply `f` to the current flow flags, replacing them with the returned
    /// value.
    #[inline]
    pub fn modify_flags<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(U16Flags<FlowFlag>) -> U16Flags<FlowFlag>,
    {
        let new = f(self.flags);
        if new != self.flags {
            self.flags = new;
            self.dirty = true;
        }

        self
    }

    /// Finalize the buffer through `builder`. Returns the original value
    /// directly when the buffer is unchanged from a
    /// [`from_typed`](Self::from_typed) origin (no intern round-trip, `meta`
    /// preserved).
    ///
    /// Interning sorts and deduplicates the atoms for canonical identity,
    /// but applies **no merge rules**: `true|false` does not collapse to
    /// `bool`, ranges are not merged, refinements are not absorbed. Callers
    /// that want the full lattice-canonical form use
    /// [`build_canonical`](Self::build_canonical). A rebuilt value carries
    /// `meta: 0`.
    ///
    /// Empty buffers collapse to
    /// [`well_known::TYPE_NEVER`](crate::ty::well_known::TYPE_NEVER).
    #[inline]
    #[must_use]
    pub fn build<S, A>(self, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Typed<'arena>
    where
        S: Arena,
        A: Arena,
    {
        if !self.dirty
            && let Some(origin) = self.origin
        {
            return origin;
        }

        Typed { ty: builder.union_of(&self.atoms), flags: self.flags, meta: 0 }
    }

    /// Finalize the buffer through the join's canonical preset:
    /// refined-int range merging, string-axis collapse, scalar synthesis,
    /// list / keyed-array element-type union, and subtype-driven
    /// absorption. Use [`build`](Self::build) when the caller does not
    /// want these collapses applied.
    #[inline]
    #[must_use]
    pub fn build_canonical<S, A>(self, builder: &mut TypeBuilder<'_, 'arena, S, A>) -> Typed<'arena>
    where
        S: Arena,
        A: Arena,
    {
        let canonical = crate::ty::join::compute(&self.atoms, builder);

        Typed { ty: builder.union_of(&canonical), flags: self.flags, meta: 0 }
    }
}

impl Default for UnionBuffer<'_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'arena> From<Typed<'arena>> for UnionBuffer<'arena> {
    #[inline]
    fn from(typed: Typed<'arena>) -> Self {
        Self::from_typed(typed)
    }
}
