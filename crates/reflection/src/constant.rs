use serde::Deserialize;
use serde::Serialize;

use mago_reporting::IssueCollection;
use mago_source::HasSource;
use mago_source::SourceCategory;
use mago_source::SourceIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

use crate::Reflection;
use crate::attribute::AttributeReflection;
use crate::identifier::Name;
use crate::r#type::TypeReflection;

/// Represents a constant reflection in the codebase.
///
/// A `ConstantReflection` provides metadata about a single constant, including its
/// name, type, and location in the source code. Constants are uniquely identified
/// and separated even when declared in a single statement, such as `const A = 1, B = 2;`.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ConstantReflection {
    /// Collection of attributes applied to the constant.
    pub attribute_reflections: Vec<AttributeReflection>,

    /// The name of the constant.
    pub name: Name,

    /// The type reflection of the constant.
    pub type_reflection: TypeReflection,

    /// The span of the specific constant item (`A = 1` in `const A = 1, B = 2;`).
    pub item_span: Span,

    /// The span of the entire constant definition (`const A = 1, B = 2;`).
    pub definition_span: Span,

    /// Whether the reflection's metadata is fully populated.
    pub is_populated: bool,

    /// Collection of issues related to the constant.
    pub issues: IssueCollection,
}

impl HasSpan for ConstantReflection {
    /// Returns the span of the constant item in the source code.
    ///
    /// This span includes just the `A = 1` part of the constant definition.
    fn span(&self) -> Span {
        self.item_span
    }
}

impl HasSource for ConstantReflection {
    /// Returns the source identifier of the file containing this constant.
    ///
    /// The source identifier includes metadata about the file or context where the constant
    /// is defined, such as whether it is a user-defined, vendor-provided, or built-in constant.
    fn source(&self) -> SourceIdentifier {
        self.span().source()
    }
}

impl Reflection for ConstantReflection {
    /// Returns the category of the source where the constant is defined.
    ///
    /// The category indicates whether the constant is part of the project (`UserDefined`),
    /// comes from a external library (`External`), or is built into the language (`BuiltIn`).
    fn get_category(&self) -> SourceCategory {
        self.source().category()
    }

    /// Indicates whether the constant's reflection data is fully populated.
    ///
    /// If `is_populated` is `false`, additional processing may be required to resolve
    /// the constant's metadata completely.
    fn is_populated(&self) -> bool {
        self.is_populated
    }

    fn take_issues(&mut self) -> IssueCollection {
        std::mem::take(&mut self.issues)
    }
}
