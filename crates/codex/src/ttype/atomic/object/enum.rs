use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;

use crate::ttype::TType;

/// Represents metadata specific to a PHP enum type (`enum`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TEnum {
    /// The fully qualified name (FQCN) of the enum.
    pub name: StringIdentifier,
    /// The case name of the enum variant, if specified.
    pub case: Option<StringIdentifier>,
}

impl TEnum {
    /// Creates metadata for an enum.
    ///
    /// # Arguments
    ///
    /// * `name`: The `StringIdentifier` for the enum's FQCN.
    #[inline]
    pub const fn new(name: StringIdentifier) -> Self {
        Self { name, case: None }
    }

    /// Creates metadata for an enum case.
    ///
    /// # Arguments
    ///
    /// * `name`: The `StringIdentifier` for the enum's FQCN.
    /// * `case`: The `StringIdentifier` for the enum case name.
    #[inline]
    pub const fn new_case(name: StringIdentifier, case: StringIdentifier) -> Self {
        Self { name, case: Some(case) }
    }

    /// Returns the `StringIdentifier` for the enum's FQCN.
    #[inline]
    pub const fn get_name(&self) -> StringIdentifier {
        self.name
    }

    /// Returns a reference to the `StringIdentifier` for the enum's FQCN.
    #[inline]
    pub const fn get_name_ref(&self) -> &StringIdentifier {
        &self.name
    }

    /// Returns the `StringIdentifier` for the enum case, if it exists.
    #[inline]
    pub const fn get_case(&self) -> Option<StringIdentifier> {
        self.case
    }
}

impl TType for TEnum {
    fn get_id(&self, interner: Option<&mago_interner::ThreadedInterner>) -> String {
        let mut id = String::new();
        id += "enum(";
        if let Some(interner) = interner {
            id += interner.lookup(&self.name);
        } else {
            id += self.name.to_string().as_str();
        }

        if let Some(case) = &self.case {
            id += "::";
            if let Some(interner) = interner {
                id += interner.lookup(case);
            } else {
                id += case.to_string().as_str();
            }
        }

        id += ")";
        id
    }
}
