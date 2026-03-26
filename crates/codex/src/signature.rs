use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;

/// Represents a signature node for a definition (function, class, method, constant, etc.).
///
/// This structure forms a hierarchical tree where top-level symbols (classes, functions)
/// can have children (methods, properties within classes).
///
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefSignatureNode {
    /// The name of the symbol (e.g., "Foo" for class Foo, "bar" for method bar)
    pub name: Atom,

    /// Whether this node represents a function or method
    pub is_function: bool,

    /// Whether this node represents a constant
    pub is_constant: bool,

    /// Starting byte offset in the source file
    pub start_offset: u32,

    /// Ending byte offset in the source file
    pub end_offset: u32,

    /// Starting line number (1-indexed)
    pub start_line: u32,

    /// Ending line number (1-indexed)
    pub end_line: u32,

    /// Starting column (0-indexed)
    pub start_column: u16,

    /// Ending column (0-indexed)
    pub end_column: u16,

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
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        name: Atom,
        is_function: bool,
        is_constant: bool,
        start_offset: u32,
        end_offset: u32,
        start_line: u32,
        end_line: u32,
        start_column: u16,
        end_column: u16,
        hash: u64,
        signature_hash: u64,
    ) -> Self {
        Self {
            name,
            is_function,
            is_constant,
            start_offset,
            end_offset,
            start_line,
            end_line,
            start_column,
            end_column,
            children: Vec::new(),
            hash,
            signature_hash,
        }
    }

    /// Adds a child node to this definition.
    #[inline]
    pub fn add_child(&mut self, child: DefSignatureNode) {
        self.children.push(child);
    }

    /// Returns a reference to the children of this node.
    #[inline]
    #[must_use]
    pub fn children(&self) -> &[DefSignatureNode] {
        &self.children
    }

    /// Returns a mutable reference to the children of this node.
    #[inline]
    pub fn children_mut(&mut self) -> &mut Vec<DefSignatureNode> {
        &mut self.children
    }
}

/// Represents the signature of an entire file.
///
/// This contains all top-level definitions (classes, interfaces, traits, enums,
/// functions, constants) in the file as a flat vector. Nested definitions
/// (methods, properties) are stored within the `children` of their parent nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FileSignature {
    pub hash: u64,
    pub ast_nodes: Vec<DefSignatureNode>,
}

impl FileSignature {
    /// Creates a new empty `FileSignature`.
    #[inline]
    #[must_use]
    pub fn new(hash: u64) -> Self {
        Self { hash, ast_nodes: Vec::new() }
    }

    /// Adds a top-level definition node to this file signature.
    #[inline]
    pub fn add_node(&mut self, node: DefSignatureNode) {
        self.ast_nodes.push(node);
    }

    /// Returns a reference to the top-level nodes.
    #[inline]
    #[must_use]
    pub fn nodes(&self) -> &[DefSignatureNode] {
        &self.ast_nodes
    }

    /// Returns a mutable reference to the top-level nodes.
    #[inline]
    pub fn nodes_mut(&mut self) -> &mut Vec<DefSignatureNode> {
        &mut self.ast_nodes
    }

    /// Returns true if this file signature has no nodes.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ast_nodes.is_empty()
    }

    /// Returns the number of top-level nodes.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.ast_nodes.len()
    }
}
