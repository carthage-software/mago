use serde::Deserialize;
use serde::Serialize;

use fennec_span::Span;

use crate::attribute::AttributeReflection;
use crate::identifier::ClassLikeMemberIdentifier;
use crate::r#type::TypeReflection;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct EnumCaseReflection {
    pub attribut_reflections: Vec<AttributeReflection>,
    pub identifier: ClassLikeMemberIdentifier,
    pub type_reflection: Option<TypeReflection>,
    pub is_backed: bool,
    pub span: Span,
}
