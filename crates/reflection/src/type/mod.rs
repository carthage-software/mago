use serde::Deserialize;
use serde::Serialize;

use mago_source::HasSource;
use mago_source::SourceIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

use crate::r#type::kind::TypeKind;

pub mod kind;

/// Represents a reflection of a type in the codebase.
///
/// This structure provides metadata about a type, including its kind (e.g., string, integer),
/// whether it was inferred, and its location in the source code.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TypeReflection {
    /// The kind of the type (e.g., string, integer).
    pub kind: TypeKind,

    /// Whether the type was inferred or explicitly declared.
    pub inferred: bool,

    /// The span of the type in the source code.
    pub span: Span,
}

impl HasSpan for TypeReflection {
    /// Returns the span of the type in the source code.
    ///
    /// The span identifies the precise location of the type definition or usage
    /// within the source file.
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSource for TypeReflection {
    /// Returns the source identifier of the file containing this type.
    ///
    /// The source identifier provides metadata about the origin of the file,
    /// such as whether it is user-defined, vendor-provided, or built-in.
    fn source(&self) -> SourceIdentifier {
        self.span.source()
    }
}
