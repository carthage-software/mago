use serde::Deserialize;
use serde::Serialize;

use fennec_span::Span;

use crate::r#type::TypeReflection;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct FunctionLikeReturnTypeReflection {
    pub type_reflection: TypeReflection,
    pub span: Span,
}
