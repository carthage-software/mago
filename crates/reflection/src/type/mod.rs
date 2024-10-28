use serde::Deserialize;
use serde::Serialize;

use fennec_span::Span;

use crate::r#type::kind::TypeKind;

pub mod kind;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct TypeReflection {
    pub kind: TypeKind,
    pub inferred: bool,
    pub span: Span,
}
