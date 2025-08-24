use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::AtomMap;

/// A pair of `Atom`s representing a symbol and its member.
///
/// This is used to uniquely identify a symbol and its member within the codebase,
/// where the first `Atom` is the symbol's fully qualified class name (FQCN)
/// and the second `Atom` is the member's name (e.g., method, property, constant),
/// or an empty string if the symbol itself is being referenced (e.g., a class or function
/// without a specific member).
pub type SymbolIdentifier = (Atom, Atom);

/// Represents the different kinds of top-level class-like structures in PHP.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum SymbolKind {
    Class,
    Enum,
    Trait,
    Interface,
}

impl SymbolKind {
    /// Checks if this symbol kind is `Class`.
    #[inline]
    pub const fn is_class(&self) -> bool {
        matches!(self, SymbolKind::Class)
    }

    /// Checks if this symbol kind is `Enum`.
    #[inline]
    pub const fn is_enum(&self) -> bool {
        matches!(self, SymbolKind::Enum)
    }

    /// Checks if this symbol kind is `Trait`.
    #[inline]
    pub const fn is_trait(&self) -> bool {
        matches!(self, SymbolKind::Trait)
    }

    /// Checks if this symbol kind is `Interface`.
    #[inline]
    pub const fn is_interface(&self) -> bool {
        matches!(self, SymbolKind::Interface)
    }

    /// Returns the string representation of the symbol kind.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::Class => "class",
            SymbolKind::Enum => "enum",
            SymbolKind::Trait => "trait",
            SymbolKind::Interface => "interface",
        }
    }
}

/// Stores a map of all known class-like symbol names (FQCNs) to their corresponding `SymbolKind`.
/// Provides basic methods for adding symbols and querying.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Symbols {
    all: AtomMap<SymbolKind>,
}

impl Symbols {
    /// Creates a new, empty `Symbols` map.
    #[inline]
    pub fn new() -> Symbols {
        Symbols { all: AtomMap::default() }
    }

    /// Adds or updates a symbol name identified as a `Class`.
    #[inline]
    pub fn add_class_name(&mut self, name: Atom) {
        self.all.insert(name, SymbolKind::Class);
    }

    /// Adds or updates a symbol name identified as an `Interface`.
    #[inline]
    pub fn add_interface_name(&mut self, name: Atom) {
        self.all.insert(name, SymbolKind::Interface);
    }

    /// Adds or updates a symbol name identified as a `Trait`.
    #[inline]
    pub fn add_trait_name(&mut self, name: Atom) {
        self.all.insert(name, SymbolKind::Trait);
    }

    /// Adds or updates a symbol name identified as an `Enum`.
    #[inline]
    pub fn add_enum_name(&mut self, name: Atom) {
        self.all.insert(name, SymbolKind::Enum);
    }

    /// Retrieves the `SymbolKind` for a given symbol name, if known.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Atom` (likely FQCN) of the symbol to look up.
    ///
    /// # Returns
    ///
    /// `Some(SymbolKind)` if the symbol exists in the map, `None` otherwise.
    #[inline]
    pub fn get_kind(&self, name: &Atom) -> Option<SymbolKind> {
        self.all.get(name).copied() // Use copied() since SymbolKind is Copy
    }

    /// Checks if a symbol with the given name is known.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Atom` (likely FQCN) of the symbol to check.
    ///
    /// # Returns
    ///
    /// `true` if the symbol exists in the map, `false` otherwise.
    #[inline]
    pub fn contains(&self, name: &Atom) -> bool {
        self.all.contains_key(name)
    }

    /// Checks if a symbol with the given name is a `Class`.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Atom` (likely FQCN) of the symbol to check.
    ///
    /// # Returns
    ///
    /// `true` if the symbol is a `Class`, `false` otherwise.
    #[inline]
    pub fn contains_class(&self, name: &Atom) -> bool {
        matches!(self.get_kind(name), Some(SymbolKind::Class))
    }

    /// Checks if a symbol with the given name is an `Interface`.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Atom` (likely FQCN) of the symbol to check.
    ///
    /// # Returns
    ///
    /// `true` if the symbol is an `Interface`, `false` otherwise.
    #[inline]
    pub fn contains_interface(&self, name: &Atom) -> bool {
        matches!(self.get_kind(name), Some(SymbolKind::Interface))
    }

    /// Checks if a symbol with the given name is a `Trait`.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Atom` (likely FQCN) of the symbol to check.
    ///
    /// # Returns
    ///
    /// `true` if the symbol is a `Trait`, `false` otherwise.
    #[inline]
    pub fn contains_trait(&self, name: &Atom) -> bool {
        matches!(self.get_kind(name), Some(SymbolKind::Trait))
    }

    /// Checks if a symbol with the given name is an `Enum`.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Atom` (likely FQCN) of the symbol to check.
    ///
    /// # Returns
    ///
    /// `true` if the symbol is an `Enum`, `false` otherwise.
    #[inline]
    pub fn contains_enum(&self, name: &Atom) -> bool {
        matches!(self.get_kind(name), Some(SymbolKind::Enum))
    }

    /// Returns a reference to the underlying map of all symbols.
    #[inline]
    pub fn get_all(&self) -> &AtomMap<SymbolKind> {
        &self.all
    }

    /// Extends the current `Symbols` map with another one.
    #[inline]
    pub fn extend(&mut self, other: Symbols) {
        for (entry, kind) in other.all {
            self.all.entry(entry).or_insert(kind);
        }
    }
}

/// Provides a default, empty `Symbols` map.
impl Default for Symbols {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
