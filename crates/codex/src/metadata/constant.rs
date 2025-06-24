use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_span::Span;

use crate::metadata::attribute::AttributeMetadata;
use crate::ttype::union::TUnion;

/// Contains metadata associated with a global constant defined using `const`.
///
/// Represents a single constant declaration item, potentially within a grouped declaration,
/// like `MAX_RETRIES = 3` in `const MAX_RETRIES = 3;` or `B = 2` in `const A = 1, B = 2;`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConstantMetadata {
    /// Attributes attached to the constant definition (e.g., `#[Internal] const VERSION = '1.0';`).
    attributes: Vec<AttributeMetadata>,

    /// The identifier (name) of the constant (e.g., `MAX_RETRIES`, `CONFIG_DIR`).
    name: StringIdentifier,

    /// The source code location (span) covering this specific constant's definition item
    /// (i.e., `NAME = value`).
    ///
    /// In grouped constant declarations like `const A = 1, B = 2;`, this span refers
    /// only to the relevant item (`A = 1` or `B = 2`), *not* the entire `const ...;` statement.
    span: Span,

    /// The inferred atomic type of the constant's value, if successfully determined.
    ///
    /// Examples:
    /// - `int(3)` for `const MAX_RETRIES = 3;`
    /// - `string("/path/to/config")` for `const CONFIG_DIR = __DIR__ . "/config";` (if evaluable)
    ///
    /// It might be `None` if the value involves complex expressions, function calls,
    /// or other elements whose type cannot be determined statically during the initial scan.
    inferred_type: Option<TUnion>,

    /// `true` if the constant is marked as deprecated, typically via `@deprecated` docblock tag
    /// associated with its specific declaration item.
    is_deprecated: bool,

    /// `true` if the constant is marked as `internal`, typically via `@internal` docblock tag.
    is_internal: bool,
}

impl ConstantMetadata {
    /// Creates new `ConstantMetadata` for a non-deprecated, non-internal global constant item.
    ///
    /// # Arguments
    ///
    /// * `name`: The identifier (name) of the constant.
    /// * `span`: The source code location of this specific constant's definition item (`NAME = value`).
    #[inline]
    pub fn new(name: StringIdentifier, span: Span) -> Self {
        Self { attributes: Vec::new(), name, span, inferred_type: None, is_deprecated: false, is_internal: false }
    }

    /// Returns a reference to the constant's name identifier.
    #[inline]
    pub fn get_name(&self) -> &StringIdentifier {
        &self.name
    }

    /// Returns the span covering this specific constant's definition item (`NAME = value`).
    #[inline]
    pub fn get_span(&self) -> Span {
        self.span
    }

    /// Returns a slice containing the attributes attached to the constant.
    #[inline]
    pub fn get_attributes(&self) -> &[AttributeMetadata] {
        &self.attributes
    }

    /// Returns a reference to the inferred type (`TUnion`) of the constant's value, if known.
    #[inline]
    pub fn get_inferred_type(&self) -> Option<&TUnion> {
        self.inferred_type.as_ref()
    }

    /// Returns a mutable reference to the inferred type (`TUnion`) of the constant's value, if known.
    #[inline]
    pub fn get_inferred_type_mut(&mut self) -> Option<&mut TUnion> {
        self.inferred_type.as_mut()
    }

    /// Checks if the constant is marked as deprecated.
    #[inline]
    pub fn is_deprecated(&self) -> bool {
        self.is_deprecated
    }

    /// Checks if the constant is marked as internal.
    #[inline]
    pub fn is_internal(&self) -> bool {
        self.is_internal
    }

    /// Sets the attributes, replacing existing ones.
    #[inline]
    pub fn set_attributes(&mut self, attributes: impl IntoIterator<Item = AttributeMetadata>) {
        self.attributes = attributes.into_iter().collect();
    }

    /// Returns a new instance with the attributes replaced.
    #[inline]
    pub fn with_attributes(mut self, attributes: impl IntoIterator<Item = AttributeMetadata>) -> Self {
        self.set_attributes(attributes);
        self
    }

    /// Adds a single attribute.
    #[inline]
    pub fn add_attribute(&mut self, attribute: AttributeMetadata) {
        self.attributes.push(attribute);
    }

    /// Returns a new instance with the attribute added.
    #[inline]
    pub fn with_added_attribute(mut self, attribute: AttributeMetadata) -> Self {
        self.add_attribute(attribute);
        self
    }

    /// Adds multiple attributes.
    #[inline]
    pub fn add_attributes(&mut self, attributes: impl IntoIterator<Item = AttributeMetadata>) {
        self.attributes.extend(attributes);
    }

    /// Returns a new instance with the attributes added.
    #[inline]
    pub fn with_added_attributes(mut self, attributes: impl IntoIterator<Item = AttributeMetadata>) -> Self {
        self.add_attributes(attributes);
        self
    }

    /// Clears all attributes.
    #[inline]
    pub fn unset_attributes(&mut self) {
        self.attributes.clear();
    }

    /// Returns a new instance with no attributes.
    #[inline]
    pub fn without_attributes(mut self) -> Self {
        self.unset_attributes();
        self
    }

    /// Sets the inferred type of the constant's value.
    #[inline]
    pub fn set_inferred_type(&mut self, inferred_type: Option<TUnion>) {
        self.inferred_type = inferred_type;
    }

    /// Returns a new instance with the inferred type set.
    #[inline]
    pub fn with_inferred_type(mut self, inferred_type: Option<TUnion>) -> Self {
        self.set_inferred_type(inferred_type);
        self
    }

    /// Sets the inferred type to `None`.
    #[inline]
    pub fn unset_inferred_type(&mut self) {
        self.inferred_type = None;
    }

    /// Returns a new instance with the inferred type set to `None`.
    #[inline]
    pub fn without_inferred_type(mut self) -> Self {
        self.unset_inferred_type();
        self
    }

    /// Sets whether the constant is marked as deprecated.
    #[inline]
    pub fn set_is_deprecated(&mut self, is_deprecated: bool) {
        self.is_deprecated = is_deprecated;
    }

    /// Returns a new instance with the deprecated flag set.
    #[inline]
    pub fn with_is_deprecated(mut self, is_deprecated: bool) -> Self {
        self.set_is_deprecated(is_deprecated);
        self
    }

    /// Sets whether the constant is marked as internal.
    #[inline]
    pub fn set_is_internal(&mut self, is_internal: bool) {
        self.is_internal = is_internal;
    }

    /// Returns a new instance with the internal flag set.
    #[inline]
    pub fn with_is_internal(mut self, is_internal: bool) -> Self {
        self.set_is_internal(is_internal);
        self
    }
}
