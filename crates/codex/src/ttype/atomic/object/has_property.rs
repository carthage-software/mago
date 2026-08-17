use mago_word::Word;
use mago_word::concat_word;

use crate::ttype::TType;
use crate::ttype::TypeRef;
use crate::ttype::atomic::TAtomic;
use crate::ttype::atomic::append_intersection_ids;

/// Represents an object type that has a known property from a `property_exists()` check.
///
/// This type is created when the analyzer encounters a conditional like
/// `if (property_exists($obj, 'foo'))` and narrows the type within that block.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TObjectHasProperty {
    /// The name of the property that is known to exist.
    pub property: Word,
    /// Additional intersection types (e.g., other `HasMethod` or `HasProperty` assertions).
    pub intersection_types: Option<Vec<TAtomic>>,
}

impl TObjectHasProperty {
    /// Creates a new `TObjectHasProperty` with the given property name.
    #[inline]
    #[must_use]
    pub const fn new(property: Word) -> Self {
        Self { property, intersection_types: None }
    }

    /// Returns the property name.
    #[inline]
    #[must_use]
    pub const fn get_property(&self) -> Word {
        self.property
    }

    /// Checks if this property name matches the given name.
    #[inline]
    #[must_use]
    pub fn has_property(&self, property_name: Word) -> bool {
        self.property == property_name
    }
}

has_member_ttype_impl!(TObjectHasProperty, property, b"has-property<'");
