use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::VariableIdentifier;

/// Contains metadata associated with a single parameter within a function, method, or closure signature.
///
/// This captures details like the parameter's name, type hint, attributes, default value,
/// pass-by-reference status, variadic nature, and other PHP features like property promotion.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct FunctionLikeParameterMetadata {
    /// Attributes attached to the parameter declaration.
    pub attributes: Vec<AttributeMetadata>,

    /// The identifier (name) of the parameter, including the leading '$'.
    pub name: VariableIdentifier,

    /// The native type declaration from the function signature.
    ///
    /// This is the type hint specified in the code (e.g., `string $name`), not from docblocks.
    /// Can be `None` if no type hint is specified in the signature.
    pub type_declaration_metadata: Option<TypeMetadata>,

    /// The explicit type declaration (type hint) or docblock type (`@param`).
    ///
    /// If there's a docblock `@param` annotation, this will contain that type (with `from_docblock=true`).
    /// Otherwise, this will be the same as `type_declaration_metadata`.
    /// Can be `None` if no type is specified.
    pub type_metadata: Option<TypeMetadata>,

    /// The type specified by a `@param-out` docblock tag.
    ///
    /// This indicates the expected type of a pass-by-reference parameter *after* the function executes.
    pub out_type: Option<TypeMetadata>,

    /// The inferred type of the parameter's default value, if `has_default` is true and the
    /// type could be determined.
    ///
    /// `None` if there is no default or the default value's type couldn't be inferred.
    pub default_type: Option<TypeMetadata>,

    /// The source code location (span) covering the entire parameter declaration.
    pub span: Span,

    /// The specific source code location (span) of the parameter's name identifier.
    pub name_span: Span,

    /// Flags indicating various properties of the parameter.
    pub flags: MetadataFlags,
}

/// Contains metadata associated with a single parameter within a function, method, or closure signature.
///
/// This captures details like the parameter's name, type hint, attributes, default value,
/// pass-by-reference status, variadic nature, and other PHP features like property promotion.
impl FunctionLikeParameterMetadata {
    /// Creates new `FunctionLikeParameterMetadata` for a basic parameter.
    /// Initializes most flags to false and optional fields to None.
    ///
    /// # Arguments
    ///
    /// * `name`: The identifier (name) of the parameter (e.g., `$userId`).
    /// * `span`: The source code location covering the entire parameter declaration.
    /// * `name_span`: The source code location of the parameter's name identifier (`$userId`).
    #[must_use]
    pub fn new(name: VariableIdentifier, span: Span, name_span: Span, flags: MetadataFlags) -> Self {
        Self {
            attributes: Vec::new(),
            name,
            flags,
            span,
            name_span,
            type_declaration_metadata: None,
            type_metadata: None,
            out_type: None,
            default_type: None,
        }
    }

    /// Returns a reference to the parameter's name identifier (e.g., `$userId`).
    #[inline]
    #[must_use]
    pub fn get_name(&self) -> &VariableIdentifier {
        &self.name
    }

    /// Returns the span covering the entire parameter declaration.
    #[inline]
    #[must_use]
    pub fn get_span(&self) -> Span {
        self.span
    }

    /// Returns the span covering the parameter's name identifier.
    #[inline]
    #[must_use]
    pub fn get_name_span(&self) -> Span {
        self.name_span
    }

    /// Returns a reference to the parameter's type metadata (effective type with docblock).
    #[inline]
    #[must_use]
    pub fn get_type_metadata(&self) -> Option<&TypeMetadata> {
        self.type_metadata.as_ref()
    }

    /// Returns a reference to the parameter's native type declaration metadata.
    #[inline]
    #[must_use]
    pub fn get_type_declaration_metadata(&self) -> Option<&TypeMetadata> {
        self.type_declaration_metadata.as_ref()
    }

    /// Returns a reference to the inferred type of the default value, if known.
    #[inline]
    #[must_use]
    pub fn get_default_type(&self) -> Option<&TypeMetadata> {
        self.default_type.as_ref()
    }

    /// Sets the attributes, replacing any existing ones.
    pub fn set_attributes(&mut self, attributes: impl IntoIterator<Item = AttributeMetadata>) {
        self.attributes = attributes.into_iter().collect();
    }

    /// Returns a new instance with the attributes replaced.
    pub fn with_attributes(mut self, attributes: impl IntoIterator<Item = AttributeMetadata>) -> Self {
        self.set_attributes(attributes);
        self
    }

    /// Sets the parameter's type metadata (effective type with docblock).
    #[inline]
    pub fn set_type_metadata(&mut self, type_metadata: Option<TypeMetadata>) {
        self.type_metadata = type_metadata;
    }

    /// Sets the parameter's native type declaration metadata.
    ///
    /// If `type_metadata` is not set, it will be initialized with the same value.
    #[inline]
    pub fn set_type_declaration_metadata(&mut self, type_declaration: Option<TypeMetadata>) {
        if self.type_metadata.is_none() {
            self.type_metadata.clone_from(&type_declaration);
        }

        self.type_declaration_metadata = type_declaration;
    }
}

impl HasSpan for FunctionLikeParameterMetadata {
    fn span(&self) -> Span {
        self.span
    }
}
