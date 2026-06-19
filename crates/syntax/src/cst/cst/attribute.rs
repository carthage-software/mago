use mago_span::HasSpan;
use mago_span::Span;

use crate::cst::cst::argument::PartialArgumentList;
use crate::cst::cst::identifier::Identifier;
use crate::cst::sequence::TokenSeparatedSequence;

/// Represents a list of attributes.
///
/// Example: `#[Foo, Bar(1)]` in `#[Foo, Bar(1)] class Foo {}`
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AttributeList<'arena> {
    pub hash_left_bracket: Span,
    pub attributes: TokenSeparatedSequence<'arena, Attribute<'arena>>,
    pub right_bracket: Span,
}

/// Represents a single attribute.
///
/// Example: `Foo` in `#[Foo]`, `Bar(1)` in `#[Bar(1)]`
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Attribute<'arena> {
    pub name: Identifier<'arena>,
    pub argument_list: Option<PartialArgumentList<'arena>>,
}

impl HasSpan for AttributeList<'_> {
    fn span(&self) -> Span {
        Span::between(self.hash_left_bracket, self.right_bracket)
    }
}

impl HasSpan for Attribute<'_> {
    fn span(&self) -> Span {
        if let Some(arguments) = &self.argument_list {
            Span::between(self.name.span(), arguments.span())
        } else {
            self.name.span()
        }
    }
}
