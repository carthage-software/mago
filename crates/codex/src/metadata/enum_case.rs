use serde::Deserialize;
use serde::Serialize;

use mago_interner::StringIdentifier;
use mago_span::Span;

use crate::metadata::attribute::AttributeMetadata;
use crate::ttype::atomic::TAtomic;

/// Contains metadata associated with a specific `case` within a PHP `enum`.
///
/// Represents enum cases in both "pure" enums (e.g., `case Pending;` in `enum Status`)
/// and "backed" enums (e.g., `case Ok = 200;` in `enum HttpStatus: int`),
/// including associated attributes, values, and source locations.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnumCaseMetadata {
    /// Attributes attached to the enum case declaration (e.g., `#[Internal] case Draft;`).
    attributes: Vec<AttributeMetadata>,

    /// The identifier (name) of the enum case (e.g., `Ok`, `Pending`, `Admin`).
    name: StringIdentifier,

    /// The specific source code location (span) of the case name identifier itself.
    name_span: Span,

    /// The source code location (span) covering the entire case declaration statement.
    span: Span,

    /// If `is_backed` is true, this *may* hold the inferred atomic type of the specific
    /// scalar value assigned to this case (e.g., `int(200)` for `case Ok = 200;`).
    ///
    /// It can be `None` even for backed cases if the value's type could not be inferred
    /// during the initial scan (e.g., if the value is a constant defined elsewhere:
    /// `case Default = Config::DEFAULT_TIMEOUT;`). The overall backing type defined on the enum
    /// (`: int`, `: string`) provides the fundamental type information in such scenarios.
    ///
    /// If `is_backed` is false (a pure enum case like `case Pending;`), this must be `None`.
    value_type: Option<TAtomic>,

    /// `true` if this case belongs to a "backed" enum declaration (one defined with `: int` or `: string`,
    /// like `enum HttpStatus: int`).
    ///
    /// `false` if it belongs to a "pure" enum declaration (like `enum Status`).
    /// This is determined by the enum declaration itself.
    is_backed: bool,

    /// `true` if the enum case is marked as deprecated, typically via `@deprecated` docblock tag.
    is_deprecated: bool,
}

impl EnumCaseMetadata {
    /// Creates new `EnumCaseMetadata` for a case assumed initially to be non-backed (pure).
    ///
    /// Use modifier methods (`set_is_backed`, `with_is_backed`) later during analysis
    /// if the enum is determined to be backed.
    ///
    /// # Arguments
    /// * `name`: The identifier (name) of the enum case (e.g., `PENDING`).
    /// * `name_span`: The source code location of the name identifier.
    /// * `span`: The source code location of the entire case declaration.
    #[inline]
    pub fn new(name: StringIdentifier, name_span: Span, span: Span) -> Self {
        Self {
            attributes: Vec::new(),
            name,
            name_span,
            span,
            value_type: None,
            is_backed: false, // Assume pure initially
            is_deprecated: false,
        }
    }

    /// Returns a reference to the enum case name identifier (e.g., `OK`, `PENDING`).
    #[inline]
    pub fn get_name(&self) -> &StringIdentifier {
        &self.name
    }

    /// Returns the span of the enum case name identifier.
    #[inline]
    pub fn get_name_span(&self) -> Span {
        self.name_span
    }

    /// Returns the span covering the entire enum case declaration.
    #[inline]
    pub fn get_span(&self) -> Span {
        self.span
    }

    /// Returns a slice containing the attributes attached to the enum case.
    #[inline]
    pub fn get_attributes(&self) -> &[AttributeMetadata] {
        &self.attributes
    }

    /// Returns a reference to the inferred type of the specific value (`TAtomic`), if known.
    /// Returns `None` if the case is pure OR if it's a backed case whose value type
    /// couldn't be inferred during the initial scan.
    #[inline]
    pub fn get_value_type(&self) -> Option<&TAtomic> {
        self.value_type.as_ref()
    }

    /// Returns a mutable reference to the inferred type of the specific value (`TAtomic`), if known.
    /// Returns `None` if the case is pure OR if it's a backed case whose value type
    /// couldn't be inferred during the initial scan.
    #[inline]
    pub fn get_value_type_mut(&mut self) -> Option<&mut TAtomic> {
        self.value_type.as_mut()
    }

    /// Checks if this case belongs to a backed enum declaration (`enum E: int|string`).
    #[inline]
    pub fn is_backed(&self) -> bool {
        self.is_backed
    }

    /// Checks if this represents a case in a pure (non-backed) enum.
    #[inline]
    pub fn is_pure(&self) -> bool {
        !self.is_backed
    }

    /// Checks if the enum case is marked as deprecated.
    #[inline]
    pub fn is_deprecated(&self) -> bool {
        self.is_deprecated
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

    /// Sets the inferred scalar type of the specific value assigned to this case.
    /// Automatically clears the type if `is_backed` is currently `false`.
    #[inline]
    pub fn set_value_type(&mut self, value_type: Option<TAtomic>) {
        self.value_type = if self.is_backed { value_type } else { None };
    }

    /// Returns a new instance with the inferred value type set.
    /// Automatically clears the type if `is_backed` is `false`.
    #[inline]
    pub fn with_value_type(mut self, value_type: Option<TAtomic>) -> Self {
        self.set_value_type(value_type);
        self
    }

    /// Sets the inferred value type to `None`.
    #[inline]
    pub fn unset_value_type(&mut self) {
        self.value_type = None;
    }

    /// Returns a new instance with the inferred value type set to `None`.
    #[inline]
    pub fn without_value_type(mut self) -> Self {
        self.unset_value_type();
        self
    }

    /// Sets whether this case belongs to a backed enum.
    /// If set to `false`, also clears any previously inferred `value_type`.
    #[inline]
    pub fn set_is_backed(&mut self, is_backed: bool) {
        self.is_backed = is_backed;
        if !is_backed {
            self.value_type = None; // Invariant: Pure enums cannot have value type
        }
    }

    /// Returns a new instance with the `is_backed` flag set.
    /// If set to `false`, also clears any previously inferred `value_type`.
    #[inline]
    pub fn with_is_backed(mut self, is_backed: bool) -> Self {
        self.set_is_backed(is_backed);
        self
    }

    /// Sets whether the enum case is marked as deprecated.
    #[inline]
    pub fn set_is_deprecated(&mut self, is_deprecated: bool) {
        self.is_deprecated = is_deprecated;
    }

    /// Returns a new instance with the `is_deprecated` flag set.
    #[inline]
    pub fn with_is_deprecated(mut self, is_deprecated: bool) -> Self {
        self.set_is_deprecated(is_deprecated);
        self
    }
}
