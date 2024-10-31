use serde::Deserialize;
use serde::Serialize;

use fennec_span::Span;

use crate::identifier::Name;
use crate::r#type::TypeReflection;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct AttributeReflection {
    pub name: Name,
    pub arguments: Option<AttributeArgumentListReflection>,
    pub span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct AttributeArgumentListReflection {
    pub arguments: Vec<AttributeArgumentReflection>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum AttributeArgumentReflection {
    Positional { value_type_reflection: Option<TypeReflection>, span: Span },
    Named { name: Name, value_type_reflection: Option<TypeReflection>, span: Span },
}
