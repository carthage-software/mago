use mago_php_version::PHPVersion;
use mago_php_version::PHPVersionRange;

use mago_span::HasSpan;
use mago_span::Span;
use mago_word::Word;

use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::ttype::TypeMetadata;
use crate::metadata::version_constraint::VersionConstraint;
use crate::ttype::atomic::TAtomic;
use crate::visibility::Visibility;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct ClassLikeConstantMetadata {
    pub attributes: Vec<AttributeMetadata>,
    pub name: Word,
    pub span: Span,
    pub visibility: Visibility,
    pub type_declaration: Option<TypeMetadata>,
    pub type_metadata: Option<TypeMetadata>,
    pub inferred_type: Option<TAtomic>,
    pub flags: MetadataFlags,
    pub version_constraint: VersionConstraint,
}

impl ClassLikeConstantMetadata {
    #[must_use]
    pub fn new(name: Word, span: Span, visibility: Visibility, flags: MetadataFlags) -> Self {
        Self {
            attributes: Vec::new(),
            name,
            span,
            visibility,
            type_declaration: None,
            type_metadata: None,
            inferred_type: None,
            flags,
            version_constraint: VersionConstraint::unconstrained(),
        }
    }

    pub fn set_type_declaration(&mut self, type_declaration: TypeMetadata) {
        if self.type_metadata.is_none() {
            self.type_metadata = Some(type_declaration.clone());
        }

        self.type_declaration = Some(type_declaration);
    }

    /// Returns `true` when this class constant is available in the given PHP
    /// version.
    #[inline]
    #[must_use]
    pub fn is_available_in_version(&self, version: PHPVersion) -> bool {
        self.version_constraint.allows_version(version)
    }

    /// Returns `true` when this class constant is available across the entire
    /// supplied [`PHPVersionRange`].
    #[inline]
    #[must_use]
    pub fn is_available_in_version_range(&self, range: PHPVersionRange) -> bool {
        self.version_constraint.allows_version_range(range)
    }
}

impl HasSpan for ClassLikeConstantMetadata {
    fn span(&self) -> Span {
        self.span
    }
}
