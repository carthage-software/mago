use mago_word::Word;

/// Represents a unique identifier for a method within a class-like structure.
/// Combines the fully qualified class name (FQCN) and the method name.
#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MethodIdentifier {
    /// The fully qualified name of the class, interface, trait, or enum containing the method.
    class_name: Word,
    /// The name of the method itself.
    method_name: Word,
}

impl MethodIdentifier {
    /// Creates a new `MethodIdentifier`.
    ///
    /// # Arguments
    ///
    /// * `class_name`: The `Word` for the fully qualified class name.
    /// * `method_name`: The `Word` for the method name.
    #[inline]
    #[must_use]
    pub const fn new(class_name: Word, method_name: Word) -> Self {
        Self { class_name, method_name }
    }

    /// Returns the `Word` for the class name.
    #[inline]
    #[must_use]
    pub const fn get_class_name(&self) -> Word {
        self.class_name
    }

    /// Returns the `Word` for the method name.
    #[inline]
    #[must_use]
    pub const fn get_method_name(&self) -> Word {
        self.method_name
    }

    /// Converts the identifier to a human-readable string "`ClassName::methodName`".
    #[inline]
    #[must_use]
    pub fn as_string(&self) -> String {
        format!("{}::{}", self.class_name, self.method_name)
    }

    /// Converts the identifier to a tuple of `Word`s representing the class name and method name.
    #[inline]
    #[must_use]
    pub fn get_key(&self) -> (Word, Word) {
        (self.class_name, self.method_name)
    }
}
