use serde::Deserialize;
use serde::Serialize;

use fennec_span::Span;

use crate::attribute::AttributeReflection;
use crate::class_like::member::ClassLikeMemberVisibilityReflection;
use crate::identifier::ClassLikeMemberName;
use crate::r#type::TypeReflection;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ClassLikeConstantReflection {
    pub attribute_reflections: Vec<AttributeReflection>,
    pub visibility_reflection: Option<ClassLikeMemberVisibilityReflection>,
    pub name: ClassLikeMemberName,
    pub type_reflection: Option<TypeReflection>,
    pub inferred_type_reflection: Option<TypeReflection>,
    pub is_final: bool,
    pub item_span: Span,
    pub definition_span: Span,
}
