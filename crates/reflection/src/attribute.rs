use serde::Deserialize;
use serde::Serialize;

use mago_source::HasSource;
use mago_source::SourceIdentifier;
use mago_span::HasSpan;
use mago_span::Span;

use crate::identifier::Name;
use crate::r#type::TypeReflection;

/// Represents an attribute applied to a class, function, or other PHP constructs.
///
/// Attributes provide metadata for the construct they are applied to, often used
/// for annotations or configuration in frameworks and libraries.
///
/// Example:
///
/// ```php
/// #[Foo, Bar]
/// class Example {}
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct AttributeReflection {
    /// The name of the attribute.
    pub name: Name,

    /// Optional list of arguments provided to the attribute.
    pub arguments: Option<AttributeArgumentListReflection>,

    /// The span of the attribute in the source code.
    pub span: Span,
}

/// Represents a list of arguments for an attribute.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct AttributeArgumentListReflection {
    /// The list of arguments for the attribute.
    pub arguments: Vec<AttributeArgumentReflection>,
}

/// Represents a single argument in an attribute.
///
/// Arguments can either be positional (e.g., `#[Foo("value")]`) or named (e.g., `#[Foo(name: "value")]`).
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum AttributeArgumentReflection {
    /// A positional argument, such as `#[Foo("value")]`.
    Positional {
        /// The type reflection of the argument's value.
        value_type_reflection: TypeReflection,

        /// The span of the argument in the source code.
        span: Span,
    },

    /// A named argument, such as `#[Foo(name: "value")]`.
    Named {
        /// The name of the argument.
        name: Name,

        /// The type reflection of the argument's value.
        value_type_reflection: TypeReflection,

        /// The span of the argument in the source code.
        span: Span,
    },
}

impl HasSource for AttributeReflection {
    /// Returns the source identifier of the file containing this attribute.
    fn source(&self) -> SourceIdentifier {
        self.span.source()
    }
}

impl HasSpan for AttributeReflection {
    /// Returns the span of the attribute in the source code.
    ///
    /// This includes the entire range of the attribute, including its arguments if present.
    fn span(&self) -> Span {
        self.span
    }
}
