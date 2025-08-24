use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_atom::AtomMap;
use mago_reporting::Issue;
use mago_span::Span;

use crate::assertion::Assertion;
use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::parameter::FunctionLikeParameterMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::GenericParent;
use crate::ttype::resolution::TypeResolutionContext;
use crate::ttype::union::TUnion;
use crate::visibility::Visibility;

pub type TemplateTuple = (Atom, Vec<(GenericParent, TUnion)>);

/// Contains metadata specific to methods defined within classes, interfaces, enums, or traits.
///
/// This complements the more general `FunctionLikeMetadata`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MethodMetadata {
    /// Marks whether this method is declared as `final`, preventing further overriding.
    pub is_final: bool,

    /// Marks whether this method is declared as `abstract`, requiring implementation in subclasses.
    pub is_abstract: bool,

    /// Marks whether this method is declared as `static`, allowing it to be called without an instance.
    pub is_static: bool,

    /// Marks whether this method is a constructor (`__construct`).
    pub is_constructor: bool,

    /// Marks whether this method is declared as `public`, `protected`, or `private`.
    pub visibility: Visibility,

    /// A map of constraints defined by `@where` docblock tags.
    ///
    /// The key is the name of a class-level template parameter (e.g., `T`), and the value
    /// is the `TUnion` type constraint that `T` must satisfy for this specific method
    /// to be considered callable.
    pub where_constraints: AtomMap<TypeMetadata>,
}

/// Distinguishes between different kinds of callable constructs in PHP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FunctionLikeKind {
    /// Represents a standard function declared in the global scope or a namespace (`function foo() {}`).
    Function,
    /// Represents a method defined within a class, trait, enum, or interface (`class C { function bar() {} }`).
    Method,
    /// Represents an anonymous function created using `function() {}`.
    Closure,
    /// Represents an arrow function (short closure syntax) introduced in PHP 7.4 (`fn() => ...`).
    ArrowFunction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionLikeMetadata {
    /// The kind of function-like structure this metadata represents.
    pub kind: FunctionLikeKind,

    /// The source code location (span) covering the entire function/method/closure definition.
    /// For closures/arrow functions, this covers the `function(...) { ... }` or `fn(...) => ...` part.
    pub span: Span,

    /// The name of the function or method, lowercased, if applicable.
    /// `None` for closures and arrow functions unless assigned to a variable later.
    /// Example: `processRequest`, `__construct`, `my_global_func`.
    pub name: Option<Atom>,

    /// The original name of the function or method, in its original case.
    pub original_name: Option<Atom>,

    /// The specific source code location (span) of the function or method name identifier.
    /// `None` if the function/method has no name (closures/arrow functions).
    pub name_span: Option<Span>,

    /// Ordered list of metadata for each parameter defined in the signature.
    pub parameters: Vec<FunctionLikeParameterMetadata>,

    /// The explicit return type declaration (type hint).
    ///
    /// Example: For `function getName(): string`, this holds metadata for `string`.
    /// `None` if no return type is specified.
    pub return_type_declaration_metadata: Option<TypeMetadata>,

    /// The explicit return type declaration (type hint) or docblock type (`@return`).
    ///
    /// Example: For `function getName(): string`, this holds metadata for `string`,
    /// or for ` /** @return string */ function getName() { .. }`, this holds metadata for `string`.
    /// `None` if neither is specified.
    pub return_type_metadata: Option<TypeMetadata>,

    /// Generic type parameters (templates) defined for the function/method (e.g., `@template T`).
    /// Stores the template name and its constraints (parent type and bound type).
    /// Example: `[("T", [(GenericParent::Function("funcName"), Arc<TUnion::object()>)])]`
    pub template_types: Vec<TemplateTuple>,

    /// Attributes attached to the function/method/closure declaration (`#[Attribute] function foo() {}`).
    pub attributes: Vec<AttributeMetadata>,

    /// Specific metadata relevant only to methods (visibility, final, static, etc.).
    /// This is `Some` if `kind` is `FunctionLikeKind::Method`, `None` otherwise.
    pub method_metadata: Option<MethodMetadata>,

    /// Contains context information needed for resolving types within this function's scope
    /// (e.g., `use` statements, current namespace, class context). Often populated during analysis.
    pub type_resolution_context: Option<TypeResolutionContext>,

    /// A list of types that this function/method might throw, derived from `@throws` docblock tags
    /// or inferred from `throw` statements within the body.
    pub thrown_types: Vec<TypeMetadata>,

    /// List of issues specifically related to parsing or interpreting this function's docblock.
    pub issues: Vec<Issue>,

    /// Assertions about parameter types or variable types that are guaranteed to be true
    /// *after* this function/method returns normally. From `@psalm-assert`, `@phpstan-assert`, etc.
    /// Maps variable/parameter name to a list of type assertions.
    pub assertions: BTreeMap<Atom, Vec<Assertion>>,

    /// Assertions about parameter/variable types that are guaranteed to be true if this
    /// function/method returns `true`. From `@psalm-assert-if-true`, etc.
    pub if_true_assertions: BTreeMap<Atom, Vec<Assertion>>,

    /// Assertions about parameter/variable types that are guaranteed to be true if this
    /// function/method returns `false`. From `@psalm-assert-if-false`, etc.
    pub if_false_assertions: BTreeMap<Atom, Vec<Assertion>>,

    pub flags: MetadataFlags,
}

