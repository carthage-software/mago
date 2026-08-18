use mago_word::Word;

/// Represents a signature node for a definition (function, class, method, constant, etc.).
///
/// This structure forms a hierarchical tree where top-level symbols (classes, functions)
/// can have children (methods, properties within classes).
///
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefSignatureNode {
    /// The name of the symbol (e.g., "Foo" for class Foo, "bar" for method bar)
    pub name: Word,

    /// Whether this node represents a function or method
    pub is_function: bool,

    /// Nested symbols (e.g., methods and properties within a class)
    pub children: Vec<DefSignatureNode>,

    /// Position-insensitive fingerprint hash covering the entire definition.
    /// Any change to signature, body, modifiers, or attributes will change this hash.
    pub hash: u64,

    /// Signature-only fingerprint hash, excluding function/method bodies.
    /// Used by the differ to determine cascade invalidation: if only the body changed
    /// (signature_hash unchanged), dependents are not invalidated — only the changed
    /// file itself is re-analyzed.
    pub signature_hash: u64,
}

impl DefSignatureNode {
    /// Creates a new `DefSignatureNode` with the given parameters.
    #[inline]
    #[must_use]
    pub fn new(name: Word, is_function: bool, hash: u64, signature_hash: u64) -> Self {
        Self { name, is_function, children: Vec::new(), hash, signature_hash }
    }
}

/// Represents the signature of an entire file.
///
/// This contains all top-level definitions (classes, interfaces, traits, enums,
/// functions, constants) in the file as a flat vector. Nested definitions
/// (methods, properties) are stored within the `children` of their parent nodes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FileSignature {
    pub hash: u64,
    pub ast_nodes: Vec<DefSignatureNode>,
}
