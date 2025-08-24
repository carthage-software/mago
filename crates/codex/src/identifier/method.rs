use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;

/// Represents a unique identifier for a method within a class-like structure.
/// Combines the fully qualified class name (FQCN) and the method name.
#[derive(Clone, Debug, PartialEq, Eq, Copy, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct MethodIdentifier {
    /// The fully qualified name of the class, interface, trait, or enum containing the method.
    class_name: Atom,
    /// The name of the method itself.
    method_name: Atom,
}

impl MethodIdentifier {
    /// Creates a new `MethodIdentifier`.
    ///
    /// # Arguments
    ///
    /// * `class_name`: The `Atom` for the fully qualified class name.
    /// * `method_name`: The `Atom` for the method name.
    #[inline]
    pub const fn new(class_name: Atom, method_name: Atom) -> Self {
        Self { class_name, method_name }
    }

    /// Returns the `Atom` for the class name.
    #[inline]
    pub const fn get_class_name(&self) -> &Atom {
        &self.class_name
    }

    /// Returns the `Atom` for the method name.
    #[inline]
    pub const fn get_method_name(&self) -> &Atom {
        &self.method_name
    }

    /// Converts the identifier to a human-readable string "ClassName::methodName".
    #[inline]
    pub fn as_string(&self) -> String {
        format!("{}::{}", self.class_name, self.method_name)
    }

    /// Converts the identifier to a tuple of `Atom`s representing the class name and method name.
    #[inline]
    pub fn get_key(&self) -> (Atom, Atom) {
        (self.class_name, self.method_name)
    }
}
