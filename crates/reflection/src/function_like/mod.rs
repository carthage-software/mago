use serde::Deserialize;
use serde::Serialize;

use fennec_span::Span;

use crate::attribute::AttributeReflection;
use crate::class_like::member::ClassLikeMemberVisibilityReflection;
use crate::function_like::parameter::FunctionLikeParameterReflection;
use crate::function_like::r#return::FunctionLikeReturnTypeReflection;
use crate::identifier::FunctionLikeName;

pub mod parameter;
pub mod r#return;

/// Represents reflection data for a function-like entity, such as a function or method.
///
/// This includes details about its parameters, return type, attributes, and various properties
/// like visibility, overrides, and whether it supports specific PHP features.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct FunctionLikeReflection {
    /// Attributes associated with this function-like entity.
    pub attribute_reflections: Vec<AttributeReflection>,

    /// Visibility information for this function-like if it is a class member.
    pub visibility_reflection: Option<ClassLikeMemberVisibilityReflection>,

    /// The unique identifier for this function or method.
    pub name: FunctionLikeName,

    /// The list of parameters accepted by this function or method, including their types and attributes.
    pub parameter_reflections: Vec<FunctionLikeParameterReflection>,

    /// The return type of this function or method, if specified.
    pub return_type_reflection: Option<FunctionLikeReturnTypeReflection>,

    /// Indicates whether the function or method returns by reference.
    pub returns_by_reference: bool,

    /// Flags if the function or method contains a `yield` expression, indicating it is a generator.
    pub has_yield: bool,

    /// Flags if the function or method has the potential to throw an exception.
    pub has_throws: bool,

    /// Indicates if this function-like entity is anonymous (i.e., a closure or an anonymous function).
    ///
    /// For functions and methods, this is always `false`.
    pub is_anonymous: bool,

    /// Indicates if this function or method is static.
    ///
    /// This is always `false` for functions; for closures, arrow functions, and methods, it depends on their declaration.
    pub is_static: bool,

    /// Indicates if this function or method is declared as final.
    ///
    /// This is always `true` for functions, arrow functions, and closures. For methods, it depends on the declaration.
    pub is_final: bool,

    /// Indicates if this function or method is abstract.
    ///
    /// Always `false` for functions, arrow functions, and closures. For methods, it depends on the declaration.
    pub is_abstract: bool,

    /// Flags if this function or method overrides a method from a parent class.
    ///
    /// Always `false` for functions, arrow functions, and closures. For methods,
    /// it depends on whether they override a parent method.
    pub is_overriding: bool,

    /// The span in the source code where this function or method is defined.
    pub span: Span,

    /// Indicate if this function-like entity is populated.
    pub is_populated: bool,
}
