use serde::Deserialize;
use serde::Serialize;

use mago_source::HasSource;
use mago_source::SourceIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

use crate::r#type::TypeReflection;

/// Represents the return type information for a function-like entity,
/// including the type itself and its location in the source code.
///
/// This structure provides metadata about the return type of a function or method,
/// allowing for introspection and reflection of its type and position.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct FunctionLikeReturnTypeReflection {
    /// The return type of the function-like entity.
    pub type_reflection: TypeReflection,

    /// The location in the source code where the return type is specified.
    pub span: Span,
}

impl HasSpan for FunctionLikeReturnTypeReflection {
    /// Returns the span of the return type in the source code.
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSource for FunctionLikeReturnTypeReflection {
    /// Returns the source identifier of the file containing this return type.
    fn source(&self) -> SourceIdentifier {
        self.span.source()
    }
}
