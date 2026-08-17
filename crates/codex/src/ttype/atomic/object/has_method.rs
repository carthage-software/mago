use mago_word::Word;
use mago_word::concat_word;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::append_intersection_ids;

/// Represents an object type that has a known method from a `method_exists()` check.
///
/// This type is created when the analyzer encounters a conditional like
/// `if (method_exists($obj, 'foo'))` and narrows the type within that block.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TObjectHasMethod {
    /// The name of the method that is known to exist.
    pub method: Word,
    /// Additional intersection types (e.g., other `HasMethod` or `HasProperty` assertions).
    pub intersection_types: Option<Vec<TAtomic>>,
}

impl TObjectHasMethod {
    /// Creates a new `TObjectHasMethod` with the given method name.
    #[inline]
    #[must_use]
    pub const fn new(method: Word) -> Self {
        Self { method, intersection_types: None }
    }

    /// Returns the method name.
    #[inline]
    #[must_use]
    pub const fn get_method(&self) -> Word {
        self.method
    }

    /// Checks if this method name matches the given name (case-insensitive).
    #[inline]
    #[must_use]
    pub fn has_method(&self, method_name: &[u8]) -> bool {
        self.method.as_bytes().eq_ignore_ascii_case(method_name)
    }
}

has_member_ttype_impl!(TObjectHasMethod, method, b"has-method<'");
