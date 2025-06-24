use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_interner::ThreadedInterner;

/// Represents a unique identifier for a method within a class-like structure.
/// Combines the fully qualified class name (FQCN) and the method name.
#[derive(Clone, Debug, PartialEq, Eq, Copy, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct MethodIdentifier {
    /// The fully qualified name of the class, interface, trait, or enum containing the method.
    class_name: StringIdentifier,
    /// The name of the method itself.
    method_name: StringIdentifier,
}

impl MethodIdentifier {
    /// Creates a new `MethodIdentifier`.
    ///
    /// # Arguments
    ///
    /// * `class_name`: The `StringIdentifier` for the fully qualified class name.
    /// * `method_name`: The `StringIdentifier` for the method name.
    #[inline]
    pub const fn new(class_name: StringIdentifier, method_name: StringIdentifier) -> Self {
        Self { class_name, method_name }
    }

    /// Returns the `StringIdentifier` for the class name.
    #[inline]
    pub const fn get_class_name(&self) -> &StringIdentifier {
        &self.class_name
    }

    /// Returns the `StringIdentifier` for the method name.
    #[inline]
    pub const fn get_method_name(&self) -> &StringIdentifier {
        &self.method_name
    }

    /// Converts the identifier to a human-readable string "ClassName::methodName" using the provided interner.
    ///
    /// # Arguments
    ///
    /// * `interner` - A reference to the `ThreadedInterner` used to resolve `StringIdentifier`s.
    #[inline]
    pub fn as_string(&self, interner: &ThreadedInterner) -> String {
        format!("{}::{}", interner.lookup(&self.class_name), interner.lookup(&self.method_name))
    }

    /// Converts the identifier to a tuple of `StringIdentifier`s representing the class name and method name.
    #[inline]
    pub fn get_key(&self) -> (StringIdentifier, StringIdentifier) {
        (self.class_name, self.method_name)
    }
}
