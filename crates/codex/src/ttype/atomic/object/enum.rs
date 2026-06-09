use mago_word::Word;
use mago_word::concat_word;

use crate::ttype::TType;

/// Represents metadata specific to a PHP enum type (`enum`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TEnum {
    /// The fully qualified name (FQCN) of the enum.
    pub name: Word,
    /// The case name of the enum variant, if specified.
    pub case: Option<Word>,
}

impl TEnum {
    /// Creates metadata for an enum.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Word` for the enum's FQCN.
    #[inline]
    #[must_use]
    pub const fn new(name: Word) -> Self {
        Self { name, case: None }
    }

    /// Creates metadata for an enum case.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Word` for the enum's FQCN.
    /// * `case`: The `Word` for the enum case name.
    #[inline]
    #[must_use]
    pub const fn new_case(name: Word, case: Word) -> Self {
        Self { name, case: Some(case) }
    }

    /// Returns the `Word` for the enum's FQCN.
    #[inline]
    #[must_use]
    pub const fn get_name(&self) -> Word {
        self.name
    }

    /// Returns the `Word` for the enum case, if it exists.
    #[inline]
    #[must_use]
    pub const fn get_case(&self) -> Option<Word> {
        self.case
    }
}

impl TType for TEnum {
    fn needs_population(&self) -> bool {
        false
    }

    fn is_expandable(&self) -> bool {
        false
    }

    fn is_complex(&self) -> bool {
        false
    }

    fn get_id(&self) -> Word {
        match self.case {
            Some(case) => concat_word!(b"enum(", self.name, b"::", case, b")"),
            None => concat_word!(b"enum(", self.name, b")"),
        }
    }

    fn get_pretty_id_with_indent(&self, _indent: usize) -> Word {
        self.get_id()
    }
}
