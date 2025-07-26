use serde::Deserialize;
use serde::Serialize;

use mago_span::Span;

use crate::ttype::union::TUnion;

/// Contains metadata associated with a specific type instance within the type system.
///
/// This struct combines the core type information (`TUnion`) with contextual details
/// about *how* and *where* this type information was determined or declared in the source code
/// or related documentation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypeMetadata {
    /// The specific location (span) in the source code or documentation
    /// that this type metadata corresponds to.
    ///
    /// This could be:
    /// - The span of a type annotation (e.g., `: string`).
    /// - The span of an expression whose type was inferred (e.g., `$x = 10` -> span of `10`).
    /// - The span of a type mentioned in a documentation block (e.g., `@param int` -> span of `int`).
    pub span: Span,

    /// The core representation of the type itself.
    pub type_union: TUnion,

    /// Distinguishes whether this type information originated from analyzing
    /// executable code constructs (e.g., type declarations, assignments)
    /// or from documentation blocks.
    ///
    /// - `true` if the type information was extracted from a docblock comment.
    /// - `false` if the type information came from actual code analysis.
    pub from_docblock: bool,

    /// Indicates whether this type was explicitly declared in the source
    /// or deduced ("inferred") by the type checker based on context.
    ///
    /// - `true` if the type checker inferred this type (e.g., from a variable initialization like `$x = 10;`).
    /// - `false` if the type was explicitly written by the user (e.g., `int $x;`).
    pub inferred: bool,
}

impl TypeMetadata {
    /// Creates new `TypeMetadata` for an explicitly declared type from code.
    ///
    /// This is a convenience constructor assuming the common case where a type
    /// is directly specified in the code, and wasn't just inferred or from a docblock.
    ///
    /// # Arguments
    ///
    /// * `type_union`: The core type information (`TUnion`).
    /// * `span`: The source code location associated with this type.
    ///
    /// # Returns
    ///
    /// A new `TypeMetadata` instance with `is_nullable`, `from_docblock`, and `inferred` set to `false`.
    pub fn new(type_union: TUnion, span: Span) -> Self {
        Self { span, type_union, from_docblock: false, inferred: false }
    }

    /// Creates a new `TypeMetadata` by applying a function to the inner `TUnion`.
    ///
    /// This allows transforming the core type while preserving the surrounding metadata.
    ///
    /// # Arguments
    ///
    /// * `f`: A function that takes the current `TUnion` and returns a new `TUnion`.
    ///
    /// # Returns
    ///
    /// A new `TypeMetadata` instance with the transformed `TUnion` and the same metadata flags and span.
    pub fn map_type_union<F>(self, f: F) -> Self
    where
        F: FnOnce(TUnion) -> TUnion,
    {
        Self {
            span: self.span,
            type_union: f(self.type_union),
            from_docblock: self.from_docblock,
            inferred: self.inferred,
        }
    }
}
