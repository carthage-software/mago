use ahash::HashMap;

use fennec_interner::StringIdentifier;
use serde::Deserialize;
use serde::Serialize;

use fennec_span::Span;

use crate::attribute::AttributeReflection;
use crate::class_like::member::ClassLikeMemberVisibilityReflection;
use crate::function_like::FunctionLikeReflection;
use crate::identifier::ClassLikeMemberName;
use crate::r#type::TypeReflection;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PropertyDefaultValueReflection {
    pub inferred_type_reflection: Option<TypeReflection>,
    pub span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct PropertyReflection {
    pub attribut_reflections: Vec<AttributeReflection>,
    pub read_visibility_reflection: Option<ClassLikeMemberVisibilityReflection>,
    pub write_visibility_reflection: Option<ClassLikeMemberVisibilityReflection>,
    pub name: ClassLikeMemberName,
    pub type_reflection: Option<TypeReflection>,
    pub default_value_reflection: Option<PropertyDefaultValueReflection>,
    pub hooks: HashMap<StringIdentifier, FunctionLikeReflection>,
    pub is_readonly: bool,
    pub is_final: bool,
    pub is_promoted: bool,
    pub is_static: bool,
    pub item_span: Span,
    pub definition_span: Span,
}