impl FunctionLikeKind {
    /// Checks if this kind represents a class/trait/enum/interface method.
    #[inline]
    pub const fn is_method(&self) -> bool {
        matches!(self, Self::Method)
    }

    /// Checks if this kind represents a globally/namespace-scoped function.
    #[inline]
    pub const fn is_function(&self) -> bool {
        matches!(self, Self::Function)
    }

    /// Checks if this kind represents an anonymous function (`function() {}`).
    #[inline]
    pub const fn is_closure(&self) -> bool {
        matches!(self, Self::Closure)
    }

    /// Checks if this kind represents an arrow function (`fn() => ...`).
    #[inline]
    pub const fn is_arrow_function(&self) -> bool {
        matches!(self, Self::ArrowFunction)
    }
}

/// Contains comprehensive metadata for any function-like structure in PHP.
impl FunctionLikeMetadata {
    /// Creates new `FunctionLikeMetadata` with basic information and default flags.
    pub fn new(kind: FunctionLikeKind, span: Span, flags: MetadataFlags) -> Self {
        let method_metadata = if kind.is_method() { Some(MethodMetadata::default()) } else { None };

        Self {
            kind,
            span,
            flags,
            name: None,
            original_name: None,
            name_span: None,
            parameters: vec![],
            return_type_declaration_metadata: None,
            return_type_metadata: None,
            template_types: vec![],
            attributes: vec![],
            method_metadata,
            type_resolution_context: None,
            thrown_types: vec![],
            assertions: BTreeMap::new(),
            if_true_assertions: BTreeMap::new(),
            if_false_assertions: BTreeMap::new(),
            issues: vec![],
        }
    }

    /// Returns the kind of function-like (Function, Method, Closure, ArrowFunction).
    #[inline]
    pub fn get_kind(&self) -> FunctionLikeKind {
        self.kind
    }

    /// Returns a mutable slice of the parameter metadata.
    #[inline]
    pub fn get_parameters_mut(&mut self) -> &mut [FunctionLikeParameterMetadata] {
        &mut self.parameters
    }

    /// Returns a reference to specific parameter metadata by name, if it exists.
    #[inline]
    pub fn get_parameter(&self, name: Atom) -> Option<&FunctionLikeParameterMetadata> {
        self.parameters.iter().find(|parameter| parameter.get_name().0 == name)
    }

    /// Returns a mutable reference to specific parameter metadata by name, if it exists.
    #[inline]
    pub fn get_parameter_mut(&mut self, name: Atom) -> Option<&mut FunctionLikeParameterMetadata> {
        self.parameters.iter_mut().find(|parameter| parameter.get_name().0 == name)
    }

    /// Returns a mutable slice of the template type parameters.
    #[inline]
    pub fn get_template_types_mut(&mut self) -> &mut [TemplateTuple] {
        &mut self.template_types
    }

    /// Returns a slice of the attributes.
    #[inline]
    pub fn get_attributes(&self) -> &[AttributeMetadata] {
        &self.attributes
    }

    /// Returns a mutable reference to the method-specific info, if this is a method.
    #[inline]
    pub fn get_method_metadata_mut(&mut self) -> Option<&mut MethodMetadata> {
        self.method_metadata.as_mut()
    }

    /// Returns a mutable slice of docblock issues.
    #[inline]
    pub fn take_issues(&mut self) -> Vec<Issue> {
        std::mem::take(&mut self.issues)
    }

    /// Sets the parameters, replacing existing ones.
    #[inline]
    pub fn set_parameters(&mut self, parameters: impl IntoIterator<Item = FunctionLikeParameterMetadata>) {
        self.parameters = parameters.into_iter().collect();
    }

    /// Returns a new instance with the parameters replaced.
    #[inline]
    pub fn with_parameters(mut self, parameters: impl IntoIterator<Item = FunctionLikeParameterMetadata>) -> Self {
        self.set_parameters(parameters);
        self
    }

    #[inline]
    pub fn set_return_type_metadata(&mut self, return_type: Option<TypeMetadata>) {
        self.return_type_metadata = return_type;
    }

    #[inline]
    pub fn set_return_type_declaration_metadata(&mut self, return_type: Option<TypeMetadata>) {
        if self.return_type_metadata.is_none() {
            self.return_type_metadata = return_type.clone();
        }

        self.return_type_declaration_metadata = return_type;
    }

    /// Adds a single template type definition.
    #[inline]
    pub fn add_template_type(&mut self, template: TemplateTuple) {
        self.template_types.push(template);
    }
}
