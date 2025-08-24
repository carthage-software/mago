use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::concat_atom;

use crate::ttype::TType;

/// Represents metadata specific to a PHP enum type (`enum`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TEnum {
    /// The fully qualified name (FQCN) of the enum.
    pub name: Atom,
    /// The case name of the enum variant, if specified.
    pub case: Option<Atom>,
}

impl TEnum {
    /// Creates metadata for an enum.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Atom` for the enum's FQCN.
    #[inline]
    pub const fn new(name: Atom) -> Self {
        Self { name, case: None }
    }

    /// Creates metadata for an enum case.
    ///
    /// # Arguments
    ///
    /// * `name`: The `Atom` for the enum's FQCN.
    /// * `case`: The `Atom` for the enum case name.
    #[inline]
    pub const fn new_case(name: Atom, case: Atom) -> Self {
        Self { name, case: Some(case) }
    }

    /// Returns the `Atom` for the enum's FQCN.
    #[inline]
    pub const fn get_name(&self) -> Atom {
        self.name
    }

    /// Returns a reference to the `Atom` for the enum's FQCN.
    #[inline]
    pub const fn get_name_ref(&self) -> &Atom {
        &self.name
    }

    /// Returns the `Atom` for the enum case, if it exists.
    #[inline]
    pub const fn get_case(&self) -> Option<Atom> {
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

    fn get_id(&self) -> Atom {
        match self.case {
            Some(case) => concat_atom!("enum(", self.name, "::", case, ")"),
            None => concat_atom!("enum(", self.name, ")"),
        }
    }
}
