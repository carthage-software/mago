use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::argument::ArgumentList;
use crate::ast::identifier::Identifier;
use crate::sequence::TokenSeparatedSequence;

/// Represents a list of attributes.
///
/// Example: `#[Foo, Bar(1)]` in `#[Foo, Bar(1)] class Foo {}`
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct AttributeList<'a> {
    pub hash_left_bracket: Span,
    pub attributes: TokenSeparatedSequence<'a, Attribute<'a>>,
    pub right_bracket: Span,
}

/// Represents a single attribute.
///
/// Example: `Foo` in `#[Foo]`, `Bar(1)` in `#[Bar(1)]`
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Attribute<'a> {
    pub name: Identifier,
    pub arguments: Option<ArgumentList<'a>>,
}

impl HasSpan for AttributeList<'_> {
    fn span(&self) -> Span {
        Span::between(self.hash_left_bracket, self.right_bracket)
    }
}

impl HasSpan for Attribute<'_> {
    fn span(&self) -> Span {
        if let Some(arguments) = &self.arguments {
            Span::between(self.name.span(), arguments.span())
        } else {
            self.name.span()
        }
    }
}
