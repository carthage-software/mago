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
    pub attributes: Vec<AttributeMetadata>,

    /// The identifier (name) of the enum case (e.g., `Ok`, `Pending`, `Admin`).
    pub name: StringIdentifier,

    /// The specific source code location (span) of the case name identifier itself.
    pub name_span: Span,

    /// The source code location (span) covering the entire case declaration statement.
    pub span: Span,

    /// If `is_backed` is true, this *may* hold the inferred atomic type of the specific
    /// scalar value assigned to this case (e.g., `int(200)` for `case Ok = 200;`).
    ///
    /// It can be `None` even for backed cases if the value's type could not be inferred
    /// during the initial scan (e.g., if the value is a constant defined elsewhere:
    /// `case Default = Config::DEFAULT_TIMEOUT;`). The overall backing type defined on the enum
    /// (`: int`, `: string`) provides the fundamental type information in such scenarios.
    ///
    /// If `is_backed` is false (a pure enum case like `case Pending;`), this must be `None`.
    pub value_type: Option<TAtomic>,

    /// `true` if this case belongs to a "backed" enum declaration (one defined with `: int` or `: string`,
    /// like `enum HttpStatus: int`).
    ///
    /// `false` if it belongs to a "pure" enum declaration (like `enum Status`).
    /// This is determined by the enum declaration itself.
    pub is_backed: bool,

    /// `true` if the enum case is marked as deprecated, typically via `@deprecated` docblock tag.
    pub is_deprecated: bool,
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
}
