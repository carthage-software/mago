use serde::Deserialize;
use serde::Serialize;

use fennec_span::Span;

use crate::identifier::ConstantIdentifier;
use crate::r#type::TypeReflection;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ConstantReflection {
    pub identifier: ConstantIdentifier,
    pub type_reflection: Option<TypeReflection>,
    pub item_span: Span,
    pub definition_span: Span,
}
