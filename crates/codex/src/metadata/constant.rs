use serde::Deserialize;
use serde::Serialize;

use mago_atom::Atom;
use mago_reporting::Issue;
use mago_span::HasSpan;
use mago_span::Span;

use crate::metadata::attribute::AttributeMetadata;
use crate::metadata::flags::MetadataFlags;
use crate::ttype::union::TUnion;

/// Contains metadata associated with a global constant defined using `const`.
///
/// Represents a single constant declaration item, potentially within a grouped declaration,
/// like `MAX_RETRIES = 3` in `const MAX_RETRIES = 3;` or `B = 2` in `const A = 1, B = 2;`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConstantMetadata {
    pub attributes: Vec<AttributeMetadata>,
    pub name: Atom,
    pub span: Span,
    pub inferred_type: Option<TUnion>,
    pub flags: MetadataFlags,
    pub issues: Vec<Issue>,
}

impl ConstantMetadata {
    /// Creates new `ConstantMetadata` for a non-deprecated, non-internal global constant item.
    ///
    /// # Arguments
    ///
    /// * `name`: The identifier (name) of the constant.
    /// * `span`: The source code location of this specific constant's definition item (`NAME = value`).
    #[inline]
    pub fn new(name: Atom, span: Span, flags: MetadataFlags) -> Self {
        Self { attributes: Vec::new(), name, span, flags, inferred_type: None, issues: Vec::new() }
    }

    /// Returns a mutable slice of docblock issues.
    #[inline]
    pub fn take_issues(&mut self) -> Vec<Issue> {
        std::mem::take(&mut self.issues)
    }
}

impl HasSpan for ConstantMetadata {
    fn span(&self) -> Span {
        self.span
    }
}
