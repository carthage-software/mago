use serde::Deserialize;
use serde::Serialize;

use mago_span::Span;

use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::ttype::TypeMetadata;
use crate::misc::VariableIdentifier;

/// Contains metadata associated with a single parameter within a function, method, or closure signature.
///
/// This captures details like the parameter's name, type hint, attributes, default value,
/// pass-by-reference status, variadic nature, and other PHP features like property promotion.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionLikeParameterMetadata {
    /// Attributes attached to the parameter declaration.
    pub attributes: Vec<AttributeMetadata>,

    /// The identifier (name) of the parameter, including the leading '$'.
    pub name: VariableIdentifier,

    /// `true` if the parameter is declared to be passed by reference using `&`.
    pub is_by_reference: bool,

    /// The explicit type declaration (type hint) or docblock type (`@param`).
    ///
    /// Can be `None` if no type is specified.
    pub type_metadata: Option<TypeMetadata>,

    /// `true` if the parameter has a default value specified in the signature.
    pub has_default: bool,

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

    /// `true` if the parameter is variadic, declared using `...`.
    pub is_variadic: bool,

    /// `true` if this parameter uses constructor property promotion (PHP 8.0+).
    pub is_promoted_property: bool,

    /// `true` if the parameter is marked as deprecated.
    pub is_deprecated: bool,
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
    pub fn new(name: VariableIdentifier, span: Span, name_span: Span) -> Self {
        Self {
            attributes: Vec::new(),
            name,
            is_by_reference: false,
            type_metadata: None,
            has_default: false,
            out_type: None,
            default_type: None,
            span,
            name_span,
            is_variadic: false,
            is_promoted_property: false,
            is_deprecated: false,
        }
    }

    /// Returns a reference to the parameter's name identifier (e.g., `$userId`).
    #[inline]
    pub fn get_name(&self) -> &VariableIdentifier {
        &self.name
    }

    /// Returns the span covering the entire parameter declaration.
    #[inline]
    pub fn get_span(&self) -> Span {
        self.span
    }

    /// Returns the span covering the parameter's name identifier.
    #[inline]
    pub fn get_name_span(&self) -> Span {
        self.name_span
    }

    /// Returns a slice containing the attributes attached to the parameter.
    #[inline]
    pub fn get_attributes(&self) -> &[AttributeMetadata] {
        &self.attributes
    }

    /// Returns a reference to the explicit type signature, if available.
    #[inline]
    pub fn get_type_metadata(&self) -> Option<&TypeMetadata> {
        self.type_metadata.as_ref()
    }

    /// Returns a mutable reference to the explicit type signature, if available.
    #[inline]
    pub fn get_type_signature_mut(&mut self) -> Option<&mut TypeMetadata> {
        self.type_metadata.as_mut()
    }

    /// Returns a reference to the `@param-out` type, if specified and relevant.
    #[inline]
    pub fn get_out_type(&self) -> Option<&TypeMetadata> {
        self.out_type.as_ref()
    }

    /// Returns a mutable reference to the `@param-out` type, if specified and relevant.
    #[inline]
    pub fn get_out_type_mut(&mut self) -> Option<&mut TypeMetadata> {
        self.out_type.as_mut()
    }

    /// Returns a reference to the inferred type of the default value, if known.
    #[inline]
    pub fn get_default_type(&self) -> Option<&TypeMetadata> {
        self.default_type.as_ref()
    }

    /// Returns a mutable reference to the inferred type of the default value, if known.
    #[inline]
    pub fn get_default_type_mut(&mut self) -> Option<&mut TypeMetadata> {
        self.default_type.as_mut()
    }

    /// Checks if the parameter is passed by reference (`&`).
    #[inline]
    pub const fn is_by_reference(&self) -> bool {
        self.is_by_reference
    }

    /// Checks if the parameter has a default value specified in the signature.
    #[inline]
    pub const fn has_default(&self) -> bool {
        self.has_default
    }

    /// Checks if the parameter is variadic (`...`).
    #[inline]
    pub const fn is_variadic(&self) -> bool {
        self.is_variadic
    }

    /// Checks if the parameter is a promoted property (constructor property promotion).
    #[inline]
    pub fn is_promoted_property(&self) -> bool {
        self.is_promoted_property
    }

    /// Checks if the parameter is marked as deprecated.
    #[inline]
    pub fn is_deprecated(&self) -> bool {
        self.is_deprecated
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

    /// Sets whether the parameter is passed by reference (`&`). Clears `out_type` if set to `false`.
    pub fn set_is_by_reference(&mut self, is_by_reference: bool) {
        self.is_by_reference = is_by_reference;
        if !is_by_reference {
            self.out_type = None; // Invariant
        }
    }

    /// Returns a new instance with the `is_by_reference` flag set. Clears `out_type` if set to `false`.
    pub fn with_is_by_reference(mut self, is_by_reference: bool) -> Self {
        self.set_is_by_reference(is_by_reference);
        self
    }

    /// Sets the explicit type signature (type hint or `@param` type).
    pub fn set_type_signature(&mut self, type_signature: Option<TypeMetadata>) {
        self.type_metadata = type_signature;
    }

    /// Returns a new instance with the explicit type signature set.
    pub fn with_type_signature(mut self, type_signature: Option<TypeMetadata>) -> Self {
        self.set_type_signature(type_signature);
        self
    }

    /// Sets the type signature to `None`.
    pub fn unset_type_signature(&mut self) {
        self.type_metadata = None;
    }

    /// Returns a new instance with the type signature set to `None`.
    pub fn without_type_signature(mut self) -> Self {
        self.unset_type_signature();
        self
    }

    /// Sets whether the parameter has a default value. Use with caution if also setting `default_type`.
    pub fn set_has_default(&mut self, has_default: bool) {
        self.has_default = has_default;
        // If setting to false, should default_type also be cleared? Let's assume yes for consistency.
        if !has_default {
            self.default_type = None;
        }
    }

    /// Returns a new instance with the `has_default` flag set. Clears `default_type` if set to `false`.
    pub fn with_has_default(mut self, has_default: bool) -> Self {
        self.set_has_default(has_default);
        self
    }

    /// Sets the `@param-out` type. Only effective if `is_by_reference` is true.
    pub fn set_out_type(&mut self, out_type: Option<TypeMetadata>) {
        if self.is_by_reference {
            self.out_type = out_type;
        } else {
            // Setting an out_type on non-reference is ignored or cleared
            self.out_type = None;
        }
    }

    /// Sets the inferred type of the default value. Also sets `has_default` to `true` if `Some`.
    pub fn set_default_type(&mut self, default_type: Option<TypeMetadata>) {
        self.default_type = default_type;
        // If we know the type, it must have a default. If type is None, maybe it still has a default?
        // Let's only set has_default=true if default_type is Some.
        // To clear has_default, use set_has_default(false).
        if self.default_type.is_some() {
            self.has_default = true;
        }
        // If default_type is None, we *don't* automatically set has_default to false,
        // because the syntax might exist (`= value`) but inference failed.
    }

    /// Returns a new instance with the inferred type of the default value set. Also sets `has_default` to `true` if `Some`.
    pub fn with_default_type(mut self, default_type: Option<TypeMetadata>) -> Self {
        self.set_default_type(default_type);
        self
    }

    /// Sets whether the parameter is variadic (`...`).
    pub fn set_is_variadic(&mut self, is_variadic: bool) {
        self.is_variadic = is_variadic;
    }

    /// Returns a new instance with the `is_variadic` flag set.
    pub fn with_is_variadic(mut self, is_variadic: bool) -> Self {
        self.set_is_variadic(is_variadic);
        self
    }

    /// Sets whether the parameter is a promoted property.
    pub fn set_is_promoted_property(&mut self, is_promoted: bool) {
        self.is_promoted_property = is_promoted;
    }

    /// Returns a new instance with the `is_promoted_property` flag set.
    pub fn with_is_promoted_property(mut self, is_promoted: bool) -> Self {
        self.set_is_promoted_property(is_promoted);
        self
    }

    /// Sets whether the parameter is marked as deprecated.
    pub fn set_is_deprecated(&mut self, is_deprecated: bool) {
        self.is_deprecated = is_deprecated;
    }

    /// Returns a new instance with the `is_deprecated` flag set.
    pub fn with_is_deprecated(mut self, is_deprecated: bool) -> Self {
        self.set_is_deprecated(is_deprecated);
        self
    }
}
