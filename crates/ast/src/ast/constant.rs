use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::attribute::AttributeList;
use crate::ast::expression::Expression;
use crate::ast::identifier::LocalIdentifier;
use crate::ast::keyword::Keyword;
use crate::ast::terminator::Terminator;
use crate::sequence::Sequence;
use crate::sequence::TokenSeparatedSequence;

/// Represents a constant statement in PHP.
///
/// Example: `const FOO = 1;` or `const BAR = 2, QUX = 3, BAZ = 4;`
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct Constant<'a> {
    pub attribute_lists: Sequence<'a, AttributeList<'a>>,
    pub r#const: Keyword,
    pub items: TokenSeparatedSequence<'a, ConstantItem<'a>>,
    pub terminator: Terminator,
}

/// Represents a single name-value pair within a constant statement.
#[derive(Debug, Hash, Serialize)]
#[repr(C)]
pub struct ConstantItem<'a> {
    pub name: LocalIdentifier,
    pub equals: Span,
    pub value: Expression<'a>,
}

impl HasSpan for Constant<'_> {
    fn span(&self) -> Span {
        self.r#const.span().join(self.terminator.span())
    }
}

impl HasSpan for ConstantItem<'_> {
    fn span(&self) -> Span {
        self.name.span().join(self.value.span())
    }
}
