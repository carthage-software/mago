use serde::Deserialize;
use serde::Serialize;

use fennec_interner::StringIdentifier;
use fennec_span::Span;

use crate::attribute::AttributeReflection;
use crate::r#type::TypeReflection;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct FunctionLikeParameterDefaultValueReflection {
    pub inferred_type_reflection: Option<TypeReflection>,
    pub span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct FunctionLikeParameterReflection {
    pub attribute_reflections: Vec<AttributeReflection>,
    pub type_reflection: Option<TypeReflection>,
    pub name: StringIdentifier,
    pub is_variadic: bool,
    pub is_passed_by_reference: bool,
    pub is_promoted_property: bool,
    pub default: Option<FunctionLikeParameterDefaultValueReflection>,
}
