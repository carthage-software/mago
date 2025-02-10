use serde::Deserialize;
use serde::Serialize;

use mago_reporting::IssueCollection;
use mago_source::HasSource;
use mago_source::SourceIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

use crate::attribute::AttributeReflection;
use crate::class_like::member::ClassLikeMemberVisibilityReflection;
use crate::function_like::parameter::FunctionLikeParameterReflection;
use crate::function_like::r#return::FunctionLikeReturnTypeReflection;
use crate::identifier::FunctionLikeName;
use crate::r#type::kind::Template;
use crate::Reflection;

pub mod parameter;
pub mod r#return;

/// Represents reflection data for a function-like entity, such as a function or method.
///
/// This includes details about its parameters, return type, attributes, and various properties
/// like visibility, overrides, and whether it supports specific PHP features.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FunctionLikeReflection {
    /// Attributes associated with this function-like entity.
    pub attribute_reflections: Vec<AttributeReflection>,

    /// Visibility information for this function-like if it is a class member.
    pub visibility_reflection: Option<ClassLikeMemberVisibilityReflection>,

    /// The unique identifier for this function or method.
    pub name: FunctionLikeName,

    /// The list of templates accepted by this function or method.
    pub templates: Vec<Template>,

    /// The list of parameters accepted by this function or method, including their types and attributes.
    pub parameters: Vec<FunctionLikeParameterReflection>,

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

    /// Indicates if this function or method is pure.
    pub is_pure: bool,

    /// Flags if this function or method overrides a method from a parent class.
    ///
    /// Always `false` for functions, arrow functions, and closures. For methods,
    /// it depends on whether they override a parent method.
    pub is_overriding: bool,

    /// The span in the source code where this function or method is defined.
    pub span: Span,

    /// Indicate if this function-like entity is populated.
    pub is_populated: bool,

    /// Collection of issues associated with this function-like entity.
    pub issues: IssueCollection,
}

impl FunctionLikeReflection {
    /// Checks if this entity is a function.
    pub fn is_function(&self) -> bool {
        matches!(self.name, FunctionLikeName::Function(_))
    }

    /// Checks if this entity is a method.
    pub fn is_method(&self) -> bool {
        matches!(self.name, FunctionLikeName::Method(_, _))
    }

    /// Checks if this entity is a property hook.
    pub fn is_property_hook(&self) -> bool {
        matches!(self.name, FunctionLikeName::PropertyHook(_, _, _))
    }

    /// Checks if this entity is a closure.
    pub fn is_closure(&self) -> bool {
        matches!(self.name, FunctionLikeName::Closure(_))
    }

    /// Checks if this entity is an arrow function.
    pub fn is_arrow_function(&self) -> bool {
        matches!(self.name, FunctionLikeName::ArrowFunction(_))
    }
}

impl HasSpan for FunctionLikeReflection {
    /// Returns the span of the function-like entity in the source code.
    fn span(&self) -> Span {
        self.span
    }
}

impl HasSource for FunctionLikeReflection {
    /// Returns the source identifier of the file containing this function-like entity.
    fn source(&self) -> SourceIdentifier {
        self.span.source()
    }
}

impl Reflection for FunctionLikeReflection {
    /// Returns the source category of the function-like entity.
    fn get_category(&self) -> crate::SourceCategory {
        self.source().category()
    }

    /// Indicates whether the function-like entity's reflection data is fully populated.
    ///
    /// If `is_populated` is `false`, additional processing may be required to resolve
    /// all metadata for this entity.
    fn is_populated(&self) -> bool {
        self.is_populated
    }

    fn take_issues(&mut self) -> IssueCollection {
        std::mem::take(&mut self.issues)
    }
}
