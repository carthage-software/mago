use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;

use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;
use mago_word::Word;

use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::ttype::TypeMetadata;
use crate::metadata::version_constraint::VersionConstraint;
use crate::ttype::union::TUnion;

/// Contains metadata associated with a global constant defined using `const`.
///
/// Represents a single constant declaration item, potentially within a grouped declaration,
/// like `MAX_RETRIES = 3` in `const MAX_RETRIES = 3;` or `B = 2` in `const A = 1, B = 2;`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct ConstantMetadata {
    pub attributes: Vec<AttributeMetadata>,
    pub name: Word,
    pub span: Span,
    pub type_metadata: Option<TypeMetadata>,
    pub inferred_type: Option<TUnion>,
    pub flags: MetadataFlags,
    pub issues: Vec<Issue>,
    pub version_constraint: VersionConstraint,
}

impl ConstantMetadata {
    /// Creates new `ConstantMetadata` for a non-deprecated, non-internal global constant item.
    ///
    /// # Arguments
    ///
    /// * `name`: The identifier (name) of the constant.
    /// * `span`: The source code location of this specific constant's definition item (`NAME = value`).
    #[inline]
    #[must_use]
    pub fn new(name: Word, span: Span, flags: MetadataFlags) -> Self {
        Self {
            attributes: Vec::new(),
            name,
            span,
            flags,
            type_metadata: None,
            inferred_type: None,
            issues: Vec::new(),
            version_constraint: VersionConstraint::unconstrained(),
        }
    }

    /// Returns a mutable slice of docblock issues.
    #[inline]
    pub fn take_issues(&mut self) -> Vec<Issue> {
        std::mem::take(&mut self.issues)
    }

    /// Returns `true` when this constant is available in the given PHP version.
    #[inline]
    #[must_use]
    pub fn is_available_in_version(&self, version: PHPVersion) -> bool {
        self.version_constraint.allows_version(version)
    }

    /// Returns `true` when this constant is available across the entire
    /// supplied [`PHPVersionRange`].
    #[inline]
    #[must_use]
    pub fn is_available_in_version_range(&self, range: PHPVersionRange) -> bool {
        self.version_constraint.allows_version_range(range)
    }

    /// Applies a patch to this entry in place, refining type information.
    pub fn apply_patch(&mut self, patch: &ConstantMetadata) {
        if patch.type_metadata.is_some() {
            self.type_metadata.clone_from(&patch.type_metadata);
        }
    }
}

impl HasSpan for ConstantMetadata {
    fn span(&self) -> Span {
        self.span
    }
}
