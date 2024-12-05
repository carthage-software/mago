use std::collections::HashSet;
use std::sync::Arc;

use lasso::Key;
use lasso::Rodeo;
use lasso::ThreadedRodeo;
use serde::Deserialize;
use serde::Serialize;

/// An string identifier that is used to represent an interned string.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct StringIdentifier(pub(crate) usize);

impl StringIdentifier {
    /// Creates a new empty `StringIdentifier`.
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Creates a new `StringIdentifier`.
    ///
    /// # Arguments
    ///
    /// * `val` - The value of the string identifier.
    pub const fn new(val: usize) -> Self {
        Self(val)
    }

    /// Returns `true` if the string is empty.
    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Returns the value of the string identifier.
    #[inline(always)]
    pub const fn value(&self) -> usize {
        self.0
    }
}

unsafe impl Key for StringIdentifier {
    fn into_usize(self) -> usize {
        self.0 - 1
    }

    fn try_from_usize(int: usize) -> Option<Self> {
        Some(Self(int + 1))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interner {
    rodeo: Rodeo<StringIdentifier>,
}

/// A string interner that stores strings and assigns them unique identifiers.
impl Interner {
    /// Creates a new `Interner`.
    pub fn new() -> Self {
        Self { rodeo: Rodeo::new() }
    }

    /// Returns the number of strings stored in the interner.
    pub fn len(&self) -> usize {
        self.rodeo.len()
    }

    /// Returns `true` if the interner is empty.
    pub fn is_empty(&self) -> bool {
        self.rodeo.is_empty()
    }

    /// Returns the identifier for the specified interned string.
    ///
    /// # Arguments
    ///
    /// * string - The interned string.
    pub fn get(&self, string: impl AsRef<str>) -> Option<StringIdentifier> {
        let str = string.as_ref();
        if str.is_empty() {
            return Some(StringIdentifier::empty());
        }

        self.rodeo.get(str)
    }

    /// Interns the specified string, returning the identifier for it.
    ///
    /// If the string is already interned, the existing identifier is returned.
    ///
    /// # Arguments
    ///
    /// * string - The string to intern.
    pub fn intern(&mut self, string: impl AsRef<str>) -> StringIdentifier {
        let str = string.as_ref();
        if str.is_empty() {
            return StringIdentifier::empty();
        }

        self.rodeo.get_or_intern(str)
    }

    /// Interns a string if it has not already been interned, then returns a reference
    /// to the interned string.
    ///
    /// # Arguments
    ///
    /// * `string` - A string or any type that implements `AsRef<str>`, representing the
    ///   string to intern.
    ///
    /// # Returns
    ///
    /// A reference to the interned version of the string.
    ///
    /// # Panics
    ///
    /// This method will panic if it encounters an invalid identifier. This should never
    /// occur unless there is an issue with the identifier or the interner is used
    /// incorrectly.
    pub fn interned_str(&mut self, string: impl AsRef<str>) -> &str {
        let str = string.as_ref();
        if str.is_empty() {
            return "";
        }

        let identifier = self.rodeo.get_or_intern(str);

        self.rodeo.try_resolve(&identifier).expect(
            "invalid string identifier; this should never happen unless the identifier is \
                corrupted or the interner is used incorrectly",
        )
    }

    /// Returns the interned string for the specified identifier.
    ///
    /// # Arguments
    ///
    /// * identifier - The identifier to look up.
    ///
    /// # Panics
    ///
    /// Panics if the identifier is invalid
    pub fn lookup(&self, identifier: &StringIdentifier) -> &str {
        if identifier.is_empty() {
            return "";
        }

        self.rodeo.try_resolve(identifier).expect(
            "invalid string identifier; this should never happen unless the identifier is \
                corrupted or the interner is used incorrectly",
        )
    }
}

/// A thread-safe interner, allowing multiple threads to concurrently intern strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadedInterner {
    rodeo: Arc<ThreadedRodeo<StringIdentifier>>,
}

impl ThreadedInterner {
    /// Creates a new `ThreadedInterner`.
    pub fn new() -> Self {
        Self { rodeo: Arc::new(ThreadedRodeo::new()) }
    }

    /// Returns the number of strings stored in the interner.
    pub fn len(&self) -> usize {
        self.rodeo.len()
    }

    /// Returns `true` if the interner is empty.
    pub fn is_empty(&self) -> bool {
        self.rodeo.is_empty()
    }

    /// Interns a string and returns its identifier.
    ///
    /// If the string is already interned, the existing identifier is returned.
    ///
    /// # Arguments
    ///
    /// * `string` - The string to intern.
    pub fn intern(&self, string: impl AsRef<str>) -> StringIdentifier {
        let str = string.as_ref();
        if str.is_empty() {
            return StringIdentifier::empty();
        }

        self.rodeo.get_or_intern(str)
    }

    /// Interns a string if it has not already been interned, then returns a reference
    /// to the interned string.
    ///
    /// # Arguments
    ///
    /// * `string` - A string or any type that implements `AsRef<str>`, representing the
    ///   string to intern.
    ///
    /// # Returns
    ///
    /// A reference to the interned version of the string.
    ///
    /// # Panics
    ///
    /// This method will panic if it encounters an invalid identifier. This should never
    /// occur unless there is an issue with the identifier or the interner is used
    /// incorrectly.
    pub fn interned_str(&self, string: impl AsRef<str>) -> &str {
        let str = string.as_ref();
        if str.is_empty() {
            return "";
        }

        let identifier = self.rodeo.get_or_intern(str);

        self.rodeo.try_resolve(&identifier).expect(
            "invalid string identifier; this should never happen unless the identifier is \
                corrupted or the interner is used incorrectly",
        )
    }

    /// Looks up an interned string by its identifier.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the interned string to look up.
    ///
    /// # Panics
    ///
    /// This method will panic if it encounters an invalid identifier. This should never
    /// occur unless there is an issue with the identifier or the interner is used
    /// incorrectly.
    pub fn lookup(&self, identifier: &StringIdentifier) -> &str {
        if identifier.is_empty() {
            return "";
        }

        self.rodeo.try_resolve(identifier).expect(
            "invalid string identifier; this should never happen unless the identifier is \
                corrupted or the interner is used incorrectly",
        )
    }

    /// Returns all interned strings and their identifiers as a hashmap.
    pub fn all(&self) -> HashSet<(StringIdentifier, &str)> {
        self.rodeo.iter().collect()
    }
}

impl std::fmt::Display for StringIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "string-identifier({})", self.0)
    }
}

unsafe impl Send for ThreadedInterner {}
unsafe impl Sync for ThreadedInterner {}

impl std::default::Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}

impl std::default::Default for ThreadedInterner {
    fn default() -> Self {
        Self::new()
    }
}
