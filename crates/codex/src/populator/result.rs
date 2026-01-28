use mago_atom::Atom;

use crate::metadata::class_like::ClassLikeMetadata;
use crate::ttype::resolution::TypeResolutionContext;

/// Result of populating a single class-like.
///
/// Contains the populated metadata and accumulated symbol references
/// that should be merged into the main reference store.
pub struct ClassPopulationResult {
    /// The class name.
    pub name: Atom,
    /// The populated class metadata.
    pub metadata: ClassLikeMetadata,
    /// Symbol-to-symbol references: (from, to, is_userland).
    pub symbol_references: Vec<(Atom, Atom, bool)>,
    /// Method type resolution contexts to apply.
    pub method_contexts: Vec<((Atom, Atom), TypeResolutionContext)>,
}

/// Accumulated references to merge at the end of population.
///
/// This collects all symbol references during parallel processing
/// and applies them in a single batch at the end.
#[derive(Default)]
pub struct AccumulatedReferences {
    /// Symbol-to-symbol references: (from, to, is_userland).
    pub symbol_to_symbol: Vec<(Atom, Atom, bool)>,
    /// Member-to-symbol references: ((class, member), to, is_userland).
    pub member_to_symbol: Vec<((Atom, Atom), Atom, bool)>,
}

impl AccumulatedReferences {
    /// Create a new empty accumulator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a symbol-to-symbol reference.
    #[inline]
    pub fn add_symbol_ref(&mut self, from: Atom, to: Atom, userland: bool) {
        self.symbol_to_symbol.push((from, to, userland));
    }

    /// Add a member-to-symbol reference.
    #[inline]
    pub fn add_member_ref(&mut self, from: (Atom, Atom), to: Atom, userland: bool) {
        self.member_to_symbol.push((from, to, userland));
    }

    /// Extend with symbol references from an iterator.
    #[inline]
    pub fn extend_symbol_refs(&mut self, refs: impl IntoIterator<Item = (Atom, Atom, bool)>) {
        self.symbol_to_symbol.extend(refs);
    }

    /// Extend with member references from an iterator.
    #[inline]
    pub fn extend_member_refs(&mut self, refs: impl IntoIterator<Item = ((Atom, Atom), Atom, bool)>) {
        self.member_to_symbol.extend(refs);
    }

    /// Merge another accumulator into this one.
    pub fn merge(&mut self, other: Self) {
        self.symbol_to_symbol.extend(other.symbol_to_symbol);
        self.member_to_symbol.extend(other.member_to_symbol);
    }

    /// Apply all accumulated references to the symbol references store.
    pub fn apply_to(self, refs: &mut crate::reference::SymbolReferences) {
        for (from, to, userland) in self.symbol_to_symbol {
            refs.add_symbol_reference_to_symbol(from, to, userland);
        }

        for ((class, member), to, userland) in self.member_to_symbol {
            refs.add_class_member_reference_to_symbol((class, member), to, userland);
        }
    }

    /// Check if empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.symbol_to_symbol.is_empty() && self.member_to_symbol.is_empty()
    }

    /// Get total count of references.
    #[inline]
    pub fn len(&self) -> usize {
        self.symbol_to_symbol.len() + self.member_to_symbol.len()
    }
}
