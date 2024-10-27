use serde::Deserialize;
use serde::Serialize;

use fennec_span::Span;

use crate::attribute::AttributeReflection;
use crate::class_like::member::ClassLikeMemberVisibilityReflection;
use crate::identifier::ClassLikeMemberIdentifier;
use crate::r#type::TypeReflection;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ClassLikeConstantReflection {
    pub attribut_reflections: Vec<AttributeReflection>,
    pub visibility_reflection: Option<ClassLikeMemberVisibilityReflection>,
    pub identifier: ClassLikeMemberIdentifier,
    pub type_reflection: Option<TypeReflection>,
    pub inferred_type_reflection: Option<TypeReflection>,
    pub span: Span,
}
