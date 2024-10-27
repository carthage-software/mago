use std::collections::HashSet;

use property::PropertyReflection;
use serde::Deserialize;
use serde::Serialize;

use fennec_span::Span;

use crate::attribute::AttributeReflection;
use crate::class_like::constant::ClassLikeConstantReflection;
use crate::class_like::enum_case::EnumCaseReflection;
use crate::class_like::inheritance::InheritanceReflection;
use crate::class_like::member::MemeberCollection;
use crate::function_like::FunctionLikeReflection;
use crate::identifier::ClassLikeIdentifier;
use crate::r#type::TypeReflection;

pub mod constant;
pub mod enum_case;
pub mod inheritance;
pub mod member;
pub mod property;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClassLikeReflection {
    pub attribute_reflections: Vec<AttributeReflection>,
    pub name: ClassLikeIdentifier,
    pub inheritance_reflection: InheritanceReflection,
    pub constant_reflections: MemeberCollection<ClassLikeConstantReflection>,
    pub case_reflections: MemeberCollection<EnumCaseReflection>,
    pub property_reflections: MemeberCollection<PropertyReflection>,
    pub method_reflections: MemeberCollection<FunctionLikeReflection>,
    pub used_traits: HashSet<ClassLikeIdentifier>,
    pub backing_type_reflection: Option<TypeReflection>,
    pub is_final: bool,
    pub is_readonly: bool,
    pub is_abstract: bool,
    pub span: Span,
}
