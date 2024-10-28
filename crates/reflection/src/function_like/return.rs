use serde::Deserialize;
use serde::Serialize;

use fennec_span::Span;

use crate::r#type::TypeReflection;

/// Represents the return type information for a function-like entity,
/// including the type itself and its location in the source code.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct FunctionLikeReturnTypeReflection {
    /// The return type of the function-like entity.
    pub type_reflection: TypeReflection,

    /// The location in the source code where the return type is specified.
    pub span: Span,
}
