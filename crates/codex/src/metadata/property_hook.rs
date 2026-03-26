use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_reporting::Issue;
use mago_span::Span;

use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::metadata::parameter::FunctionLikeParameterMetadata;
use crate::metadata::ttype::TypeMetadata;

/// Metadata for a property hook (get or set).
///
/// PHP 8.4 introduced property hooks, which allow defining custom get/set behavior
/// for properties. This struct stores the metadata for a single hook.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PropertyHookMetadata {
    /// The hook name ("get" or "set").
    pub name: Atom,

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
    /// Creates a new `PropertyHookMetadata` with the given name and span.
    #[inline]
    #[must_use]
    pub fn new(name: Atom, span: Span) -> Self {
        Self {
            name,
            span,
            flags: MetadataFlags::empty(),
            parameter: None,
            returns_by_ref: false,
            is_abstract: false,
            attributes: Vec::new(),
            return_type_metadata: None,
            has_docblock: false,
            issues: Vec::new(),
        }
    }

    /// Returns whether this is a get hook.
    #[inline]
    #[must_use]
    pub fn is_get(&self) -> bool {
        self.name.as_str() == "get"
    }

    /// Returns whether this is a set hook.
    #[inline]
    #[must_use]
    pub fn is_set(&self) -> bool {
        self.name.as_str() == "set"
    }

    /// Sets the flags for this hook.
    #[inline]
    #[must_use]
    pub fn with_flags(mut self, flags: MetadataFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Sets the parameter for this hook (for set hooks).
    #[inline]
    #[must_use]
    pub fn with_parameter(mut self, parameter: Option<FunctionLikeParameterMetadata>) -> Self {
        self.parameter = parameter;
        self
    }

    /// Sets whether the hook returns by reference.
    #[inline]
    #[must_use]
    pub fn with_returns_by_ref(mut self, returns_by_ref: bool) -> Self {
        self.returns_by_ref = returns_by_ref;
        self
    }

    /// Sets whether this is an abstract hook.
    #[inline]
    #[must_use]
    pub fn with_is_abstract(mut self, is_abstract: bool) -> Self {
        self.is_abstract = is_abstract;
        self
    }

    /// Sets the attributes for this hook.
    #[inline]
    #[must_use]
    pub fn with_attributes(mut self, attributes: Vec<AttributeMetadata>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Sets the return type metadata from docblock (for get hooks).
    #[inline]
    #[must_use]
    pub fn with_return_type_metadata(mut self, return_type_metadata: Option<TypeMetadata>) -> Self {
        self.return_type_metadata = return_type_metadata;
        self
    }

    /// Sets whether this hook has a docblock.
    #[inline]
    #[must_use]
    pub fn with_has_docblock(mut self, has_docblock: bool) -> Self {
        self.has_docblock = has_docblock;
        self
    }

    /// Sets the issues from parsing the docblock.
    #[inline]
    #[must_use]
    pub fn with_issues(mut self, issues: Vec<Issue>) -> Self {
        self.issues = issues;
        self
    }

    /// Takes the issues, leaving an empty vector.
    #[inline]
    pub fn take_issues(&mut self) -> Vec<Issue> {
        std::mem::take(&mut self.issues)
    }
}
