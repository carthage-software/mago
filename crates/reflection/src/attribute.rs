use serde::Deserialize;
use serde::Serialize;

use fennec_interner::StringIdentifier;
use fennec_span::Span;

use crate::r#type::TypeReflection;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct AttributeReflection {
    pub name: StringIdentifier,
    pub arguments: Option<AttributeArgumentListReflection>,
    pub span: Span,
    pub name_span: Span,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct AttributeArgumentListReflection {
    pub arguments: Vec<AttributeArgumentReflection>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum AttributeArgumentReflection {
    Positional {
        value_type_reflection: Option<TypeReflection>,
        span: Span,
    },
    Named {
        name: StringIdentifier,
        value_type_reflection: Option<TypeReflection>,
        name_span: Span,
        span: Span,
    },
}
