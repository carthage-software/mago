use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::identifier::Identifier;
use crate::ast::ast::keyword::Keyword;
use crate::ast::sequence::TokenSeparatedSequence;

/// Represents `implements` keyword with one or more types.
///
/// # Example
///
/// ```php
/// <?php
///
/// final class Foo implements Bar, Baz {}
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Implements<'arena> {
    pub implements: Keyword<'arena>,
    pub types: TokenSeparatedSequence<'arena, Identifier<'arena>>,
}

/// Represents `extends` keyword with one or more types.
///
/// # Example
///
/// ```php
/// <?php
///
/// interface Foo extends Bar, Baz {}
/// ```
///
/// ```php
/// <?php
///
/// class Foo extends Bar {}
/// ```
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Extends<'arena> {
    pub extends: Keyword<'arena>,
    pub types: TokenSeparatedSequence<'arena, Identifier<'arena>>,
}

impl HasSpan for Implements<'_> {
    fn span(&self) -> Span {
        let span = self.implements.span();

        Span::between(span, self.types.span(span.file_id, span.end))
    }
}

impl HasSpan for Extends<'_> {
    fn span(&self) -> Span {
        let span = self.extends.span();

        Span::between(span, self.types.span(span.file_id, span.end))
    }
}
