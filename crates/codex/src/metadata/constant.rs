use mago_reporting::Issue;
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
    pub attributes: Vec<AttributeMetadata>,
    pub name: StringIdentifier,
    pub span: Span,
    pub inferred_type: Option<TUnion>,
    pub is_deprecated: bool,
    pub is_internal: bool,
    pub user_defined: bool,
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
    pub fn new(name: StringIdentifier, span: Span) -> Self {
        Self {
            attributes: Vec::new(),
            name,
            span,
            inferred_type: None,
            is_deprecated: false,
            is_internal: false,
            user_defined: span.start.source.category().is_user_defined(),
            issues: Vec::new(),
        }
    }

    /// Returns a mutable slice of docblock issues.
    #[inline]
    pub fn take_issues(&mut self) -> Vec<Issue> {
        std::mem::take(&mut self.issues)
    }
}
