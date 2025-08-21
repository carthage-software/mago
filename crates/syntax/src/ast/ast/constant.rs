use serde::Serialize;

use mago_span::HasSpan;
use mago_span::Span;

use crate::ast::ast::attribute::AttributeList;
use crate::ast::ast::expression::Expression;
use crate::ast::ast::identifier::LocalIdentifier;
use crate::ast::ast::keyword::Keyword;
use crate::ast::ast::terminator::Terminator;
use crate::ast::sequence::Sequence;
use crate::ast::sequence::TokenSeparatedSequence;

/// Represents a constant statement in PHP.
///
/// Example: `const FOO = 1;` or `const BAR = 2, QUX = 3, BAZ = 4;`
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct Constant<'arena> {
    pub attribute_lists: Sequence<'arena, AttributeList<'arena>>,
    pub r#const: Keyword<'arena>,
    pub items: TokenSeparatedSequence<'arena, ConstantItem<'arena>>,
    pub terminator: Terminator<'arena>,
}

/// Represents a single name-value pair within a constant statement.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, PartialOrd, Ord)]
pub struct ConstantItem<'arena> {
    pub name: LocalIdentifier<'arena>,
    pub equals: Span,
    pub value: Expression<'arena>,
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
