use mago_span::HasSpan;
use mago_span::Span;
use mago_word::Word;

use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::version_constraint::VersionConstraint;
use crate::ttype::atomic::TAtomic;

/// Contains metadata associated with a specific `case` within a PHP `enum`.
///
/// Represents enum cases in both "pure" enums (e.g., `case Pending;` in `enum Status`)
/// and "backed" enums (e.g., `case Ok = 200;` in `enum HttpStatus: int`),
/// including associated attributes, values, and source locations.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct EnumCaseMetadata {
    pub attributes: Vec<AttributeMetadata>,
    pub name: Word,
    pub name_span: Span,
    pub span: Span,
    pub value_type: Option<TAtomic>,
    pub flags: MetadataFlags,
    /// PHP version range in which this enum case is available, derived from
    /// `Mago\AvailableSince` / `Mago\AvailableUntil` attributes during
    /// scanning.
    pub version_constraint: VersionConstraint,
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
    #[must_use]
    pub fn new(name: Word, name_span: Span, span: Span, flags: MetadataFlags) -> Self {
        Self {
            attributes: Vec::new(),
            name,
            name_span,
            span,
            flags,
            value_type: None,
            version_constraint: VersionConstraint::unconstrained(),
        }
    }

    /// Returns `true` when this enum case is available in the given PHP version.
    #[inline]
    #[must_use]
    pub fn is_available_in_version(&self, version: mago_php_version::PHPVersion) -> bool {
        self.version_constraint.allows_version(version)
    }

    /// Returns `true` when this enum case is available across the entire
    /// supplied [`PHPVersionRange`].
    #[inline]
    #[must_use]
    pub fn is_available_in_version_range(&self, range: mago_php_version::PHPVersionRange) -> bool {
        self.version_constraint.allows_version_range(range)
    }
}

impl HasSpan for EnumCaseMetadata {
    fn span(&self) -> Span {
        self.span
    }
}
