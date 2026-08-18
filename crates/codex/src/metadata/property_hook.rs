use mago_reporting::Issue;
use mago_span::Span;
use mago_word::Word;

use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::parameter::FunctionLikeParameterMetadata;
use crate::metadata::ttype::TypeMetadata;

/// Metadata for a property hook (get or set).
///
/// PHP 8.4 introduced property hooks, which allow defining custom get/set behavior
/// for properties. This struct stores the metadata for a single hook.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct PropertyHookMetadata {
    /// The hook name ("get" or "set").
    pub name: Word,

    /// Span of the hook declaration.
    pub span: Span,

    /// Hook modifiers (final, etc.).
    pub flags: MetadataFlags,

    /// For set hooks: the parameter (explicit or implicit $value).
    /// None for get hooks.
    pub parameter: Option<FunctionLikeParameterMetadata>,

    /// Whether the hook returns by reference (&get).
    pub returns_by_ref: bool,

    /// Whether this is an abstract hook (no body, just semicolon).
    pub is_abstract: bool,

    /// Attributes on the hook.
    pub attributes: Vec<AttributeMetadata>,

    /// Return type from @return docblock (for get hooks).
    pub return_type_metadata: Option<TypeMetadata>,

    /// Whether this hook has a docblock comment.
    pub has_docblock: bool,

    /// Issues from parsing the docblock.
    pub issues: Vec<Issue>,
}

impl PropertyHookMetadata {
    /// Returns whether this is a get hook.
    #[inline]
    #[must_use]
    pub fn is_get(&self) -> bool {
        self.name.as_bytes() == b"get"
    }

    /// Takes the issues, leaving an empty vector.
    #[inline]
    pub fn take_issues(&mut self) -> Vec<Issue> {
        std::mem::take(&mut self.issues)
    }
}
