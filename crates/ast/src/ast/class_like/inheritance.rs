use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::identifier::Identifier;
use crate::ast::keyword::Keyword;
use crate::sequence::TokenSeparatedSequence;

/// Represents `implements` keyword with one or more types.
///
/// # Example
///
/// ```php
/// <?php
///
/// final class Foo implements Bar, Baz {}
/// ```
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Implements<'a> {
    pub implements: Keyword,
    pub types: TokenSeparatedSequence<'a, Identifier>,
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
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Extends<'a> {
    pub extends: Keyword,
    pub types: TokenSeparatedSequence<'a, Identifier>,
}

impl HasSpan for Implements<'_> {
    fn span(&self) -> Span {
        let span = self.implements.span();

        Span::between(span, self.types.span(span.end))
    }
}

impl HasSpan for Extends<'_> {
    fn span(&self) -> Span {
        let span = self.extends.span();

        Span::between(span, self.types.span(span.end))
    }
}
