use ahash::HashSet;

use serde::Deserialize;
use serde::Serialize;

use fennec_span::Span;

use crate::attribute::AttributeReflection;
use crate::class_like::member::ClassLikeMemberVisibilityReflection;
use crate::function_like::FunctionLikeReflection;
use crate::identifier::ClassLikeMemberIdentifier;
use crate::r#type::TypeReflection;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PropertyDefaultValueReflection {
    pub inferred_type_reflection: Option<TypeReflection>,
    pub span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PropertyReflection {
    pub attribut_reflections: Vec<AttributeReflection>,
    pub visibility_reflection: Option<ClassLikeMemberVisibilityReflection>,
    pub identifier: ClassLikeMemberIdentifier,
    pub type_reflection: Option<TypeReflection>,
    pub default_value_reflection: Option<PropertyDefaultValueReflection>,
    pub hooks: HashSet<FunctionLikeReflection>,
    pub is_readonly: bool,
    pub is_final: bool,
    pub is_promoted: bool,
    pub is_static: bool,
    pub span: Span,
}
