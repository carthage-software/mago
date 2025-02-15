use serde::Deserialize;
use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::argument::ArgumentList;
use crate::ast::identifier::Identifier;
use crate::sequence::TokenSeparatedSequence;

/// Represents a list of attributes.
///
/// Example: `#[Foo, Bar(1)]` in `#[Foo, Bar(1)] class Foo {}`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct AttributeList {
    pub hash_left_bracket: Span,
    pub attributes: TokenSeparatedSequence<Attribute>,
    pub right_bracket: Span,
}

/// Represents a single attribute.
///
/// Example: `Foo` in `#[Foo]`, `Bar(1)` in `#[Bar(1)]`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[repr(C)]
pub struct Attribute {
    pub name: Identifier,
    pub arguments: Option<ArgumentList>,
}

impl HasSpan for AttributeList {
    fn span(&self) -> Span {
        Span::between(self.hash_left_bracket, self.right_bracket)
    }
}

impl HasSpan for Attribute {
    fn span(&self) -> Span {
        if let Some(arguments) = &self.arguments {
            Span::between(self.name.span(), arguments.span())
        } else {
            self.name.span()
        }
    }
}
