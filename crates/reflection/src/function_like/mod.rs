use fennec_span::Span;
use serde::Deserialize;
use serde::Serialize;

use crate::attribute::AttributeReflection;
use crate::function_like::parameter::FunctionLikeParameterReflection;
use crate::function_like::r#return::FunctionLikeReturnTypeReflection;
use crate::identifier::FunctionLikeIdentifier;

pub mod parameter;
pub mod r#return;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct FunctionLikeReflection {
    pub name: FunctionLikeIdentifier,
    pub attribute_reflections: Vec<AttributeReflection>,
    pub parameter_reflections: Vec<FunctionLikeParameterReflection>,
    pub return_type_reflection: Option<FunctionLikeReturnTypeReflection>,
    pub returns_by_reference: bool,
    pub has_yield: bool,
    pub has_throws: bool,
    pub is_anonymous: bool,
    pub is_static: bool,
    pub is_final: bool,
    pub is_abstract: bool,
    pub is_overriding: bool,
    pub span: Span,
}
