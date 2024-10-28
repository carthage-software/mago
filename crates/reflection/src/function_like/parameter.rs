use serde::Deserialize;
use serde::Serialize;

use fennec_interner::StringIdentifier;
use fennec_span::Span;

use crate::attribute::AttributeReflection;
use crate::r#type::TypeReflection;

/// Represents the default value of a function-like parameter, including its inferred type
/// and the source code span where it is defined.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct FunctionLikeParameterDefaultValueReflection {
    /// The inferred type of the default value, if applicable.
    /// This type may be determined based on the default value itself.
    pub type_reflection: Option<TypeReflection>,

    /// The span in the source code where the default value is located.
    pub span: Span,
}

/// Represents a parameter in a function-like entity (such as a function or method),
/// including its type, attributes, and various properties.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct FunctionLikeParameterReflection {
    /// Attributes associated with the parameter, such as annotations or metadata.
    pub attribute_reflections: Vec<AttributeReflection>,

    /// The type of the parameter, if specified.
    pub type_reflection: Option<TypeReflection>,

    /// The name identifier of the parameter.
    pub name: StringIdentifier,

    /// Indicates whether the parameter accepts a variable number of arguments.
    pub is_variadic: bool,

    /// Indicates whether the parameter is passed by reference.
    pub is_passed_by_reference: bool,

    /// Indicates whether the parameter promotes a property in a constructor, typically used in PHP class constructors.
    pub is_promoted_property: bool,

    /// The default value of the parameter, if any, including its type and span in the source code.
    pub default: Option<FunctionLikeParameterDefaultValueReflection>,
}
