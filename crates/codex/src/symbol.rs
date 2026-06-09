use mago_word::WordSet;
use mago_word::word;

use mago_word::Word;
use mago_word::WordMap;

/// A pair of `Word`s representing a symbol and its member.
///
/// This is used to uniquely identify a symbol and its member within the codebase,
/// where the first `Word` is the symbol's fully qualified class name (FQCN)
/// and the second `Word` is the member's name (e.g., method, property, constant),
/// or an empty string if the symbol itself is being referenced (e.g., a class or function
/// without a specific member).
pub type SymbolIdentifier = (Word, Word);

/// Represents the different kinds of top-level class-like structures in PHP.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SymbolKind {
    Class,
    Enum,
    Trait,
    Interface,
}

impl SymbolKind {
    /// Checks if this symbol kind is `Class`.
    #[inline]
    #[must_use]
    pub const fn is_class(&self) -> bool {
        matches!(self, SymbolKind::Class)
    }

    /// Checks if this symbol kind is `Enum`.
    #[inline]
    #[must_use]
    pub const fn is_enum(&self) -> bool {
        matches!(self, SymbolKind::Enum)
    }

    /// Checks if this symbol kind is `Trait`.
    #[inline]
    #[must_use]
    pub const fn is_trait(&self) -> bool {
        matches!(self, SymbolKind::Trait)
    }

    /// Checks if this symbol kind is `Interface`.
    #[inline]
    #[must_use]
    pub const fn is_interface(&self) -> bool {
        matches!(self, SymbolKind::Interface)
    }

    /// Returns the string representation of the symbol kind.
    #[inline]
    #[must_use]
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
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Symbols {
    all: WordMap<SymbolKind>,
    namespaces: WordSet,
}

impl Symbols {
    /// Creates a new, empty `Symbols` map.
    #[inline]
    #[must_use]
    pub fn new() -> Symbols {
        Symbols { all: WordMap::default(), namespaces: WordSet::default() }
    }

    /// Adds or updates a symbol name identified as a `Class`.
    #[inline]
    pub fn add_class_name(&mut self, name: Word) {
        self.namespaces.extend(get_symbol_namespaces(name));
        self.all.insert(name, SymbolKind::Class);
    }

    /// Adds or updates a symbol name identified as an `Interface`.
    #[inline]
    pub fn add_interface_name(&mut self, name: Word) {
        self.namespaces.extend(get_symbol_namespaces(name));
        self.all.insert(name, SymbolKind::Interface);
    }

    /// Adds or updates a symbol name identified as a `Trait`.
    #[inline]
    pub fn add_trait_name(&mut self, name: Word) {
        self.namespaces.extend(get_symbol_namespaces(name));
        self.all.insert(name, SymbolKind::Trait);
    }

    /// Adds or updates a symbol name identified as an `Enum`.
    #[inline]
    pub fn add_enum_name(&mut self, name: Word) {
        self.namespaces.extend(get_symbol_namespaces(name));
        self.all.insert(name, SymbolKind::Enum);
    }

    /// Retrieves the `SymbolKind` for a given symbol name, if known.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Word` (likely FQCN) of the symbol to look up.
    ///
    /// # Returns
    ///
    /// `Some(SymbolKind)` if the symbol exists in the map, `None` otherwise.
    #[inline]
    #[must_use]
    pub fn get_kind(&self, name: Word) -> Option<SymbolKind> {
        self.all.get(&name).copied() // Use copied() since SymbolKind is Copy
    }

    /// Checks if a symbol with the given name is known.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Word` (likely FQCN) of the symbol to check.
    ///
    /// # Returns
    ///
    /// `true` if the symbol exists in the map, `false` otherwise.
    #[inline]
    #[must_use]
    pub fn contains(&self, name: Word) -> bool {
        self.all.contains_key(&name)
    }

    /// Check if any symbol within the table is part of the given namespace.
    ///
    /// # Arguments
    ///
    /// * `namespace`: The `Word` of the namespace to check for.
    ///
    /// # Returns
    ///
    /// `true` if the namespace is present, `false` otherwise.
    #[must_use]
    pub fn contains_namespace(&self, namespace: Word) -> bool {
        self.namespaces.contains(&namespace)
    }

    /// Checks if a symbol with the given name is a `Class`.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Word` (likely FQCN) of the symbol to check.
    ///
    /// # Returns
    ///
    /// `true` if the symbol is a `Class`, `false` otherwise.
    #[inline]
    #[must_use]
    pub fn contains_class(&self, name: Word) -> bool {
        matches!(self.get_kind(name), Some(SymbolKind::Class))
    }

    /// Checks if a symbol with the given name is an `Interface`.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Word` (likely FQCN) of the symbol to check.
    ///
    /// # Returns
    ///
    /// `true` if the symbol is an `Interface`, `false` otherwise.
    #[inline]
    #[must_use]
    pub fn contains_interface(&self, name: Word) -> bool {
        matches!(self.get_kind(name), Some(SymbolKind::Interface))
    }

    /// Checks if a symbol with the given name is a `Trait`.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Word` (likely FQCN) of the symbol to check.
    ///
    /// # Returns
    ///
    /// `true` if the symbol is a `Trait`, `false` otherwise.
    #[inline]
    #[must_use]
    pub fn contains_trait(&self, name: Word) -> bool {
        matches!(self.get_kind(name), Some(SymbolKind::Trait))
    }

    /// Checks if a symbol with the given name is an `Enum`.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Word` (likely FQCN) of the symbol to check.
    ///
    /// # Returns
    ///
    /// `true` if the symbol is an `Enum`, `false` otherwise.
    #[inline]
    #[must_use]
    pub fn contains_enum(&self, name: Word) -> bool {
        matches!(self.get_kind(name), Some(SymbolKind::Enum))
    }

    /// Returns a reference to the underlying map of all symbols.
    #[inline]
    #[must_use]
    pub fn get_all(&self) -> &WordMap<SymbolKind> {
        &self.all
    }

    /// Extends the current `Symbols` map with another one.
    #[inline]
    pub fn extend(&mut self, other: Symbols) {
        self.namespaces.extend(other.namespaces);
        for (entry, kind) in other.all {
            self.all.entry(entry).or_insert(kind);
        }
    }

    /// Extends the current `Symbols` map from a reference without consuming the source.
    #[inline]
    pub fn extend_ref(&mut self, other: &Symbols) {
        self.namespaces.extend(other.namespaces.iter().copied());

        for (entry, kind) in &other.all {
            self.all.entry(*entry).or_insert(*kind);
        }
    }

    /// Removes a symbol by its FQCN.
    ///
    /// Note: does not remove namespaces (they may be shared by other symbols).
    #[inline]
    pub fn remove(&mut self, name: Word) {
        self.all.remove(&name);
    }
}

/// Provides a default, empty `Symbols` map.
impl Default for Symbols {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
/// Returns an iterator that yields all parent namespaces of a given symbol.
///
/// For example, if the symbol is `Foo\Bar\Baz\Qux`, the iterator yields:
/// 1. `Foo`
/// 2. `Foo\Bar`
/// 3. `Foo\Bar\Baz`
pub(super) fn get_symbol_namespaces(symbol_name: Word) -> impl Iterator<Item = Word> {
    let bytes: Vec<u8> = symbol_name.as_bytes().to_vec();

    (0..bytes.len()).filter_map(move |i| if bytes[i] == b'\\' { Some(word(&bytes[..i])) } else { None })
}
